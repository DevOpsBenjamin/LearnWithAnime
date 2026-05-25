use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use uuid::Uuid;

const NAMESPACE: Uuid = Uuid::from_u128(0x6ba7b811_9dad_11d1_80b4_00c04fd430c8);
const CARD_BATCH_SIZE: usize = 1000;

#[derive(serde::Deserialize, Debug)]
struct JmdictEntry {
    id: u64,
    kanji: Vec<String>,
    readings: Vec<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pos: Vec<String>,
    glosses: Vec<String>,
    #[serde(default)]
    freq: Vec<String>,
}

#[derive(serde::Serialize, Debug)]
struct Card {
    id: String,
    kanji: String,
    romaji: String,
    fr: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    readings: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    meanings_en: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    jlpt_level: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    jmdict_seq: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    total_freq: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pos: Option<Vec<String>>,
}

fn load_jmdict(path: &Path) -> Vec<JmdictEntry> {
    let content = fs::read_to_string(path).expect("Cannot read jmdict.jsonl");
    let mut entries = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        entries.push(serde_json::from_str::<JmdictEntry>(line).expect("Cannot parse JMdict line"));
    }
    entries
}

fn build_jmdict_index(
    entries: &[JmdictEntry],
) -> (HashMap<String, Vec<usize>>, HashMap<String, Vec<usize>>) {
    let mut kanji_index: HashMap<String, Vec<usize>> = HashMap::new();
    let mut reading_index: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, entry) in entries.iter().enumerate() {
        for k in &entry.kanji {
            kanji_index.entry(k.clone()).or_default().push(i);
        }
        for r in &entry.readings {
            let normalized = r.replace('ー', "");
            reading_index.entry(normalized).or_default().push(i);
        }
    }
    (kanji_index, reading_index)
}

fn load_jlpt_vocab(path: &Path) -> HashMap<String, u8> {
    let content = fs::read_to_string(path).expect("Cannot read jlpt_vocab.json");
    let raw: HashMap<String, Vec<serde_json::Value>> =
        serde_json::from_str(&content).expect("Cannot parse jlpt_vocab.json");
    let mut result: HashMap<String, u8> = HashMap::new();
    for (word, entries) in raw {
        let mut best_level = 5u8;
        for entry in &entries {
            if let Some(level) = entry.get("level").and_then(|v| v.as_u64()) {
                // level 5 = N5 (easiest), level 1 = N1 (hardest)
                if level < best_level as u64 {
                    best_level = level as u8;
                }
            }
        }
        result.insert(word, best_level);
    }
    result
}

fn load_kanji_jlpt(base_dir: &Path) -> HashMap<String, u8> {
    let kanji_dir = base_dir.join("kanji");
    let mut result = HashMap::new();
    if !kanji_dir.exists() {
        return result;
    }
    let entries = fs::read_dir(&kanji_dir).expect("Cannot read kanji dir");
    for entry in entries {
        let entry = entry.expect("Cannot read entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
            continue;
        }
        let content = fs::read_to_string(&path).expect("Cannot read kanji file");
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let v: serde_json::Value = serde_json::from_str(line).expect("Cannot parse kanji line");
            let ch = v["char"].as_str().unwrap_or_default().to_string();
            let level = v["jlpt_level"].as_u64().unwrap_or(5) as u8;
            if !ch.is_empty() {
                result.insert(ch, level);
            }
        }
    }
    result
}

fn load_existing_ids(cards_dir: &Path) -> HashSet<String> {
    let mut ids = HashSet::new();
    if !cards_dir.exists() {
        return ids;
    }
    let entries = fs::read_dir(cards_dir).expect("Cannot read cards dir");
    for entry in entries {
        let entry = entry.expect("Cannot read entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
            continue;
        }
        let content = fs::read_to_string(&path).expect("Cannot read card file");
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let v: serde_json::Value = serde_json::from_str(line).expect("Cannot parse card line");
            if let Some(id) = v["id"].as_str() {
                ids.insert(id.to_string());
            }
        }
    }
    ids
}

fn kana_to_romaji(s: &str, kana_map: &HashMap<String, String>) -> String {
    // Simple per-character mapping for hiragana/katakana
    let mut result = String::new();
    let mut i = 0;
    let len = s.len();
    while i < len {
        // Try 2 characters (6 bytes for two 3-byte UTF-8 chars)
        let mut found = false;
        if i + 6 <= len {
            let substr = &s[i..i + 6];
            if let Some(roma) = kana_map.get(substr) {
                result.push_str(roma);
                i += 6;
                found = true;
            }
        }
        if !found && i + 3 <= len {
            let substr = &s[i..i + 3];
            if let Some(roma) = kana_map.get(substr) {
                result.push_str(roma);
                i += 3;
                found = true;
            }
        }
        if !found {
            // Skip unknown character
            let c = s[i..].chars().next().unwrap();
            result.push(c);
            i += c.len_utf8();
        }
    }
    result
}

fn compute_jlpt_for_word(
    word: &str,
    jlpt_vocab: &HashMap<String, u8>,
    kanji_jlpt: &HashMap<String, u8>,
) -> Option<u8> {
    // First try direct vocab match
    if let Some(&level) = jlpt_vocab.get(word) {
        return Some(level);
    }
    // Fallback: check each kanji character, take the hardest (lowest number)
    let mut hardest: Option<u8> = None;
    for c in word.chars() {
        if let Some(&level) = kanji_jlpt.get(&c.to_string()) {
            match hardest {
                None => hardest = Some(level),
                Some(h) if level < h => hardest = Some(level),
                _ => {}
            }
        }
    }
    hardest
}

fn load_kana_map(path: &Path) -> HashMap<String, String> {
    let content = fs::read_to_string(path).expect("Cannot read kana.json");
    let entries: Vec<serde_json::Value> =
        serde_json::from_str(&content).expect("Cannot parse kana.json");
    let mut map = HashMap::new();
    for entry in entries {
        let ch = entry["char"].as_str().unwrap_or_default().to_string();
        let roma = entry["romaji"].as_str().unwrap_or_default().to_string();
        map.insert(ch, roma);
    }
    map
}

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let sources_dir = root.join("_sources");
    let data_dir = root.join("data");
    let backend_data = root.parent().unwrap().join("backend").join("data");
    let cards_dir = backend_data.join("cards");
    let kana_path = backend_data.join("kana").join("kana.json");

    println!("📖 Loading kana map...");
    let kana_map = load_kana_map(&kana_path);
    println!("   {} kana mappings loaded", kana_map.len());

    println!("📖 Loading JMdict...");
    let jmdict = load_jmdict(&data_dir.join("jmdict.jsonl"));
    println!("   {} JMdict entries loaded", jmdict.len());
    let (kanji_idx, reading_idx) = build_jmdict_index(&jmdict);
    println!(
        "   {} kanji keys, {} reading keys indexed",
        kanji_idx.len(),
        reading_idx.len()
    );

    println!("📖 Loading JLPT vocab...");
    let jlpt_vocab = load_jlpt_vocab(&sources_dir.join("jlpt_vocab.json"));
    println!("   {} words in JLPT vocab", jlpt_vocab.len());

    println!("📖 Loading kanji JLPT levels...");
    let kanji_jlpt = load_kanji_jlpt(&backend_data);
    println!("   {} kanji with JLPT levels", kanji_jlpt.len());

    println!("📖 Loading existing cards...");
    let existing_ids = load_existing_ids(&cards_dir);
    println!("   {} existing card IDs", existing_ids.len());

    println!("📖 Reading word frequency list...");
    let freq_content = fs::read_to_string(
        sources_dir
            .join("word-freq-lists")
            .join("word_freq_report.txt"),
    )
    .expect("Cannot read word_freq_report.txt");
    let freq_lines: Vec<&str> = freq_content.lines().filter(|l| !l.is_empty()).collect();
    println!("   {} words in frequency list", freq_lines.len());

    // Process words
    let mut new_cards: Vec<Card> = Vec::new();
    let mut skipped_existing = 0u64;

    for (_line_idx, line) in freq_lines.iter().enumerate() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }
        let freq: u64 = parts[0].parse().unwrap_or(0);
        let word = parts[1];
        let pos_tag = parts.get(6).map(|s| s.to_string()).unwrap_or_default();

        if word.is_empty() {
            continue;
        }

        // Skip pure English/numeric (words with only ASCII letters or digits)
        if word
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c.is_ascii_punctuation())
        {
            continue;
        }
        // Must contain at least one Japanese character
        if !word.chars().any(|c| c as u32 > 0x2E80) {
            continue;
        }

        // Generate UUID
        let uuid = generate_uuid(word, None);

        // Check if card already exists
        if existing_ids.contains(&uuid.to_string()) {
            skipped_existing += 1;
            continue;
        }

        // Look up in JMdict
        let jmdict_match = find_jmdict_match(word, &jmdict, &kanji_idx, &reading_idx);

        let (romaji, readings, meanings, jmdict_seq) = if let Some(entry) = jmdict_match {
            let readings_str = entry.readings.clone();
            let roma = readings_str
                .first()
                .map(|r| kana_to_romaji(r, &kana_map))
                .unwrap_or_default();
            (
                roma,
                Some(entry.readings.clone()),
                if entry.glosses.is_empty() {
                    None
                } else {
                    Some(entry.glosses.clone())
                },
                Some(entry.id),
            )
        } else {
            // No JMdict match: try to convert kana to romaji
            let roma = kana_to_romaji(word, &kana_map);
            (
                if roma == word { String::new() } else { roma },
                None,
                None,
                None,
            )
        };

        // JLPT level
        let jlpt = compute_jlpt_for_word(word, &jlpt_vocab, &kanji_jlpt);

        let pos = if pos_tag.is_empty() {
            None
        } else {
            Some(vec![pos_tag])
        };

        let card = Card {
            id: uuid.to_string(),
            kanji: word.to_string(),
            romaji,
            fr: meanings.as_ref().map(|m| m.join("; ")).unwrap_or_default(),
            readings,
            meanings_en: None, // already in fr
            jlpt_level: jlpt,
            jmdict_seq,
            total_freq: Some(freq),
            pos,
        };

        new_cards.push(card);
    }

    // Sort by frequency descending
    new_cards.sort_by(|a, b| b.total_freq.unwrap_or(0).cmp(&a.total_freq.unwrap_or(0)));

    println!("\n📊 Results:");
    println!("   Total words in freq list: {}", freq_lines.len());
    println!("   Cards generated: {}", new_cards.len());
    println!("   Skipped (already exist): {}", skipped_existing);

    // Write in batches of CARD_BATCH_SIZE
    println!("\n📝 Writing card files...");
    fs::create_dir_all(&cards_dir).expect("Cannot create cards dir");

    let batch_count = (new_cards.len() + CARD_BATCH_SIZE - 1) / CARD_BATCH_SIZE;
    for batch_idx in 0..batch_count {
        let start = batch_idx * CARD_BATCH_SIZE;
        let end = (start + CARD_BATCH_SIZE).min(new_cards.len());
        let batch = &new_cards[start..end];

        // File number: batch 0 → 001.jsonl, batch 4 → 005.jsonl
        let file_num = batch_idx + 1;
        let filename = format!("{:03}.jsonl", file_num);
        let out_path = cards_dir.join(&filename);

        let mut content = String::new();
        for card in batch {
            content.push_str(&serde_json::to_string(card).expect("Cannot serialize card"));
            content.push('\n');
        }
        fs::write(&out_path, &content).expect("Cannot write card file");
        println!(
            "   {}: {} cards (rank {}-{})",
            filename,
            batch.len(),
            start + 1,
            end
        );
    }

    println!(
        "\n✅ Done! {} cards written in {} files",
        new_cards.len(),
        batch_count
    );
}

fn generate_uuid(word: &str, jmdict_seq: Option<u64>) -> Uuid {
    let name = if let Some(seq) = jmdict_seq {
        format!("jmdict:{}", seq)
    } else {
        format!("custom:{}", word)
    };
    Uuid::new_v5(&NAMESPACE, name.as_bytes())
}

fn find_jmdict_match<'a>(
    word: &str,
    entries: &'a [JmdictEntry],
    kanji_idx: &HashMap<String, Vec<usize>>,
    reading_idx: &HashMap<String, Vec<usize>>,
) -> Option<&'a JmdictEntry> {
    // Try exact kanji match first
    if let Some(indices) = kanji_idx.get(word) {
        // Pick the entry with the most frequent tags (ichi1 > news1 > spec1, etc.)
        let best = indices
            .iter()
            .max_by_key(|&&idx| score_jmdict_freq(&entries[idx].freq));
        if let Some(&idx) = best {
            return Some(&entries[idx]);
        }
    }

    // Try reading match (normalized: remove long vowel mark)
    let normalized = word.replace('ー', "");
    if let Some(indices) = reading_idx.get(&normalized) {
        let best = indices
            .iter()
            .max_by_key(|&&idx| score_jmdict_freq(&entries[idx].freq));
        if let Some(&idx) = best {
            return Some(&entries[idx]);
        }
    }

    // Try partial match: check if word is a compound of known kanji entries
    // This is a simpler approach: just return None
    None
}

fn score_jmdict_freq(freq_tags: &[String]) -> u32 {
    let mut score = 0u32;
    for tag in freq_tags {
        score += match tag.as_str() {
            "ichi1" => 100,
            "news1" => 80,
            "spec1" => 60,
            "ichi2" => 50,
            "news2" => 40,
            "spec2" => 30,
            t if t.starts_with("nf") => {
                // nf01 = most frequent, nf48 = least
                if let Ok(n) = t[2..].parse::<u32>() {
                    (49 - n).max(0) * 2
                } else {
                    0
                }
            }
            _ => 0,
        };
    }
    score
}
