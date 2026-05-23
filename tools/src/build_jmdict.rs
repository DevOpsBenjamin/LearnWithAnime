use quick_xml::events::Event;
use quick_xml::Reader;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

#[derive(Debug, Serialize)]
struct JmdictEntry {
    id: u32,
    kanji: Vec<String>,
    readings: Vec<String>,
    pos: Vec<String>,
    glosses: Vec<String>,
    freq: Vec<String>,
}

fn parse_entities(xml: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in xml.lines() {
        let line = line.trim();
        if line.starts_with("<!ENTITY ") {
            let inner = line
                .strip_prefix("<!ENTITY ")
                .and_then(|s| s.strip_suffix('>'))
                .unwrap_or("");
            let mut parts = inner.splitn(2, char::is_whitespace);
            if let (Some(name), Some(value)) = (parts.next(), parts.next()) {
                map.insert(name.to_string(), value.trim_matches('"').to_string());
            }
        }
    }
    map
}

fn resolve_entities(text: &str, entities: &HashMap<String, String>) -> String {
    let mut result = text.to_string();
    for (name, value) in entities {
        result = result.replace(&format!("&{};", name), value);
    }
    result
}

fn collect_text(reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>) -> String {
    let mut text = String::new();
    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Text(e)) => {
                if let Ok(t) = e.unescape() {
                    text.push_str(&t);
                }
            }
            Ok(Event::End(_)) => break,
            Ok(Event::Eof) => break,
            _ => {}
        }
        buf.clear();
    }
    text
}

fn skip_element(reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>) {
    let mut depth = 1;
    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(_)) => depth += 1,
            Ok(Event::End(_)) => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            Ok(Event::Empty(_)) => {}
            Ok(Event::Eof) => break,
            _ => {}
        }
        buf.clear();
    }
}

fn main() -> anyhow::Result<()> {
    let sources_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("_sources");
    let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data");
    std::fs::create_dir_all(&data_dir)?;

    let xml_path = sources_dir.join("jmdict_e.xml");
    if !xml_path.exists() {
        anyhow::bail!(
            "JMdict file not found at {}. Run the download command from README.md first.",
            xml_path.display()
        );
    }

    let xml_string = std::fs::read_to_string(&xml_path)?;
    let entities = parse_entities(&xml_string);
    let xml_resolved = resolve_entities(&xml_string, &entities);

    let out_path = data_dir.join("jmdict.jsonl");
    let out_file = File::create(&out_path)?;
    let mut writer = BufWriter::new(out_file);

    let mut reader = Reader::from_str(&xml_resolved);
    reader.trim_text(true);
    let mut buf = Vec::new();
    let mut count = 0u32;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"entry" => {
                let entry = parse_entry(&mut reader, &mut buf)?;
                let json = serde_json::to_string(&entry)?;
                writeln!(writer, "{}", json)?;
                count += 1;
                if count % 10000 == 0 {
                    eprintln!("  {} entries parsed...", count);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => anyhow::bail!("XML parse error: {}", e),
            _ => {}
        }
        buf.clear();
    }

    eprintln!(
        "✅ Parsed {} entries -> {}",
        count,
        out_path.display()
    );
    eprintln!("   ({} entities resolved)", entities.len());
    Ok(())
}

fn parse_entry(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
) -> anyhow::Result<JmdictEntry> {
    let mut id = 0u32;
    let mut kanji = Vec::new();
    let mut readings = Vec::new();
    let mut pos = Vec::new();
    let mut glosses = Vec::new();
    let mut freq = Vec::new();

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(ref e)) => match e.name().as_ref() {
                b"ent_seq" => {
                    let t = collect_text(reader, buf);
                    id = t.parse::<u32>().unwrap_or(0);
                }
                b"k_ele" => parse_kanji_element(reader, buf, &mut kanji, &mut freq)?,
                b"r_ele" => parse_reading_element(reader, buf, &mut readings, &mut freq)?,
                b"sense" => parse_sense(reader, buf, &mut pos, &mut glosses)?,
                _ => {
                    skip_element(reader, buf);
                }
            },
            Ok(Event::End(ref e)) if e.name().as_ref() == b"entry" => break,
            Ok(Event::Eof) => break,
            Err(e) => anyhow::bail!("Entry parse error: {}", e),
            _ => {}
        }
        buf.clear();
    }

    freq.sort();
    freq.dedup();

    Ok(JmdictEntry {
        id,
        kanji,
        readings,
        pos,
        glosses,
        freq,
    })
}

fn parse_kanji_element(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    kanji: &mut Vec<String>,
    freq: &mut Vec<String>,
) -> anyhow::Result<()> {
    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(ref e)) => match e.name().as_ref() {
                b"keb" => {
                    let t = collect_text(reader, buf);
                    if !t.is_empty() {
                        kanji.push(t);
                    }
                }
                b"ke_pri" => {
                    let t = collect_text(reader, buf);
                    if !t.is_empty() {
                        freq.push(t);
                    }
                }
                _ => {
                    skip_element(reader, buf);
                }
            },
            Ok(Event::End(ref e)) if e.name().as_ref() == b"k_ele" => break,
            Ok(Event::Eof) => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(())
}

fn parse_reading_element(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    readings: &mut Vec<String>,
    freq: &mut Vec<String>,
) -> anyhow::Result<()> {
    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(ref e)) => match e.name().as_ref() {
                b"reb" => {
                    let t = collect_text(reader, buf);
                    if !t.is_empty() {
                        readings.push(t);
                    }
                }
                b"re_pri" => {
                    let t = collect_text(reader, buf);
                    if !t.is_empty() {
                        freq.push(t);
                    }
                }
                _ => {
                    skip_element(reader, buf);
                }
            },
            Ok(Event::End(ref e)) if e.name().as_ref() == b"r_ele" => break,
            Ok(Event::Eof) => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(())
}

fn parse_sense(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    pos: &mut Vec<String>,
    glosses: &mut Vec<String>,
) -> anyhow::Result<()> {
    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(ref e)) => match e.name().as_ref() {
                b"pos" => {
                    let t = collect_text(reader, buf);
                    if !t.is_empty() {
                        pos.push(t);
                    }
                }
                b"gloss" => {
                    let t = collect_text(reader, buf);
                    if !t.is_empty() {
                        glosses.push(t);
                    }
                }
                _ => {
                    skip_element(reader, buf);
                }
            },
            Ok(Event::End(ref e)) if e.name().as_ref() == b"sense" => break,
            Ok(Event::Eof) => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(())
}
