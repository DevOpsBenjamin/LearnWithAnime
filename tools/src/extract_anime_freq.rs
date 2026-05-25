use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let sources_dir = root.join("_sources").join("JP-Subtitles");
    let output_dir = root.join("data").join("anime_freq");
    fs::create_dir_all(&output_dir).expect("Cannot create anime_freq dir");

    println!("🔍 Scanning subtitle directories...");
    let entries = fs::read_dir(&sources_dir).expect("Cannot read JP-Subtitles dir");
    let mut anime_dirs: Vec<String> = Vec::new();
    for entry in entries {
        let entry = entry.expect("Cannot read entry");
        if entry.path().is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                anime_dirs.push(name.to_string());
            }
        }
    }
    anime_dirs.sort();
    println!("   {} anime directories found", anime_dirs.len());

    // Common Japanese particles/function words to skip
    // Using a set for O(1) lookup
    let skip_words: &[&str] = &[
        "は",
        "が",
        "を",
        "に",
        "で",
        "と",
        "も",
        "の",
        "へ",
        "や",
        "から",
        "まで",
        "より",
        "って",
        "じゃ",
        "ちゃ",
        "ね",
        "よ",
        "な",
        "か",
        "さ",
        "わ",
        "ぜ",
        "ぞ",
        "だ",
        "です",
        "ます",
        "ん",
        "さん",
        "ちゃん",
        "くん",
        "様",
        "さま",
        "君",
        "ちん",
        "たち",
        "ら",
        "よう",
        "たい",
        "ない",
        "ぬ",
        "ず",
        "れる",
        "られる",
        "せる",
        "させる",
        "たい",
        "たがる",
        "う",
        "よう",
        "まい",
        "れる",
        "られる",
        "せる",
        "させる",
        "たい",
        "たがる",
        "う",
        "よう",
        "まい",
        "です",
        "ます",
        "た",
        "だ",
        "である",
        "なる",
        "できる",
        "さん",
        "ちゃん",
        "くん",
        "様",
        "君",
        "ちん",
    ];

    println!("📖 Processing anime subtitle files...");
    let total = anime_dirs.len();
    for (idx, anime_name) in anime_dirs.iter().enumerate() {
        let anime_dir = sources_dir.join(anime_name);
        let subtitle_files = collect_subtitle_files(&anime_dir);

        if subtitle_files.is_empty() {
            continue;
        }

        let text = extract_text_from_files(&subtitle_files);
        if text.is_empty() {
            continue;
        }

        let word_counts = count_words_fallback(&text, skip_words);

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
                &serde_json::json!({
                    "word": word,
                    "count": count,
                })
                .to_string(),
            );
            content.push('\n');
        }
        fs::write(&out_path, &content).expect("Cannot write anime freq file");

        if (idx + 1) % 100 == 0 || idx == total - 1 {
            println!(
                "   [{}/{}] {}: {} words, {} files",
                idx + 1,
                total,
                anime_name,
                word_counts.len(),
                subtitle_files.len()
            );
        }
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
        if line.is_empty() || line.contains("-->") || line.chars().all(|c| c.is_ascii_digit()) {
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
                        .replace("{\\i1}", "");
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

fn count_words_fallback(text: &str, skip_words: &[&str]) -> HashMap<String, u64> {
    let mut counts: HashMap<String, u64> = HashMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let tokens = split_jp_text(line);
        for token in &tokens {
            if token.len() < 2 {
                continue;
            }
            if skip_words.contains(&token.as_str()) {
                continue;
            }
            *counts.entry(token.clone()).or_insert(0) += 1;
        }
    }
    counts
}

fn split_jp_text(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for c in text.chars() {
        if is_jap_char(c) {
            current.push(c);
        } else {
            if !current.is_empty() {
                if current.len() >= 2 {
                    tokens.push(current.clone());
                }
                current.clear();
            }
        }
    }
    if !current.is_empty() && current.len() >= 2 {
        tokens.push(current);
    }
    tokens
}

fn is_jap_char(c: char) -> bool {
    let cp = c as u32;
    (cp >= 0x3040 && cp <= 0x309F)
        || (cp >= 0x30A0 && cp <= 0x30FF)
        || (cp >= 0x4E00 && cp <= 0x9FFF)
        || (cp >= 0x3400 && cp <= 0x4DBF)
        || cp == 0x3005
        || cp == 0x30FC
        || cp == 0x3099
        || cp == 0x309A
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
