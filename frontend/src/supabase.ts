import { createClient } from '@supabase/supabase-js'

const supabaseUrl = import.meta.env.VITE_SUPABASE_URL || 'https://xwydowdadjijeggpmztc.supabase.co'
const supabaseAnonKey = import.meta.env.VITE_SUPABASE_ANON_KEY || ''

if (!supabaseAnonKey || supabaseAnonKey === 'VOTRE_CLE_PUBLIQUE_ANON_ICI') {
  console.warn(
    "⚠️ Supabase Anon Key manquante dans le fichier frontend/.env. \n" +
    "Veuillez y renseigner votre VITE_SUPABASE_ANON_KEY pour que l'authentification fonctionne."
  )
}

export const supabase = createClient(supabaseUrl, supabaseAnonKey)
