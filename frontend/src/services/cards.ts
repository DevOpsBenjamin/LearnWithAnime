import { API_BASE, PresetItem } from './api'

export async function fetchCards(): Promise<{ presets: PresetItem[]; isSeeded: boolean }> {
  const response = await fetch(`${API_BASE}/cards`)
  if (!response.ok) throw new Error('Failed to fetch cards')
  const data = await response.json()
  const presets: PresetItem[] = data.map((c: any) => ({
    vocab: c.vocab,
    anime: c.anime_reference || 'Général',
    defaultAnswer: c.french_translation,
  }))
  return { presets, isSeeded: presets.length > 0 }
}

export async function seedDatabase(): Promise<void> {
  const response = await fetch(`${API_BASE}/db/seed`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
  })
  if (!response.ok) {
    throw new Error(`Erreur serveur (${response.status})`)
  }
}
