import { API_BASE, EvalData } from './api'

interface HintParams {
  vocab: string
  tier: number
  model: string | null
  temperature: number
  api_url: string | null
  api_key: string | null
  top_p: number
  max_tokens: number
  frequency_penalty: number
}

interface EvalParams {
  vocab: string
  user_answer: string
  context: string | null
  model: string | null
  temperature: number
  api_url: string | null
  api_key: string | null
  top_p: number
  max_tokens: number
  frequency_penalty: number
}

export async function fetchModels(
  apiUrl: string | null,
  apiKey: string | null,
): Promise<string[]> {
  const response = await fetch(`${API_BASE}/ai/models`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ api_url: apiUrl, api_key: apiKey }),
  })
  if (!response.ok) throw new Error(`Serveur injoignable (${response.status})`)
  return response.json()
}

export async function testLlmConnection(
  apiUrl: string,
  apiKey: string,
): Promise<{ models: string[] }> {
  const response = await fetch(`${API_BASE}/ai/models`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ api_url: apiUrl || null, api_key: apiKey || null }),
  })
  if (!response.ok) {
    const text = await response.text()
    throw new Error(text || `Erreur serveur (${response.status})`)
  }
  const models: string[] = await response.json()
  return { models }
}

export async function fetchHint(params: HintParams): Promise<string> {
  const response = await fetch(`${API_BASE}/ai/hint`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(params),
  })
  if (!response.ok) throw new Error(`Erreur serveur (${response.status})`)
  const data = await response.json()
  return data.hint
}

export async function evaluateAnswer(params: EvalParams): Promise<EvalData> {
  const response = await fetch(`${API_BASE}/ai/evaluate`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(params),
  })
  if (!response.ok) {
    const text = await response.text()
    throw new Error(text || `Erreur serveur (${response.status})`)
  }
  return response.json()
}
