use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

const SKIP_POS: &[&str] = &[
    "助詞", "助動詞", "記号", "補助記号", "接続詞", "連体詞", "フィラー", "接頭詞",
];

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <anime-dir> [--merge] [--min-count N]", args[0]);
        std::process::exit(1);
    }

    let anime_dir = Path::new(&args[1]);
    let merge = args.contains(&"--merge".to_string());
    let min_count = args.windows(2)
        .find(|w| w[0] == "--min-count")
        .and_then(|w| w[1].parse().ok())
        .unwrap_or(1u64);

    if !anime_dir.is_dir() {
        eprintln!("❌ Not a directory: {}", anime_dir.display());
        std::process::exit(1);
    }

    if Command::new("mecab").arg("--version").output().is_err() {
        eprintln!("❌ mecab not found. Install with: brew install mecab mecab-ipadic");
        std::process::exit(1);
    }

    let anime_name = anime_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let output_dir = root.join("data").join("anime_freq");
    fs::create_dir_all(&output_dir).expect("Cannot create anime_freq dir");

    let slug = slugify(anime_name);

    // Collect subtitle files
    let files = collect_subtitle_files(anime_dir);
    if files.is_empty() {
        eprintln!("⚠️ {}: no .srt / .ass files found", anime_name);
        std::process::exit(0);
    }
    println!("📁 {} ({} files)", anime_name, files.len());

    // Extract text from all files
    let mut all_text = String::new();
    let mut files_with_jp = 0usize;
    for f in &files {
        let fname = Path::new(f)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(f);

        let raw = match fs::read(f) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("   ⚠️ {}: {}", fname, e);
                continue;
            }
        };

        let content = match String::from_utf8(raw) {
            Ok(s) => s,
            Err(e) => {
                let bytes = e.into_bytes();
                let (decoded, _, _) = encoding_rs::SHIFT_JIS.decode(&bytes);
                decoded.to_string()
            }
        };

        let mut extracted = String::new();
        if f.ends_with(".ass") {
            extract_ass_text(&content, &mut extracted);
        } else {
            extract_srt_text(&content, &mut extracted);
        }

        if !extracted.trim().is_empty() {
            files_with_jp += 1;
            all_text.push_str(&extracted);
        }
    }

    if all_text.trim().is_empty() {
        eprintln!("⚠️ {}: no Japanese text found", anime_name);
        std::process::exit(0);
    }

    println!("   {} files with Japanese text", files_with_jp);

    let mecab_start = Instant::now();
    let mut child = Command::new("mecab")
        .arg("-b")
        .arg("262144")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Cannot spawn mecab");

    let mut child_stdin = child.stdin.take().unwrap();
    let child_stdout = child.stdout.take().unwrap();
    let text_bytes = all_text.as_bytes().to_vec();

    let writer = std::thread::spawn(move || {
        child_stdin.write_all(&text_bytes).unwrap();
    });

    let reader = BufReader::new(child_stdout);
    let mut counts: HashMap<String, u64> = HashMap::new();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        let line = line.trim().to_string();
        if line.is_empty() || line == "EOS" {
            continue;
        }

        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }
        let surface = parts[0];
        let features: Vec<&str> = parts[1].split(',').collect();
        if features.is_empty() {
            continue;
        }

        let pos = features[0];
        if SKIP_POS.contains(&pos) {
            continue;
        }
        if surface.len() < 2 {
            continue;
        }
        let word = features.get(6).unwrap_or(&surface);
        if word.len() < 2 {
            continue;
        }
        *counts.entry(word.to_string()).or_insert(0) += 1;
    }

    writer.join().unwrap();
    let _ = child.wait();
    let mecab_elapsed = mecab_start.elapsed().as_secs_f64();

    let out_path = output_dir.join(format!("{}.jsonl", slug));

    // Merge existing counts if --merge
    let mut merged = if merge && out_path.exists() {
        load_existing_counts(&out_path)
    } else {
        HashMap::new()
    };
    for (word, count) in &counts {
        *merged.entry(word.clone()).or_insert(0) += count;
    }

    // Write output sorted by frequency (filtering by min_count)
    let mut content = String::new();
    let mut entries: Vec<(&str, u64)> =
        merged.iter().map(|(w, c)| (w.as_str(), *c)).collect();
    entries.sort_by(|a, b| b.1.cmp(&a.1));
    for (word, count) in &entries {
        if *count < min_count {
            continue;
        }
        content.push_str(&serde_json::json!({ "word": word, "count": count }).to_string());
        content.push('\n');
    }
    fs::write(&out_path, &content).expect("Cannot write freq file");

    println!(
        "   ✅ {} words unique (min {}+), {} total, mecab {:.2}s → {}",
        merged.iter().filter(|(_, c)| **c >= min_count).count(),
        min_count,
        merged.values().sum::<u64>(),
        mecab_elapsed,
        out_path.file_name().unwrap().to_string_lossy()
    );
}

fn load_existing_counts(path: &Path) -> HashMap<String, u64> {
    let mut map = HashMap::new();
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return map,
    };
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
            let word = v["word"].as_str().unwrap_or_default().to_string();
            let count = v["count"].as_u64().unwrap_or(0);
            if !word.is_empty() {
                map.insert(word, count);
            }
        }
    }
    map
}

fn collect_subtitle_files(dir: &Path) -> Vec<String> {
    let mut files = Vec::new();
    collect_files_inner(dir, &mut files);
    files
}

fn collect_files_inner(dir: &Path, files: &mut Vec<String>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if path.is_dir() {
            collect_files_inner(&path, files);
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if ext == "srt" || ext == "ass" {
                if let Some(p) = path.to_str() {
                    files.push(p.to_string());
                }
            }
        }
    }
}

fn extract_srt_text(content: &str, out: &mut String) {
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty()
            || line.contains("-->")
            || line.chars().all(|c| c.is_ascii_digit())
        {
            continue;
        }
        if line.starts_with('<') && line.ends_with('>') {
            continue;
        }
        if !line.contains("WEBVTT") && has_japanese(line) {
            out.push_str(line);
            out.push('\n');
        }
    }
}

fn extract_ass_text(content: &str, out: &mut String) {
    let mut in_events = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("[Events]") {
            in_events = true;
            continue;
        }
        if trimmed.starts_with('[') && in_events {
            break;
        }
        if !in_events {
            continue;
        }
        if trimmed.starts_with("Dialogue:") {
            let parts: Vec<&str> = trimmed.splitn(10, ',').collect();
            if parts.len() >= 10 {
                let style = parts[3].trim();
                let text = parts[9];
                if style.contains("JP") || style.contains("Default") {
                    let cleaned = text
                        .replace("\\N", " ")
                        .replace("\\n", " ")
                        .replace("{\\i0}", "")
                        .replace("{\\i1}", "")
                        .replace("{\\b0}", "")
                        .replace("{\\b1}", "");
                    if has_japanese(&cleaned) {
                        out.push_str(&cleaned);
                        out.push('\n');
                    }
                }
            }
        }
    }
}

fn has_japanese(s: &str) -> bool {
    s.chars().any(|c| {
        let cp = c as u32;
        (cp >= 0x3040 && cp <= 0x309F)
            || (cp >= 0x30A0 && cp <= 0x30FF)
            || (cp >= 0x4E00 && cp <= 0x9FFF)
            || (cp >= 0x3400 && cp <= 0x4DBF)
    })
}

fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}
