use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct Card {
    pub id: String,
    pub kanji: String,
    pub romaji: String,
    pub fr: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeckCardLink {
    pub card_id: String,
    pub anime_reference: Option<String>,
    pub context_sentence: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Deck {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub cards: Vec<DeckCardLink>,
}

#[derive(Debug, Clone, Deserialize)]
struct CardsFile {
    cards: Vec<Card>,
}

#[derive(Debug, Clone, Deserialize)]
struct DeckFileMeta {
    slug: String,
    name: String,
    description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct DeckFile {
    deck: DeckFileMeta,
    cards: Vec<DeckCardLink>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KanjiEntry {
    pub char: String,
    pub on_yomi: Vec<String>,
    pub kun_yomi: Vec<String>,
    pub meanings_en: Vec<String>,
    pub components: Vec<String>,
    pub stroke_count: Option<u8>,
    pub frequency_rank: Option<u32>,
    pub jlpt_level: u8,
    pub grade: Option<u8>,
    pub radical_number: Option<u8>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Radical {
    pub number: u8,
    pub char: String,
    pub meaning: String,
    pub stroke_count: u8,
    #[serde(default)]
    pub variants: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CardCatalog {
    pub cards: HashMap<String, Card>,
    pub decks: HashMap<String, Deck>,
    pub kanji: HashMap<String, KanjiEntry>,
    pub radicals: HashMap<u8, Radical>,
}

#[derive(Debug)]
pub enum LoadError {
    Io(String),
    DuplicateCardId {
        id: String,
        file1: String,
        file2: String,
    },
    DuplicateDeckSlug {
        slug: String,
        file1: String,
        file2: String,
    },
    UnresolvedCardRef {
        deck_slug: String,
        card_id: String,
    },
    InvalidSlug {
        slug: String,
        kind: &'static str,
    },
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::Io(msg) => write!(f, "IO error: {}", msg),
            LoadError::DuplicateCardId { id, file1, file2 } => {
                write!(
                    f,
                    "Duplicate card id '{}' in files '{}' and '{}'",
                    id, file1, file2
                )
            }
            LoadError::DuplicateDeckSlug { slug, file1, file2 } => {
                write!(
                    f,
                    "Duplicate deck slug '{}' in files '{}' and '{}'",
                    slug, file1, file2
                )
            }
            LoadError::UnresolvedCardRef { deck_slug, card_id } => {
                write!(
                    f,
                    "Deck '{}' references card_id '{}' which does not exist in any cards file",
                    deck_slug, card_id
                )
            }
            LoadError::InvalidSlug { slug, kind } => {
                write!(
                    f,
                    "Invalid {} slug '{}': must be lowercase with hyphens only",
                    kind, slug
                )
            }
        }
    }
}

fn validate_slug(slug: &str) -> bool {
    if slug.is_empty() {
        return false;
    }
    slug.chars().all(|c| c.is_ascii_lowercase() || c == '-')
}

fn load_json_files<T: for<'de> serde::Deserialize<'de>>(
    dir: &Path,
) -> Result<Vec<(String, T)>, LoadError> {
    let mut results = Vec::new();
    let entries = fs::read_dir(dir)
        .map_err(|e| LoadError::Io(format!("Cannot read directory {}: {}", dir.display(), e)))?;

    for entry in entries {
        let entry = entry.map_err(|e| LoadError::Io(e.to_string()))?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            let content = fs::read_to_string(&path)
                .map_err(|e| LoadError::Io(format!("Cannot read {}: {}", path.display(), e)))?;
            let parsed: T = serde_json::from_str(&content).map_err(|e| {
                LoadError::Io(format!("JSON parse error in {}: {}", path.display(), e))
            })?;
            let fname = path.file_name().unwrap().to_string_lossy().to_string();
            results.push((fname, parsed));
        }
    }

    Ok(results)
}

pub fn load_catalog(data_dir: &Path) -> Result<CardCatalog, LoadError> {
    let cards_dir = data_dir.join("cards");
    let decks_dir = data_dir.join("decks");
    let kanji_dir = data_dir.join("kanji");
    let radicals_path = data_dir.join("radicals").join("kangxi.json");

    // Load all cards
    let mut cards_map: HashMap<String, (String, Card)> = HashMap::new();
    let cards_files = load_json_files::<CardsFile>(&cards_dir)?;

    for (fname, cards_file) in &cards_files {
        for card in &cards_file.cards {
            if !validate_slug(&card.id) {
                return Err(LoadError::InvalidSlug {
                    slug: card.id.clone(),
                    kind: "card",
                });
            }
            if let Some((existing_file, _)) = cards_map.get(&card.id) {
                return Err(LoadError::DuplicateCardId {
                    id: card.id.clone(),
                    file1: existing_file.clone(),
                    file2: fname.clone(),
                });
            }
            cards_map.insert(card.id.clone(), (fname.clone(), card.clone()));
        }
    }

    let cards: HashMap<String, Card> = cards_map
        .into_values()
        .map(|(_, card)| (card.id.clone(), card))
        .collect();

    // Load all decks
    let mut decks_map: HashMap<String, (String, Deck)> = HashMap::new();
    let deck_files = load_json_files::<DeckFile>(&decks_dir)?;

    for (fname, deck_file) in &deck_files {
        let deck = Deck {
            slug: deck_file.deck.slug.clone(),
            name: deck_file.deck.name.clone(),
            description: deck_file.deck.description.clone(),
            cards: deck_file.cards.clone(),
        };

        if !validate_slug(&deck.slug) {
            return Err(LoadError::InvalidSlug {
                slug: deck.slug.clone(),
                kind: "deck",
            });
        }

        if let Some((existing_file, _)) = decks_map.get(&deck.slug) {
            return Err(LoadError::DuplicateDeckSlug {
                slug: deck.slug.clone(),
                file1: existing_file.clone(),
                file2: fname.clone(),
            });
        }

        // Validate all card references
        for link in &deck.cards {
            if !cards.contains_key(&link.card_id) {
                return Err(LoadError::UnresolvedCardRef {
                    deck_slug: deck.slug.clone(),
                    card_id: link.card_id.clone(),
                });
            }
        }

        decks_map.insert(deck.slug.clone(), (fname.clone(), deck));
    }

    let decks: HashMap<String, Deck> = decks_map
        .into_values()
        .map(|(_, deck)| (deck.slug.clone(), deck))
        .collect();

    // Load kanji from JSONL files
    let mut kanji: HashMap<String, KanjiEntry> = HashMap::new();
    if kanji_dir.exists() {
        let entries = fs::read_dir(&kanji_dir)
            .map_err(|e| LoadError::Io(format!("Cannot read kanji dir: {}", e)))?;
        for entry in entries {
            let entry = entry.map_err(|e| LoadError::Io(e.to_string()))?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                continue;
            }
            let content = fs::read_to_string(&path)
                .map_err(|e| LoadError::Io(format!("Cannot read {}: {}", path.display(), e)))?;
            for (line_num, line) in content.lines().enumerate() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let k: KanjiEntry = serde_json::from_str(line).map_err(|e| {
                    LoadError::Io(format!(
                        "JSONL parse error in {} line {}: {}",
                        path.display(),
                        line_num + 1,
                        e
                    ))
                })?;
                if kanji.contains_key(&k.char) {
                    return Err(LoadError::Io(format!(
                        "Duplicate kanji '{}' in JSONL files",
                        k.char
                    )));
                }
                kanji.insert(k.char.clone(), k);
            }
        }
    }

    // Load radicals
    let radicals: HashMap<u8, Radical> = if radicals_path.exists() {
        let content = fs::read_to_string(&radicals_path)
            .map_err(|e| LoadError::Io(format!("Cannot read kangxi.json: {}", e)))?;
        let list: Vec<Radical> = serde_json::from_str(&content)
            .map_err(|e| LoadError::Io(format!("Cannot parse kangxi.json: {}", e)))?;
        list.into_iter().map(|r| (r.number, r)).collect()
    } else {
        HashMap::new()
    };

    Ok(CardCatalog {
        cards,
        decks,
        kanji,
        radicals,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct HydratedDeckCard {
    pub card_id: String,
    pub kanji: String,
    pub romaji: String,
    pub fr: String,
    pub anime_reference: Option<String>,
    pub context_sentence: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HydratedDeck {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub cards: Vec<HydratedDeckCard>,
}

impl HydratedDeck {
    pub fn from_deck(deck: &Deck, catalog: &CardCatalog) -> Self {
        let cards = deck
            .cards
            .iter()
            .filter_map(|link| {
                catalog
                    .cards
                    .get(&link.card_id)
                    .map(|card| HydratedDeckCard {
                        card_id: link.card_id.clone(),
                        kanji: card.kanji.clone(),
                        romaji: card.romaji.clone(),
                        fr: card.fr.clone(),
                        anime_reference: link.anime_reference.clone(),
                        context_sentence: link.context_sentence.clone(),
                    })
            })
            .collect();

        HydratedDeck {
            slug: deck.slug.clone(),
            name: deck.name.clone(),
            description: deck.description.clone(),
            cards,
        }
    }
}
