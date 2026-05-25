use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

struct Args {
    sources: Vec<PathBuf>,
    merge: bool,
}

fn parse_args() -> Args {
    let mut sources = Vec::new();
    let mut merge = false;
    let mut i = 1;
    let raw: Vec<String> = std::env::args().collect();
    while i < raw.len() {
        match raw[i].as_str() {
            "--source" => {
                i += 1;
                sources.push(PathBuf::from(&raw[i]));
            }
            "--merge" => merge = true,
            other => {
                eprintln!("Unknown arg: {}", other);
                std::process::exit(1);
            }
        }
        i += 1;
    }
    if sources.is_empty() {
        sources.push(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("_sources")
                .join("JP-Subtitles"),
        );
    }
    Args { sources, merge }
}

fn main() {
    let args = parse_args();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let output_dir = root.join("data").join("anime_freq");
    fs::create_dir_all(&output_dir).expect("Cannot create anime_freq dir");

    // Verify mecab
    if Command::new("mecab").arg("--version").output().is_err() {
        eprintln!("❌ mecab not found. Install with: brew install mecab mecab-ipadic");
        std::process::exit(1);
    }

    // Count total anime across all sources
    let mut all_anime: Vec<(String, PathBuf)> = Vec::new(); // (name, dir)
    for src in &args.sources {
        if !src.exists() {
            eprintln!("⚠️ Source not found: {}", src.display());
            continue;
        }
        let entries = match fs::read_dir(src) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("⚠️ Cannot read {}: {}", src.display(), e);
                continue;
            }
        };
        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with('.') {
                    continue;
                }
                all_anime.push((name.to_string(), path));
            }
        }
    }
    all_anime.sort_by(|a, b| a.0.cmp(&b.0));

    let total = all_anime.len();
    println!("📂 {} anime found across {} source(s)", total, args.sources.len());
    println!("🔧 Initializing MeCab...");

    // How to count total effort: we'll collect subtitle files for each anime first
    // Use a single persistent mecab process
    println!("📖 Processing...");

    for (idx, (anime_name, anime_dir)) in all_anime.iter().enumerate() {
        let files = collect_subtitle_files_recursive(anime_dir);

        if files.is_empty() {
            println!("   [{}/{}] {} (skipped)", idx + 1, total, anime_name);
            std::io::stdout().flush().ok();
            continue;
        }

        let text = extract_text_from_files(&files);
        if text.trim().is_empty() {
            println!("   [{}/{}] {} (no text)", idx + 1, total, anime_name);
            std::io::stdout().flush().ok();
            continue;
        }

        println!("   [{}/{}] {} → parsing {} files...", idx + 1, total, anime_name, files.len());
        std::io::stdout().flush().ok();

        let word_counts = match count_words_mecab(&text) {
            Ok(map) => map,
            Err(e) => {
                eprintln!("   ⚠️ {}: {}", anime_name, e);
                continue;
            }
        };

        if word_counts.is_empty() {
            println!("   [{}/{}] {} (0 words)", idx + 1, total, anime_name);
            std::io::stdout().flush().ok();
            continue;
        }

        let slug = slugify(anime_name);
        let out_path = output_dir.join(format!("{}.jsonl", slug));

        // Load existing counts if --merge
        let mut merged = if args.merge && out_path.exists() {
            load_existing_counts(&out_path)
        } else {
            HashMap::new()
        };

        // Add new counts
        for (word, count) in &word_counts {
            *merged.entry(word.clone()).or_insert(0) += count;
        }

        // Write output
        let mut content = String::new();
        let mut entries: Vec<(&str, u64)> =
            merged.iter().map(|(w, c)| (w.as_str(), *c)).collect();
        entries.sort_by(|a, b| b.1.cmp(&a.1));
        for (word, count) in &entries {
            content.push_str(
                &serde_json::json!({ "word": word, "count": count }).to_string(),
            );
            content.push('\n');
        }
        fs::write(&out_path, &content).expect("Cannot write anime freq file");

        let pct = (idx + 1) as f64 / total as f64 * 100.0;
        println!("   [{}/{}] {}: {} words, {} files ({:.0}%)",
            idx + 1, total, anime_name, merged.len(), files.len(), pct);
        std::io::stdout().flush().ok();
    }

    println!("\n✅ Done! {} anime in {}", all_anime.len(), output_dir.display());
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

fn collect_subtitle_files_recursive(dir: &Path) -> Vec<String> {
    let mut files = Vec::new();
    collect_subtitle_files_inner(dir, &mut files);
    files
}

fn collect_subtitle_files_inner(dir: &Path, files: &mut Vec<String>) {
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
            collect_subtitle_files_inner(&path, files);
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if ext == "srt" || ext == "ass" || ext == "txt" {
                if let Some(p) = path.to_str() {
                    files.push(p.to_string());
                }
            }
        }
    }
}

fn extract_text_from_files(files: &[String]) -> String {
    let mut all_text = String::new();
    for f in files {
        let content = match fs::read_to_string(f) {
            Ok(c) => c,
            Err(_) => continue,
        };
        if f.ends_with(".ass") {
            extract_ass_text(&content, &mut all_text);
        } else {
            extract_srt_text(&content, &mut all_text);
        }
    }
    all_text
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
                if style.contains("JP")
                    || style.contains("Default")
                    || style.contains("ED-")
                    || style.contains("OP-")
                {
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

fn count_words_mecab(text: &str) -> Result<HashMap<String, u64>, String> {
    let mut child = Command::new("mecab")
        .arg("-b")
        .arg("65536")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Cannot spawn mecab: {}", e))?;

    {
        let stdin = child.stdin.as_mut().unwrap();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            writeln!(stdin, "{}", line)
                .map_err(|e| format!("Cannot write to mecab: {}", e))?;
        }
    }

    let reader = BufReader::new(child.stdout.take().unwrap());
    let mut counts: HashMap<String, u64> = HashMap::new();

    let skip_pos = &[
        "助詞", "助動詞", "記号", "補助記号", "接続詞", "連体詞", "フィラー", "接頭詞",
    ];

    for line in reader.lines() {
        let line = line.map_err(|e| format!("Cannot read mecab output: {}", e))?;
        let line = line.trim();
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

        if skip_pos.contains(&pos) {
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

    let status = child
        .wait()
        .map_err(|e| format!("Cannot wait for mecab: {}", e))?;
    if !status.success() {
        return Err(format!("mecab exited with: {:?}", status.code()));
    }

    Ok(counts)
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
