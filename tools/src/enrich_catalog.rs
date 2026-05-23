use anyhow::{Context, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Radical {
    number: u8,
    char: String,
    meaning: String,
    stroke_count: u8,
}

/// Output struct for our enriched JSONL
#[derive(Debug, Clone, Serialize)]
struct KanjiEntry {
    char: String,
    on_yomi: Vec<String>,
    kun_yomi: Vec<String>,
    meanings_en: Vec<String>,
    components: Vec<String>,
    stroke_count: Option<u8>,
    frequency_rank: Option<u32>,
    jlpt_level: u8,
    grade: Option<u8>,
    radical_number: Option<u8>,
    radical_char: Option<String>,
}

fn parse_kanjidic2(path: &Path) -> Result<HashMap<String, KanjiEntry>> {
    let xml = fs::read_to_string(path).context("Cannot read kanjidic2.xml")?;
    let mut reader = Reader::from_str(&xml);
    reader.trim_text(true);

    let mut kanji_map: HashMap<String, KanjiEntry> = HashMap::new();
    let mut buf = Vec::new();
    let mut in_character = false;
    let mut in_rmgroup = false;
    let mut in_reading_meaning = false;

    let mut current_char = String::new();
    let mut current_on: Vec<String> = Vec::new();
    let mut current_kun: Vec<String> = Vec::new();
    let mut current_meanings: Vec<String> = Vec::new();
    let mut stroke_count: Option<u8> = None;
    let mut freq: Option<u32> = None;
    let mut grade: Option<u8> = None;
    let mut radical_number: Option<u8> = None;
    let mut in_literal = false;
    let mut in_radical = false;
    let mut in_misc = false;
    let mut in_stroke = false;
    let mut in_freq = false;
    let mut in_grade = false;
    let mut in_meaning = false;
    let mut in_reading = false;
    let mut reading_type = String::new();
    let mut in_rad_value = false;
    let mut rad_value_type = String::new();
    let mut _depth = 0u8;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                _depth += 1;
                match e.name().as_ref() {
                    b"character" => {
                        in_character = true;
                        current_char.clear();
                        current_on.clear();
                        current_kun.clear();
                        current_meanings.clear();
                        stroke_count = None;
                        freq = None;
                        grade = None;
                        radical_number = None;
                    }
                    b"literal" if in_character => in_literal = true,
                    b"radical" if in_character => in_radical = true,
                    b"rad_value" if in_radical => {
                        in_rad_value = true;
                        rad_value_type.clear();
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"rad_type" {
                                rad_value_type = String::from_utf8_lossy(&attr.value).to_string();
                            }
                        }
                    }
                    b"misc" if in_character => in_misc = true,
                    b"stroke_count" if in_misc => in_stroke = true,
                    b"freq" if in_misc => in_freq = true,
                    b"grade" if in_misc => in_grade = true,
                    b"reading_meaning" if in_character => in_reading_meaning = true,
                    b"rmgroup" if in_reading_meaning => in_rmgroup = true,
                    b"meaning" if in_rmgroup => {
                        in_meaning = true;
                        // Only accept meanings without m_lang (English)
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"m_lang" {
                                in_meaning = false;
                                break;
                            }
                        }
                    }
                    b"reading" if in_rmgroup => {
                        in_reading = true;
                        reading_type.clear();
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"r_type" {
                                reading_type = String::from_utf8_lossy(&attr.value).to_string();
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                _depth -= 1;
                match e.name().as_ref() {
                    b"character" => {
                        if !current_char.is_empty() {
                            kanji_map.insert(
                                current_char.clone(),
                                KanjiEntry {
                                    char: current_char.clone(),
                                    on_yomi: current_on.clone(),
                                    kun_yomi: current_kun.clone(),
                                    meanings_en: current_meanings.clone(),
                                    components: Vec::new(),
                                    stroke_count,
                                    frequency_rank: freq,
                                    jlpt_level: 0,
                                    grade,
                                    radical_number,
                                    radical_char: None,
                                },
                            );
                        }
                        in_character = false;
                    }
                    b"literal" => in_literal = false,
                    b"radical" => in_radical = false,
                    b"rad_value" => in_rad_value = false,
                    b"misc" => in_misc = false,
                    b"stroke_count" => in_stroke = false,
                    b"freq" => in_freq = false,
                    b"grade" => in_grade = false,
                    b"reading_meaning" => in_reading_meaning = false,
                    b"rmgroup" => in_rmgroup = false,
                    b"meaning" => in_meaning = false,
                    b"reading" => in_reading = false,
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                // <rad_value rad_type="classical">75</rad_value> is actually an Empty event? No, it's Start+Text+End
                match e.name().as_ref() {
                    b"rad_value" if in_radical => {}
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                if in_literal {
                    current_char = text;
                } else if in_stroke {
                    stroke_count = text.parse().ok();
                } else if in_freq {
                    freq = text.parse().ok();
                } else if in_grade {
                    grade = text.parse().ok();
                } else if in_rad_value && rad_value_type == "classical" {
                    radical_number = text.parse().ok();
                } else if in_meaning {
                    if !text.is_empty() {
                        current_meanings.push(text);
                    }
                } else if in_reading {
                    match reading_type.as_str() {
                        "ja_on" => current_on.push(text),
                        "ja_kun" => current_kun.push(text),
                        _ => {}
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => anyhow::bail!("XML parse error: {}", e),
            _ => {}
        }
        buf.clear();
    }

    Ok(kanji_map)
}

fn parse_kradfile(path: &Path) -> Result<HashMap<String, Vec<String>>> {
    let text = fs::read_to_string(path).context("Cannot read kradfile")?;
    let mut map: HashMap<String, Vec<String>> = HashMap::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((kanji, rest)) = line.split_once(':') {
            let kanji = kanji.trim();
            let components: Vec<String> = rest
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();
            if !components.is_empty() {
                map.insert(kanji.to_string(), components);
            }
        }
    }

    Ok(map)
}

fn main() -> Result<()> {
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let sources_dir = project_root.join("_sources");
    let output_dir = project_root
        .parent()
        .unwrap()
        .join("backend")
        .join("data")
        .join("kanji");

    fs::create_dir_all(&output_dir).context("Cannot create output directory")?;

    // 1. Load JLPT mapping
    println!("📖 Chargement du mapping JLPT...");
    let jlpt_raw: String =
        fs::read_to_string(sources_dir.join("jlpt_kanji.json"))
            .context("Cannot read jlpt_kanji.json")?;
    let jlpt_map: HashMap<String, u8> =
        serde_json::from_str(&jlpt_raw).context("Cannot parse jlpt_kanji.json")?;
    println!(
        "   {} kanji dans le mapping JLPT",
        jlpt_map.len()
    );

    // 2. Parse KANJIDIC2
    println!("📖 Parsing de kanjidic2.xml...");
    let kanjidic = parse_kanjidic2(&sources_dir.join("kanjidic2.xml"))?;
    println!("   {} kanji parsés depuis KANJIDIC2", kanjidic.len());

    // 3. Parse KRADFILE
    println!("📖 Parsing de kradfile...");
    let kradfile = parse_kradfile(&sources_dir.join("kradfile"))?;
    println!(
        "   {} kanji avec décompositions",
        kradfile.len()
    );

    // Merge kradfile2 for extended kanji
    let kradfile2_path = sources_dir.join("kradfile2");
    let kradfile_combined = {
        let mut m = kradfile;
        if kradfile2_path.exists() {
            println!("📖 Parsing de kradfile2 (kanji étendus)...");
            let k2 = parse_kradfile(&kradfile2_path)?;
            println!("   {} kanji supplémentaires", k2.len());
            m.extend(k2);
        }
        m
    };

    // Load radicals index for radical_char enrichment
    let radicals_path = project_root
        .parent()
        .unwrap()
        .join("backend")
        .join("data")
        .join("radicals")
        .join("kangxi.json");
    let radicals: Vec<Radical> = if radicals_path.exists() {
        let raw = fs::read_to_string(&radicals_path)?;
        serde_json::from_str(&raw)?
    } else {
        Vec::new()
    };
    let radical_map: HashMap<u8, String> = radicals
        .iter()
        .map(|r| (r.number, r.char.clone()))
        .collect();
    println!(
        "   {} radicaux KangXi chargés",
        radical_map.len()
    );

    // 4. Build output per JLPT level
    let mut not_found: Vec<String> = Vec::new();
    let mut no_krad: Vec<String> = Vec::new();
    let mut by_level: HashMap<u8, Vec<KanjiEntry>> = HashMap::new();
    by_level.insert(5, Vec::new());
    by_level.insert(4, Vec::new());
    by_level.insert(3, Vec::new());
    by_level.insert(2, Vec::new());
    by_level.insert(1, Vec::new());

    for (kanji_char, jlpt_level) in &jlpt_map {
        let level = *jlpt_level;
        if level < 1 || level > 5 {
            continue; // skip invalid
        }

        let mut entry = match kanjidic.get(kanji_char) {
            Some(e) => e.clone(),
            None => {
                not_found.push(format!("{} (N{})", kanji_char, level));
                continue;
            }
        };

        entry.jlpt_level = level;

        // Add radical character from radicals index
        entry.radical_char = entry
            .radical_number
            .and_then(|n| radical_map.get(&n))
            .cloned();

        // Add components from KRADFILE
        if let Some(comps) = kradfile_combined.get(kanji_char) {
            entry.components = comps.clone();
        } else {
            no_krad.push(format!("{} (N{})", kanji_char, level));
        }

        by_level.entry(level).or_default().push(entry);
    }

    // 5. Sort each level by frequency rank (or alphabetically if no freq)
    for (_level, entries) in by_level.iter_mut() {
        entries.sort_by(|a, b| {
            a.frequency_rank
                .unwrap_or(u32::MAX)
                .cmp(&b.frequency_rank.unwrap_or(u32::MAX))
        });
    }

    // 6. Write JSONL files
    let mut total_written = 0;
    let level_names = [1u8, 2, 3, 4, 5];
    for level in level_names {
        let entries = by_level.get(&level).unwrap();
        let filename = format!("jlpt-n{}.jsonl", level);
        let path = output_dir.join(&filename);
        let mut file = fs::File::create(&path)
            .with_context(|| format!("Cannot create {}", filename))?;
        for entry in entries {
            let line = serde_json::to_string(entry)?;
            writeln!(file, "{}", line)?;
        }
        total_written += entries.len();
        println!(
            "   ✍️  N{:>2}: {:>4} kanji → {}",
            level,
            entries.len(),
            filename
        );
    }

    // 7. Report
    println!("\n📊 Rapport final :");
    println!("   Total écrits  : {}", total_written);
    println!("   Non trouvés   : {} ({})", not_found.len(), if not_found.is_empty() { "✓" } else { "⚠️" });
    if !not_found.is_empty() {
        for nf in &not_found {
            println!("                    • {}", nf);
        }
    }
    println!("   Sans KRADFILE : {} ({})", no_krad.len(), if no_krad.len() < 50 { "✓" } else { "ℹ️ > 50" });
    if no_krad.len() <= 50 && !no_krad.is_empty() {
        for nk in &no_krad {
            println!("                    • {}", nk);
        }
    }

    Ok(())
}
