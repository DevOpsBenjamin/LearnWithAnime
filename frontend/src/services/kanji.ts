import { API_BASE } from './api'

export interface KanjiMeta {
  char: string
  jlpt_level: number
  radical_number: number | null
  stroke_count: number | null
  frequency_rank: number | null
}

export interface KanjiEntry {
  char: string
  on_yomi: string[]
  kun_yomi: string[]
  meanings_en: string[]
  components: string[]
  stroke_count: number | null
  frequency_rank: number | null
  jlpt_level: number
  grade: number | null
  radical_number: number | null
}

export interface Radical {
  number: number
  char: string
  meaning: string
  stroke_count: number
  variants: string[]
}

export async function fetchRadicals(): Promise<Radical[]> {
  const res = await fetch(`${API_BASE}/radicals`)
  if (!res.ok) throw new Error(`Erreur serveur (${res.status})`)
  return res.json()
}

export async function fetchKanjiList(): Promise<KanjiMeta[]> {
  const res = await fetch(`${API_BASE}/kanji`)
  if (!res.ok) throw new Error(`Erreur serveur (${res.status})`)
  return res.json()
}

export async function fetchKanjiDetail(char: string): Promise<KanjiEntry> {
  const res = await fetch(`${API_BASE}/kanji/${encodeURIComponent(char)}`)
  if (!res.ok) throw new Error(`Erreur serveur (${res.status})`)
  return res.json()
}
