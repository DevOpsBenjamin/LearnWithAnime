import { API_BASE, PresetItem } from './api'

export interface DeckMeta {
  slug: string
  name: string
  description: string | null
  card_count: number
}

export interface HydratedCard {
  card_id: string
  kanji: string
  romaji: string
  fr: string
  anime_reference: string | null
  context_sentence: string | null
}

export interface HydratedDeck {
  slug: string
  name: string
  description: string | null
  cards: HydratedCard[]
}

export async function fetchDecks(): Promise<DeckMeta[]> {
  const res = await fetch(`${API_BASE}/decks`)
  if (!res.ok) throw new Error(`Erreur serveur (${res.status})`)
  return res.json()
}

export async function fetchDeckBySlug(slug: string): Promise<HydratedDeck> {
  const res = await fetch(`${API_BASE}/decks/${slug}`)
  if (!res.ok) throw new Error(`Erreur serveur (${res.status})`)
  return res.json()
}

export function deckCardsAsPresets(deck: HydratedDeck): PresetItem[] {
  return deck.cards.map((c) => ({
    vocab: c.kanji,
    anime: c.anime_reference || deck.name,
    defaultAnswer: c.fr,
  }))
}
