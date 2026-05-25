use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let sources_dir = root.join("_sources").join("JP-Subtitles");
    let output_dir = root.join("data").join("anime_freq");
    fs::create_dir_all(&output_dir).expect("Cannot create anime_freq dir");

    // Verify mecab
    if Command::new("mecab").arg("--version").output().is_err() {
        eprintln!("❌ mecab not found. Install with: brew install mecab mecab-ipadic");
        std::process::exit(1);
    }

    println!("🔍 Scanning subtitle directories...");
    let entries = fs::read_dir(&sources_dir).expect("Cannot read JP-Subtitles dir");
    let mut anime_dirs: Vec<String> = Vec::new();
    for entry in entries {
        let entry = entry.expect("Cannot read entry");
        if entry.path().is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                if !name.starts_with('.') {
                    anime_dirs.push(name.to_string());
                }
            }
        }
    }
    anime_dirs.sort();
    println!("   {} anime directories found", anime_dirs.len());

    println!("📖 Processing with MeCab tokenizer...");
    let total = anime_dirs.len();

    for (idx, anime_name) in anime_dirs.iter().enumerate() {
        let anime_dir = sources_dir.join(anime_name);
        let files = collect_subtitle_files(&anime_dir);
        if files.is_empty() {
            let pct = (idx + 1) as f64 / total as f64 * 100.0;
            println!("   [{}/{}] {} (skipped) {:.0}%",
                idx + 1, total, anime_name, pct);
            std::io::stdout().flush().ok();
            continue;
        }

        let text = extract_text_from_files(&files);
        if text.is_empty() {
            let pct = (idx + 1) as f64 / total as f64 * 100.0;
            println!("   [{}/{}] {} (no text) {:.0}%",
                idx + 1, total, anime_name, pct);
            std::io::stdout().flush().ok();
            continue;
        }

        let word_counts = match count_words_mecab(&text) {
            Ok(map) => map,
            Err(e) => {
                eprintln!("   ⚠️ {}: {}", anime_name, e);
                continue;
            }
        };

        if word_counts.is_empty() {
            continue;
        }

        let slug = slugify(anime_name);
        let out_path = output_dir.join(format!("{}.jsonl", slug));
        let mut content = String::new();
        let mut entries: Vec<(&str, u64)> =
            word_counts.iter().map(|(w, c)| (w.as_str(), *c)).collect();
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
            idx + 1, total, anime_name, word_counts.len(), files.len(), pct);
        std::io::stdout().flush().ok();
    }

    println!("\n✅ Done! Files in {}", output_dir.display());
}

fn collect_subtitle_files(dir: &Path) -> Vec<String> {
    let mut files = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return files,
    };
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if ext == "srt" || ext == "ass" || ext == "txt" {
                if let Some(p) = path.to_str() {
                    files.push(p.to_string());
                }
            }
        }
    }
    files
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

    // Feed Japanese text line by line with newlines as sentence boundaries
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

    // Read output
    let reader = BufReader::new(child.stdout.take().unwrap());
    let mut counts: HashMap<String, u64> = HashMap::new();

    // Skip POS categories we don't want
    let skip_pos = &[
        "助詞",       // particle
        "助動詞",     // auxiliary verb
        "記号",       // symbol
        "補助記号",   // supplementary symbol
        "接続詞",     // conjunction
        "連体詞",     // pre-noun adjectival
        "フィラー",   // filler (あのー, えーと)
        "接頭詞",     // prefix
    ];

    for line in reader.lines() {
        let line = line.map_err(|e| format!("Cannot read mecab output: {}", e))?;
        let line = line.trim();
        if line.is_empty() || line == "EOS" {
            continue;
        }

        // Format: 表層形\t品詞,品詞細分類1,...,原形,読み,発音
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

        // Skip unwanted POS
        if skip_pos.contains(&pos) {
            continue;
        }

        // Skip 1-character tokens (particles, single kana)
        if surface.len() < 2 {
            continue;
        }

        // Use the dictionary form (原形) at index 6 if available, otherwise surface
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
