export const API_BASE =
  import.meta.env.VITE_API_URL || 'http://localhost:3000/api'

export interface PresetItem {
  vocab: string
  anime: string
  defaultAnswer: string
}

export interface EvalData {
  is_correct: boolean
  score: number
  explanation: string
  correction: string | null
}

export interface UserLlmSettings {
  user_id: string
  config_name: string
  api_url: string
  api_key: string
  model: string
  temperature_eval: number
  temperature_hint: number
  top_p: number
  frequency_penalty: number
  max_tokens: number
  is_active: boolean
}
