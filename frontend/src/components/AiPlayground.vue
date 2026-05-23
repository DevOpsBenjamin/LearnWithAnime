<template>
  <div class="playground-container">
    <HeaderSection
      :user="user"
      :avatarUrl="avatarUrl"
      :userInitial="userInitial"
      :userDisplayName="userDisplayName"
      :hasActiveConfig="hasActiveConfig"
      :userConfigs="userConfigs"
      :activeConfigName="activeConfigName"
      :currentView="currentView"
      :isAdmin="isAdmin"
      @update:activeConfigName="activeConfigName = $event"
      @activate-config="activateConfig"
      @go-to-settings="currentView = 'settings'"
      @go-to-playground="currentView = 'playground'"
      @go-to-admin="currentView = 'admin'"
      @sign-out="handleSignOut"
    />

    <div v-if="currentView === 'playground'" class="playground-grid">
      <!-- Section Gauche : Sélection & Indices -->
      <section class="card-section">
        <!-- Panel 1 : Défi -->
        <div class="glass-panel">
          <div class="panel-header">
            <span class="step-num">01</span>
            <h2>Choisissez un défi japonais</h2>
          </div>

          <!-- Presets -->
          <div class="preset-selector">
            <div class="preset-header-inline">
              <label class="section-label">Sélection depuis la base de données :</label>
              <button 
                v-if="!isSeeded" 
                @click="seedDatabase" 
                class="seed-btn" 
                :disabled="loadingCards"
                title="Initialiser la base PostgreSQL/Supabase avec nos exemples cultes"
              >
                🌱 Remplir la BDD
              </button>
            </div>

            <div v-if="loadingCards" class="presets-loader">
              <span class="spinner"></span> <span>Chargement des cartes depuis la base...</span>
            </div>
            
            <div v-else-if="presets.length === 0" class="empty-presets-alert">
              <p>La base de données est connectée mais vide.</p>
              <button @click="seedDatabase" class="btn btn-secondary btn-sm" style="margin-top: 10px;">
                🌱 Charger le deck 'Animés Légendaires'
              </button>
            </div>

            <div v-else class="presets-grid">
              <button 
                v-for="item in presets" 
                :key="item.vocab"
                @click="selectPreset(item)"
                :class="['preset-btn', { active: currentVocab === item.vocab && !isCustom }]"
              >
                <div class="preset-vocab">{{ item.vocab }}</div>
                <div class="preset-meta">{{ item.anime }}</div>
              </button>
            </div>
          </div>

          <!-- Custom Input Toggle -->
          <div class="custom-toggle-container">
            <button 
              @click="toggleCustom" 
              :class="['custom-toggle-btn', { active: isCustom }]"
            >
              ⌨️ Saisir mon propre mot / expression
            </button>
          </div>

          <!-- Custom Input Fields -->
          <Transition name="expand">
            <div v-if="isCustom" class="custom-inputs">
              <div class="input-group">
                <label>Expression / Mot Japonais</label>
                <input 
                  type="text" 
                  v-model="customVocab" 
                  placeholder="Ex: 螺旋丸 ou 諦める..." 
                  class="glass-input"
                />
              </div>
              <div class="input-group">
                <label>Manga de référence (facultatif)</label>
                <input 
                  type="text" 
                  v-model="customAnime" 
                  placeholder="Ex: Naruto, Shingeki no Kyojin..." 
                  class="glass-input"
                />
              </div>
            </div>
          </Transition>

          <!-- Divider -->
          <div class="divider"></div>

          <!-- Hints System -->
          <div class="hints-system">
            <label class="section-label">Besoin d'aide ? Demandez des indices progressifs :</label>
            <div class="hints-buttons">
              <button 
                @click="fetchHint(1)" 
                :disabled="loadingHint1 || !activeVocab"
                class="btn btn-secondary"
              >
                <span v-if="loadingHint1" class="spinner"></span>
                <span v-else>💡 Indice Tier 1 (Lecture)</span>
              </button>
              
              <button 
                @click="fetchHint(2)" 
                :disabled="loadingHint2 || !activeVocab"
                class="btn btn-secondary"
              >
                <span v-if="loadingHint2" class="spinner"></span>
                <span v-else>🎭 Indice Tier 2 (Contexte)</span>
              </button>
            </div>

            <!-- Hints Display -->
            <Transition name="fade">
              <div v-if="hint1" class="hint-bubble tier1">
                <div class="bubble-header">🟢 Indice de lecture / traits :</div>
                <p>{{ hint1 }}</p>
              </div>
            </Transition>
            
            <Transition name="fade">
              <div v-if="hint2" class="hint-bubble tier2">
                <div class="bubble-header">🟡 Phrase d'exemple en contexte :</div>
                <p class="japanese-font">{{ hint2 }}</p>
              </div>
            </Transition>
          </div>
        </div>
      </section>

      <!-- Section Droite : Réponse & Évaluation -->
      <section class="answer-section">
        <div class="glass-panel">
          <div class="panel-header">
            <span class="step-num">02</span>
            <h2>Saisissez votre traduction</h2>
          </div>

          <div class="active-card-display">
            <span class="card-sub">Expression ciblée :</span>
            <div class="card-main-vocab japanese-font">{{ activeVocab || 'Aucune sélection' }}</div>
            <div v-if="activeAnime" class="card-anime-ref">🎬 Manga : {{ activeAnime }}</div>
          </div>

          <div class="input-group">
            <label for="user-answer">Votre traduction ou explication en français :</label>
            <textarea 
              id="user-answer"
              v-model="userAnswer"
              placeholder="Ex: renoncer, abandonner... ou donnez une phrase explicative"
              rows="3"
              class="glass-textarea"
              :disabled="!activeVocab"
            ></textarea>
          </div>

          <button 
            @click="evaluateAnswer" 
            :disabled="loadingEval || !userAnswer.trim() || !activeVocab"
            class="btn btn-primary btn-block btn-eval"
          >
            <div v-if="loadingEval" class="eval-loading-wrapper">
              <span class="pulsing-ring"></span>
              <span>Analyse IA en cours...</span>
            </div>
            <span v-else>🌸 Évaluer avec l'IA locale</span>
          </button>

          <!-- Error Alert -->
          <div v-if="errorMessage" class="error-alert">
            <span class="alert-icon">⚠️</span>
            <p>{{ errorMessage }}</p>
          </div>

          <!-- Evaluation Results -->
          <Transition name="slide-up">
            <div v-if="evaluation" class="evaluation-result">
              <div class="divider"></div>
              
              <div class="result-header">
                <h3>Résultat de l'évaluation</h3>
                <div :class="['score-badge', getScoreClass(evaluation.score)]">
                  <div class="score-num">{{ evaluation.score }}</div>
                  <div class="score-label">/100</div>
                </div>
              </div>

              <!-- Status Banner -->
              <div :class="['status-banner', evaluation.is_correct ? 'success' : 'failure']">
                <span class="status-icon">{{ evaluation.is_correct ? '✅ Réponse Validée !' : '❌ À corriger' }}</span>
              </div>

              <!-- Explanation -->
              <div class="result-body">
                <div class="result-block">
                  <h4>💡 Analyse du tuteur :</h4>
                  <p class="explanation-text">{{ evaluation.explanation }}</p>
                </div>

                <!-- Correction -->
                <div v-if="evaluation.correction" class="result-block correction-block">
                  <h4>✍️ Correction suggérée :</h4>
                  <p class="correction-text japanese-font">{{ evaluation.correction }}</p>
                </div>
              </div>
            </div>
          </Transition>
        </div>
      </section>
    </div>

    <!-- Vue Paramètres (Page dédiée) -->
    <div v-else-if="currentView === 'settings'" class="settings-grid">
      <!-- Encart 1 : Connexion au LLM -->
      <div class="glass-panel settings-card">
        <div class="panel-header">
          <span class="step-num">01</span>
          <h2>Connexion au service d'IA</h2>
        </div>

        <div class="input-group">
          <label for="api-url-input">URL du point d'accès API (Endpoint) :</label>
          <input 
            id="api-url-input" 
            type="text" 
            v-model="apiUrl" 
            placeholder="Ex: http://localhost:1337/v1" 
            class="glass-input"
          />
          <!-- Presets Row -->
          <div class="presets-row">
            <button type="button" @click="apiUrl = 'http://localhost:11434/v1'" class="preset-badge">🦙 Ollama (Local)</button>
            <button type="button" @click="apiUrl = 'https://api.openai.com/v1'" class="preset-badge">⚡ OpenAI</button>
            <button type="button" @click="apiUrl = 'https://api.x.ai/v1'" class="preset-badge">🤖 xAI (Grok)</button>
            <button type="button" @click="apiUrl = 'https://openrouter.ai/api/v1'" class="preset-badge">🔮 OpenRouter</button>
            <button type="button" @click="apiUrl = 'https://api.mistral.ai/v1'" class="preset-badge">🌀 Mistral AI</button>
            <button type="button" @click="apiUrl = 'https://api.groq.com/openai/v1'" class="preset-badge">⚡ Groq</button>
          </div>
        </div>

        <div class="input-group" style="margin-top: 20px;">
          <label for="api-key-input">Clé API secrète (Optionnelle) :</label>
          <input 
            id="api-key-input" 
            type="password" 
            v-model="apiKey" 
            placeholder="Ex: sk-... (laissez vide pour un LLM local)" 
            class="glass-input"
          />
        </div>

        <button 
          @click="testLlmConnection" 
          :disabled="loadingModels" 
          class="btn btn-primary btn-block btn-connect"
          style="margin-top: 25px;"
        >
          <span v-if="loadingModels" class="spinner"></span>
          <span v-else>🔌 Se connecter &amp; Tester l'API</span>
        </button>

        <!-- Error Alert if failed connection -->
        <Transition name="fade">
          <div v-if="connectionError" class="error-alert" style="margin-top: 20px;">
            <span class="alert-icon">⚠️</span>
            <div>
              <strong>Échec de connexion :</strong> {{ connectionError }}
              <p style="margin: 5px 0 0 0; font-size: 0.8rem; opacity: 0.85;">Vérifiez l'URL de votre API et que votre serveur local ou cloud est en ligne.</p>
            </div>
          </div>
        </Transition>
      </div>

      <!-- Encart 2 : Configuration du modèle (Affiché uniquement si connecté) -->
      <Transition name="slide-up">
        <div v-if="isConnectionVerified" class="glass-panel settings-card success-border">
          <div class="panel-header">
            <span class="step-num">02</span>
            <h2>Configuration du modèle</h2>
            <span v-if="isEditing" class="edit-badge">✏️ Édition de « {{ editingConfigName }} »</span>
            <span v-else class="edit-badge create-badge">🆕 Nouvelle configuration</span>
          </div>

          <!-- Succès -->
          <div class="status-banner success" style="margin-bottom: 20px;">
            <span>✅ Connexion établie avec succès !</span>
          </div>

          <div class="input-group">
            <label for="model-select">Modèle LLM ciblé :</label>
            <select 
              id="model-select" 
              v-model="selectedModel" 
              class="glass-select"
            >
              <option value="">-- Choisir un modèle détecté --</option>
              <option v-for="mod in models" :key="mod" :value="mod">
                {{ mod.replace('mlx/', '') }}
              </option>
            </select>
          </div>

          <!-- Nom de la configuration -->
          <div class="input-group" style="margin-top: 20px;">
            <label for="config-name-input">Nom de cette configuration d'API :</label>
            <input 
              id="config-name-input" 
              type="text" 
              v-model="configNameInput" 
              placeholder="ex: Ollama Local, OpenAI Pro, Groq Fast..." 
              class="glass-input"
            />
          </div>

          <div class="save-config-row" style="margin-top: 25px; display: flex; gap: 10px;">
            <button 
              v-if="isEditing"
              @click="cancelEditing" 
              class="btn btn-secondary"
              style="flex: 0 0 auto;"
            >
              ✖ Annuler
            </button>
            <button 
              @click="saveUserSettings" 
              class="btn btn-primary btn-block btn-save"
              style="flex: 1;"
            >
              {{ isEditing ? '💾 Mettre à jour la configuration' : '💾 Créer la configuration' }}
            </button>
          </div>

          <!-- Liste des profils sauvegardés -->
          <div v-if="userConfigs.length > 0" class="input-group" style="margin-top: 25px;">
            <label style="opacity: 0.85;">Profils IA enregistrés :</label>
            <div class="saved-configs-list">
              <div v-for="cfg in userConfigs" :key="cfg.config_name" class="config-list-item" :class="{ 'active-item': cfg.is_active }">
                <span class="config-item-name" @click="activeConfigName = cfg.config_name; activateConfig(cfg.config_name)">
                  {{ cfg.is_active ? '✅' : '🤖' }} <strong>{{ cfg.config_name }}</strong>
                </span>
                <div class="config-item-actions">
                  <button 
                    @click="loadConfigForEditing(cfg)" 
                    class="edit-config-btn" 
                    title="Modifier ce profil"
                  >
                    ✏️
                  </button>
                  <button 
                    v-if="userConfigs.length > 1" 
                    @click="deleteConfig(cfg.config_name)" 
                    class="delete-config-btn" 
                    title="Supprimer ce profil"
                  >
                    🗑️
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </Transition>

      <!-- Encart 3 : Paramètres Avancés (Affiché si connecté) -->
      <Transition name="slide-up">
        <div v-if="isConnectionVerified" class="glass-panel settings-card" style="grid-column: span 2; width: 100%; margin-top: 20px;">
          <div class="panel-header">
            <span class="step-num">03</span>
            <h2>Paramètres avancés de l'IA</h2>
          </div>

          <!-- Avertissement de Sécurité -->
          <div class="warning-alert-box" style="margin-bottom: 25px;">
            <span class="alert-icon">⚠️</span>
            <p class="warning-text">
              <strong>Attention :</strong> Des valeurs par défaut existent. Seuls ceux qui savent quoi faire de ces options et pourquoi leur modèle a besoin de les changer devraient y toucher.
            </p>
          </div>

          <div class="advanced-settings-grid">
            <!-- Double Contrôle des Températures -->
            <div class="input-group">
              <div class="slider-label-row">
                <label for="temp-eval-slider">Température d'Évaluation (JSON Strict) :</label>
                <span class="temp-value">{{ temperatureEval }} ({{ getTempLabel(temperatureEval) }})</span>
              </div>
              <input 
                id="temp-eval-slider" 
                type="range" 
                v-model.number="temperatureEval" 
                min="0.0" 
                max="1.0" 
                step="0.1" 
                class="glass-slider"
              />
              <p class="slider-desc">
                Recommandé : 0.0 - 0.1. Maintient le tuteur logique et évite les erreurs de format JSON.
              </p>
            </div>

            <div class="input-group">
              <div class="slider-label-row">
                <label for="temp-hint-slider">Température des Indices (Créativité) :</label>
                <span class="temp-value">{{ temperatureHint }} ({{ getTempLabel(temperatureHint) }})</span>
              </div>
              <input 
                id="temp-hint-slider" 
                type="range" 
                v-model.number="temperatureHint" 
                min="0.0" 
                max="1.0" 
                step="0.1" 
                class="glass-slider"
              />
              <p class="slider-desc">
                Recommandé : 0.6 - 0.8. Favorise la variété et l'imagination pour les phrases d'exemples.
              </p>
            </div>

            <!-- Top P Slider -->
            <div class="input-group" style="margin-top: 20px;">
              <div class="slider-label-row">
                <label for="top-p-slider">Top P (Nucleus Sampling) :</label>
                <span class="temp-value">{{ topP }}</span>
              </div>
              <input 
                id="top-p-slider" 
                type="range" 
                v-model.number="topP" 
                min="0.0" 
                max="1.0" 
                step="0.05" 
                class="glass-slider"
              />
              <p class="slider-desc">
                Recommandé : 1.0 (neutre) ou 0.9. Contrôle la diversité des jetons retenus.
              </p>
            </div>

            <!-- Frequency Penalty Slider -->
            <div class="input-group" style="margin-top: 20px;">
              <div class="slider-label-row">
                <label for="freq-penalty-slider">Pénalité de Répétition (Frequency Penalty) :</label>
                <span class="temp-value">{{ frequencyPenalty }}</span>
              </div>
              <input 
                id="freq-penalty-slider" 
                type="range" 
                v-model.number="frequencyPenalty" 
                min="-2.0" 
                max="2.0" 
                step="0.1" 
                class="glass-slider"
              />
              <p class="slider-desc">
                Recommandé : 0.0 (neutre) à 1.0. Évite les boucles de répétitions de mots.
              </p>
            </div>

            <!-- Max Tokens Input -->
            <div class="input-group" style="margin-top: 20px; grid-column: span 2;">
              <label for="max-tokens-input">Nombre maximal de jetons générés (Max Tokens) :</label>
              <input 
                id="max-tokens-input" 
                type="number" 
                v-model.number="maxTokens" 
                min="1" 
                max="8192" 
                class="glass-input"
              />
              <p class="slider-desc">
                Recommandé : 2048. Limite supérieure de la taille de la réponse du modèle.
              </p>
            </div>
          </div>

          <button 
            @click="saveUserSettings" 
            class="btn btn-primary btn-block btn-save"
            style="margin-top: 25px;"
          >
            💾 Enregistrer les paramètres avancés
          </button>
        </div>
      </Transition>
    </div>

    <!-- Vue Admin -->
    <div v-else-if="currentView === 'admin'" class="playground-grid" style="display: flex; justify-content: center; padding-top: 20px;">
      <AdminPanel
        :user="user"
        @admin-updated="checkAdminStatus"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { supabase } from '../supabase'
import HeaderSection from './HeaderSection.vue'
import AdminPanel from './AdminPanel.vue'

interface UserLlmSettings {
  user_id: string;
  config_name: string;
  api_url: string;
  api_key: string;
  model: string;
  temperature_eval: number;
  temperature_hint: number;
  top_p: number;
  frequency_penalty: number;
  max_tokens: number;
  is_active: boolean;
}

const userConfigs = ref<UserLlmSettings[]>([])
const activeConfigName = ref<string>('')
const configNameInput = ref<string>('Défaut')
const editingConfigName = ref<string | null>(null)
const hasActiveConfig = ref<boolean>(false)
const isEditing = computed(() => editingConfigName.value !== null)

const handleSignOut = async () => {
  await supabase.auth.signOut()
}

const checkAdminStatus = async () => {
  if (!user.value) return
  try {
    const res = await fetch(`${API_BASE}/admin/admins`)
    if (!res.ok) return
    const admins = await res.json()
    const userEmail = user.value.email
    const userId = user.value.id
    isAdmin.value = admins.some(
      (a: any) => (a.user_id === userId) || (a.email === userEmail)
    )
  } catch {
    isAdmin.value = false
  }
}

interface PresetItem {
  vocab: string;
  anime: string;
  defaultAnswer: string;
}

// État dynamique pour les presets (cartes lues en base de données)
const presets = ref<PresetItem[]>([])
const loadingCards = ref(false)
const isSeeded = ref(false)

// Profil Utilisateur
const user = ref<any>(null)
const showSettings = ref(false)
const temperatureEval = ref(0.1) // Température strict pour l'évaluation JSON
const temperatureHint = ref(0.7) // Température créative pour les indices
const topP = ref(1.0)
const frequencyPenalty = ref(0.0)
const maxTokens = ref(2048)
const apiUrl = ref('http://localhost:1337/v1')
const apiKey = ref('')

// Navigation et validation d'API
const currentView = ref<'playground' | 'settings' | 'admin'>('playground')
const isAdmin = ref(false)
const isConnectionVerified = ref(false)
const connectionError = ref('')
const isLoadingSettings = ref(false) // Guard pour éviter que le watcher ne reset la connexion pendant le chargement

// Reset de la validation en cas de modification de l'URL/Clé (sauf pendant le chargement initial)
watch([apiUrl, apiKey], () => {
  if (isLoadingSettings.value) return // Ne pas reset pendant fetchUserSettings
  isConnectionVerified.value = false
  connectionError.value = ''
})

const userDisplayName = computed(() => {
  if (!user.value) return 'Utilisateur'
  return (
    user.value.user_metadata?.username ||
    user.value.user_metadata?.full_name ||
    user.value.email?.split('@')[0] ||
    'Apprenant'
  )
})

const avatarUrl = computed(() => {
  return user.value?.user_metadata?.avatar_url || null
})

const userInitial = computed(() => {
  const name = userDisplayName.value
  return name ? name.charAt(0).toUpperCase() : 'U'
})

const getTempLabel = (temp: number) => {
  if (temp <= 0.2) return 'Strict & Précis'
  if (temp <= 0.5) return 'Équilibré'
  if (temp <= 0.8) return 'Créatif'
  return 'Très Imaginatif'
}

// Sélection
const currentVocab = ref('')
const currentAnime = ref('')
const isCustom = ref(false)

const customVocab = ref('')
const customAnime = ref('')

const activeVocab = computed(() => {
  return isCustom.value ? customVocab.value : currentVocab.value
})

const activeAnime = computed(() => {
  return isCustom.value ? customAnime.value : currentAnime.value
})

// Vider les réponses et indices en cas de changement de mot
watch(activeVocab, () => {
  hint1.value = ''
  hint2.value = ''
  userAnswer.value = ''
  evaluation.value = null
  errorMessage.value = ''
})

const selectPreset = (item: PresetItem) => {
  isCustom.value = false
  currentVocab.value = item.vocab
  currentAnime.value = item.anime
}

const toggleCustom = () => {
  isCustom.value = !isCustom.value
}

// État des API
const userAnswer = ref('')
const loadingHint1 = ref(false)
const loadingHint2 = ref(false)
const loadingEval = ref(false)

const hint1 = ref('')
const hint2 = ref('')
const errorMessage = ref('')

// Configuration IA Dynamique
const models = ref<string[]>([])
const selectedModel = ref<string>('')
const loadingModels = ref(false)
const loadingModelsError = ref('')

interface EvalData {
  is_correct: boolean;
  score: number;
  explanation: string;
  correction: string | null;
}
const evaluation = ref<EvalData | null>(null)

// API Host (Backend local Rust)
const API_BASE = 'http://localhost:3000/api'

// Récupération des modèles (utilisé en arrière-plan)
const fetchModels = async () => {
  loadingModels.value = true
  loadingModelsError.value = ''
  try {
    const response = await fetch(`${API_BASE}/ai/models`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        api_url: apiUrl.value || null,
        api_key: apiKey.value || null
      })
    })
    if (!response.ok) {
      throw new Error(`Serveur injoignable (${response.status})`)
    }
    const data: string[] = await response.json()
    models.value = data
  } catch (err: any) {
    loadingModelsError.value = `Impossible de récupérer les modèles.`
    console.error(err)
  } finally {
    loadingModels.value = false
  }
}

// Validation de connexion et récupération des modèles (Encart 1)
const testLlmConnection = async () => {
  loadingModels.value = true
  connectionError.value = ''
  isConnectionVerified.value = false
  
  try {
    const response = await fetch(`${API_BASE}/ai/models`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        api_url: apiUrl.value || null,
        api_key: apiKey.value || null
      })
    })
    
    if (!response.ok) {
      const text = await response.text()
      throw new Error(text || `Erreur serveur (${response.status})`)
    }
    
    const data: string[] = await response.json()
    models.value = data
    
    // Essaye de présélectionner le modèle MiniMax
    if (!selectedModel.value || !data.includes(selectedModel.value)) {
      const defaultModel = data.find((m: string) => m.toLowerCase().includes('minimax'))
      if (defaultModel) {
        selectedModel.value = defaultModel
      } else if (data.length > 0) {
        selectedModel.value = data[0]
      }
    }
    
    isConnectionVerified.value = true
  } catch (err: any) {
    connectionError.value = err.message || "Impossible de contacter l'API."
    isConnectionVerified.value = false
  } finally {
    loadingModels.value = false
  }
}

// Récupération et sauvegarde des configurations utilisateur dans Supabase (via notre backend Rust)
const fetchUserSettings = async (userId: string) => {
  isLoadingSettings.value = true
  try {
    // 1. Récupération de la configuration active
    const response = await fetch(`${API_BASE}/user/llm-settings/${userId}`)
    if (response.ok) {
      const data: UserLlmSettings = await response.json()
      apiUrl.value = data.api_url
      apiKey.value = data.api_key || ''
      selectedModel.value = data.model
      temperatureEval.value = data.temperature_eval
      temperatureHint.value = data.temperature_hint
      topP.value = data.top_p
      frequencyPenalty.value = data.frequency_penalty
      maxTokens.value = data.max_tokens
      activeConfigName.value = data.config_name
      configNameInput.value = data.config_name
      hasActiveConfig.value = true
      isConnectionVerified.value = true // La configuration active est déjà validée
    } else if (response.status === 404) {
      hasActiveConfig.value = false
      // S'il n'y a aucun profil, on reset l'état de connexion vérifiée
      isConnectionVerified.value = false
    }

    // 2. Récupération de l'intégralité des profils IA
    const allResponse = await fetch(`${API_BASE}/user/llm-settings/${userId}/all`)
    if (allResponse.ok) {
      userConfigs.value = await allResponse.json()
    }
  } catch (err) {
    console.error("Erreur lors de la récupération des paramètres utilisateur:", err)
    hasActiveConfig.value = false
  } finally {
    isLoadingSettings.value = false
  }
}

const saveUserSettings = async () => {
  if (!user.value) return
  
  try {
    const response = await fetch(`${API_BASE}/user/llm-settings`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        user_id: user.value.id,
        config_name: configNameInput.value.trim() || 'Défaut',
        api_url: apiUrl.value,
        api_key: apiKey.value || null,
        model: selectedModel.value,
        temperature_eval: temperatureEval.value,
        temperature_hint: temperatureHint.value,
        top_p: topP.value,
        frequency_penalty: frequencyPenalty.value,
        max_tokens: maxTokens.value,
        is_active: true
      })
    })
    
    if (!response.ok) {
      throw new Error(`Erreur serveur (${response.status})`)
    }
    
    // Recharge les paramètres
    editingConfigName.value = null
    await fetchUserSettings(user.value.id)
    currentView.value = 'playground'
  } catch (err: any) {
    alert(`Impossible d'enregistrer les paramètres : ${err.message}`)
  }
}

const activateConfig = async (configName: string) => {
  if (!user.value) return
  editingConfigName.value = null
  try {
    const response = await fetch(`${API_BASE}/user/llm-settings/activate`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        user_id: user.value.id,
        config_name: configName
      })
    })
    if (!response.ok) {
      throw new Error(`Code ${response.status}`)
    }
    // Recharge
    await fetchUserSettings(user.value.id)
  } catch (err: any) {
    alert(`Impossible d'activer la configuration : ${err.message}`)
  }
}

const deleteConfig = async (configName: string) => {
  if (!user.value) return
  if (!confirm(`Voulez-vous vraiment supprimer la configuration "${configName}" ?`)) return
  
  try {
    const response = await fetch(`${API_BASE}/user/llm-settings/${user.value.id}/${encodeURIComponent(configName)}`, {
      method: 'DELETE'
    })
    if (!response.ok) {
      throw new Error(`Code ${response.status}`)
    }
    // Si on était en train d'éditer la config supprimée, annuler l'édition
    if (editingConfigName.value === configName) {
      editingConfigName.value = null
    }
    // Recharge
    await fetchUserSettings(user.value.id)
  } catch (err: any) {
    alert(`Impossible de supprimer la configuration : ${err.message}`)
  }
}

const loadConfigForEditing = (cfg: UserLlmSettings) => {
  apiUrl.value = cfg.api_url
  apiKey.value = cfg.api_key || ''
  selectedModel.value = cfg.model
  configNameInput.value = cfg.config_name
  temperatureEval.value = cfg.temperature_eval
  temperatureHint.value = cfg.temperature_hint
  topP.value = cfg.top_p
  frequencyPenalty.value = cfg.frequency_penalty
  maxTokens.value = cfg.max_tokens
  editingConfigName.value = cfg.config_name
  if (models.value.length === 0) {
    testLlmConnection()
  }
}

const cancelEditing = () => {
  editingConfigName.value = null
  // Recharger la config active dans le formulaire
  if (user.value) {
    fetchUserSettings(user.value.id)
  }
}

// Récupération des cartes de vocabulaire en BDD
const fetchCards = async () => {
  loadingCards.value = true
  try {
    const response = await fetch(`${API_BASE}/cards`)
    if (!response.ok) throw new Error()
    const data = await response.json()
    
    presets.value = data.map((c: any) => ({
      vocab: c.vocab,
      anime: c.anime_reference || 'Général',
      defaultAnswer: c.french_translation
    }))
    
    if (presets.value.length > 0) {
      isSeeded.value = true
      // Sélectionne le premier mot par défaut si rien n'est sélectionné
      if (!currentVocab.value) {
        currentVocab.value = presets.value[0].vocab
        currentAnime.value = presets.value[0].anime
      }
    } else {
      isSeeded.value = false
    }
  } catch (err) {
    console.warn("Échec du chargement des cartes de la base de données.")
    isSeeded.value = false
  } finally {
    loadingCards.value = false
  }
}

// Remplir / Seeder la base de données
const seedDatabase = async () => {
  loadingCards.value = true
  errorMessage.value = ''
  try {
    const response = await fetch(`${API_BASE}/db/seed`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' }
    })
    if (!response.ok) {
      throw new Error(`Erreur serveur (${response.status})`)
    }
    await fetchCards()
  } catch (err: any) {
    errorMessage.value = `Impossible d'initialiser la base de données : ${err.message}. \nVeuillez configurer DATABASE_URL dans votre backend/.env avec vos identifiants Supabase.`
  } finally {
    loadingCards.value = false
  }
}

onMounted(() => {
  fetchCards()
  
  // Récupérer les données de l'utilisateur connecté et charger ses paramètres
  supabase.auth.getUser().then(async ({ data }) => {
    if (data.user) {
      user.value = data.user
      await fetchUserSettings(data.user.id)
      await checkAdminStatus()
      
      // Auto-vérifier la connexion au montage uniquement si pas déjà vérifié par fetchUserSettings
      if (apiUrl.value && !isConnectionVerified.value) {
        await testLlmConnection()
      }
    }
  })
})

const fetchHint = async (tier: number) => {
  if (!activeVocab.value) return
  
  if (tier === 1) {
    loadingHint1.value = true
    hint1.value = ''
  } else {
    loadingHint2.value = true
    hint2.value = ''
  }
  errorMessage.value = ''

  try {
    const response = await fetch(`${API_BASE}/ai/hint`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        vocab: activeVocab.value,
        tier,
        model: selectedModel.value || null,
        temperature: temperatureHint.value,
        api_url: apiUrl.value || null,
        api_key: apiKey.value || null,
        top_p: topP.value,
        max_tokens: maxTokens.value,
        frequency_penalty: frequencyPenalty.value
      })
    })

    if (!response.ok) {
      throw new Error(`Erreur serveur (${response.status})`)
    }

    const data = await response.json()
    if (tier === 1) {
      hint1.value = data.hint
    } else {
      hint2.value = data.hint
    }
  } catch (err: any) {
    errorMessage.value = `Impossible d'obtenir l'indice : ${err.message}. Assurez-vous que le backend Rust (port 3000) et votre LLM local/distant sont configurés et démarrés.`
  } finally {
    loadingHint1.value = false
    loadingHint2.value = false
  }
}

const evaluateAnswer = async () => {
  if (!activeVocab.value || !userAnswer.value.trim()) return
  
  loadingEval.value = true
  evaluation.value = null
  errorMessage.value = ''

  try {
    const response = await fetch(`${API_BASE}/ai/evaluate`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        vocab: activeVocab.value,
        user_answer: userAnswer.value,
        context: activeAnime.value ? `Manga de référence: ${activeAnime.value}` : null,
        model: selectedModel.value || null,
        temperature: temperatureEval.value,
        api_url: apiUrl.value || null,
        api_key: apiKey.value || null,
        top_p: topP.value,
        max_tokens: maxTokens.value,
        frequency_penalty: frequencyPenalty.value
      })
    })

    if (!response.ok) {
      const errText = await response.text()
      throw new Error(errText || `Erreur serveur (${response.status})`)
    }

    const data = await response.json()
    evaluation.value = data
  } catch (err: any) {
    errorMessage.value = `Échec de l'évaluation : ${err.message}. \n\nNote : Votre LLM doit renvoyer un format JSON strict. Vérifiez vos paramètres d'API et la disponibilité du modèle.`
  } finally {
    loadingEval.value = false
  }
}

const getScoreClass = (score: number) => {
  if (score >= 80) return 'score-success'
  if (score >= 50) return 'score-warning'
  return 'score-danger'
}
</script>

<style scoped>
.playground-container {
  max-width: var(--content-max-width);
  margin: 0 auto;
  padding: 40px 20px;
}

.subtitle {
  color: var(--text-muted);
  font-size: 1.1rem;
  max-width: 700px;
  margin: 20px auto 0 auto;
  text-align: center;
  line-height: 1.6;
}

.model-name {
  color: var(--primary);
  font-weight: 600;
}

/* Grid Layout */
.playground-grid {
  display: grid;
  grid-template-columns: 1fr 1.1fr;
  gap: 30px;
  align-items: start;
}

@media (max-width: 900px) {
  .playground-grid {
    grid-template-columns: 1fr;
  }
}

/* Glassmorphism Panel */
.glass-panel {
  background: var(--glass-bg);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border: 1px solid var(--glass-border);
  border-radius: 24px;
  padding: 30px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  margin-bottom: 25px;
}

.glass-panel:hover {
  border-color: rgba(139, 92, 246, 0.2);
  box-shadow: 0 15px 40px rgba(139, 92, 246, 0.15);
}

.panel-header {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 25px;
}

.step-num {
  font-size: 1.1rem;
  font-weight: 800;
  color: var(--primary);
  background: rgba(236, 72, 153, 0.1);
  width: 32px;
  height: 32px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid rgba(236, 72, 153, 0.25);
}

.panel-header h2 {
  font-size: 1.4rem;
  margin: 0;
  font-weight: 700;
  color: var(--text-main);
}

.section-label {
  display: block;
  font-size: 0.95rem;
  font-weight: 500;
  color: var(--text-muted);
  margin: 0;
}

/* Header Actions & Profile */


/* Modal Styling */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
  box-sizing: border-box;
}

.modal-card {
  width: 100%;
  max-width: 480px;
  padding: 30px;
  border-color: rgba(139, 92, 246, 0.3);
  box-shadow: 0 20px 50px rgba(0, 0, 0, 0.5);
  animation: slide-in 0.3s cubic-bezier(0.16, 1, 0.3, 1);
  box-sizing: border-box;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 25px;
}

.modal-header h2 {
  margin: 0;
  font-size: 1.4rem;
  font-weight: 700;
}

.close-modal-btn {
  background: transparent;
  border: none;
  color: var(--text-muted);
  font-size: 2rem;
  cursor: pointer;
  line-height: 1;
  outline: none;
  transition: color 0.2s ease;
}

.close-modal-btn:hover {
  color: var(--primary);
}

.modal-body {
  margin-bottom: 25px;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
}

/* Slider Controls */
.slider-label-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.temp-value {
  font-size: 0.85rem;
  font-weight: 700;
  color: #a78bfa;
}

.glass-slider {
  width: 100%;
  height: 6px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 3px;
  outline: none;
  appearance: none;
  -webkit-appearance: none;
  cursor: pointer;
}

.glass-slider::-webkit-slider-thumb {
  appearance: none;
  -webkit-appearance: none;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: linear-gradient(135deg, #ec4899 0%, #8b5cf6 100%);
  cursor: pointer;
  box-shadow: 0 0 10px var(--primary-glow);
  transition: transform 0.1s ease;
}

.glass-slider::-webkit-slider-thumb:hover {
  transform: scale(1.2);
}

.slider-desc {
  font-size: 0.78rem;
  color: var(--text-muted);
  line-height: 1.4;
  margin: 8px 0 0 0;
}

/* Configuration select styling */
.select-wrapper {
  display: flex;
  gap: 10px;
  width: 100%;
}

.glass-select {
  flex: 1;
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid var(--glass-border);
  border-radius: 12px;
  padding: 12px 16px;
  color: var(--text-main);
  font-size: 1rem;
  font-family: inherit;
  outline: none;
  cursor: pointer;
  appearance: none;
  -webkit-appearance: none;
  background-image: url("data:image/svg+xml;utf8,<svg fill='white' height='24' viewBox='0 0 24 24' width='24' xmlns='http://www.w3.org/2000/svg'><path d='M7 10l5 5 5-5z'/><path d='M0 0h24v24H0z' fill='none'/></svg>");
  background-repeat: no-repeat;
  background-position: right 14px center;
  transition: all 0.2s ease;
}

.glass-select:focus {
  border-color: var(--primary);
  box-shadow: 0 0 10px var(--primary-glow);
}

.refresh-btn {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid var(--glass-border);
  border-radius: 12px;
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 1.1rem;
}

.refresh-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.1);
  transform: rotate(45deg);
}

.refresh-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.model-error {
  font-size: 0.85rem;
  color: #fca5a5;
  margin-top: 8px;
  white-space: pre-line;
}

.active-model-indicator {
  font-size: 0.85rem;
  color: var(--text-muted);
  margin-top: 12px;
  display: flex;
  align-items: center;
  gap: 4px;
}

.active-model-name {
  color: #a78bfa;
  font-weight: 600;
  word-break: break-all;
}

/* Preset Selector Styles (SQL) */
.preset-header-inline {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
  gap: 8px;
}

.seed-btn {
  background: rgba(16, 185, 129, 0.12);
  border: 1px solid rgba(16, 185, 129, 0.3);
  color: var(--accent);
  border-radius: 20px;
  padding: 4px 12px;
  font-size: 0.8rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
  outline: none;
}

.seed-btn:hover:not(:disabled) {
  background: rgba(16, 185, 129, 0.22);
  border-color: var(--accent);
  transform: translateY(-1px);
}

.presets-loader {
  font-size: 0.95rem;
  color: var(--text-muted);
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 24px;
  background: rgba(0, 0, 0, 0.15);
  border-radius: 16px;
  justify-content: center;
  border: 1px solid var(--glass-border);
}

.empty-presets-alert {
  font-size: 0.95rem;
  color: var(--text-muted);
  border: 1px dashed rgba(255, 255, 255, 0.1);
  padding: 24px;
  border-radius: 16px;
  text-align: center;
  background: rgba(0, 0, 0, 0.1);
}

.empty-presets-alert p {
  margin: 0 0 10px 0;
}

.btn-sm {
  padding: 8px 14px;
  font-size: 0.85rem;
  border-radius: 10px;
}

/* Presets Grid */
.presets-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
  margin-bottom: 20px;
}

.preset-btn {
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid var(--glass-border);
  border-radius: 14px;
  padding: 12px 16px;
  text-align: left;
  cursor: pointer;
  transition: all 0.2s ease;
  color: var(--text-main);
  outline: none;
}

.preset-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(255, 255, 255, 0.15);
  transform: translateY(-2px);
}

.preset-btn.active {
  background: rgba(236, 72, 153, 0.08);
  border-color: var(--primary);
  box-shadow: 0 0 12px var(--primary-glow);
}

.preset-vocab {
  font-size: 1.1rem;
  font-weight: 700;
  margin-bottom: 4px;
}

.preset-meta {
  font-size: 0.75rem;
  color: var(--text-muted);
}

/* Custom Toggle */
.custom-toggle-container {
  margin-bottom: 20px;
}

.custom-toggle-btn {
  background: transparent;
  border: 1px dashed var(--glass-border);
  color: var(--text-muted);
  width: 100%;
  padding: 12px;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 0.9rem;
}

.custom-toggle-btn:hover {
  border-color: var(--secondary);
  color: var(--text-main);
}

.custom-toggle-btn.active {
  border-color: var(--secondary);
  color: var(--secondary);
  background: rgba(139, 92, 246, 0.05);
}

/* Custom Inputs */
.custom-inputs {
  display: flex;
  flex-direction: column;
  gap: 15px;
  margin-bottom: 20px;
  overflow: hidden;
}

.input-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.input-group label {
  font-size: 0.9rem;
  font-weight: 500;
  color: var(--text-muted);
}

.glass-input {
  background: rgba(0, 0, 0, 0.25);
  border: 1px solid var(--glass-border);
  border-radius: 12px;
  padding: 12px 16px;
  color: var(--text-main);
  font-size: 1rem;
  transition: all 0.2s ease;
  outline: none;
  font-family: inherit;
}

.glass-input:focus {
  border-color: var(--secondary);
  box-shadow: 0 0 10px var(--secondary-glow);
}

.glass-textarea {
  background: rgba(0, 0, 0, 0.25);
  border: 1px solid var(--glass-border);
  border-radius: 14px;
  padding: 14px;
  color: var(--text-main);
  font-size: 1rem;
  line-height: 1.5;
  transition: all 0.2s ease;
  outline: none;
  resize: none;
  font-family: inherit;
}

.glass-textarea:focus {
  border-color: var(--primary);
  box-shadow: 0 0 12px var(--primary-glow);
}

.glass-textarea:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Buttons */
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  border-radius: 12px;
  font-size: 0.95rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  padding: 12px 20px;
  outline: none;
}

.btn-primary {
  background: linear-gradient(135deg, #ec4899 0%, #8b5cf6 100%);
  border: none;
  color: white;
  box-shadow: 0 4px 15px var(--primary-glow);
}

.btn-primary:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(236, 72, 153, 0.5);
}

.btn-primary:active:not(:disabled) {
  transform: translateY(0);
}

.btn-primary:disabled {
  opacity: 0.4;
  cursor: not-allowed;
  box-shadow: none;
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid var(--glass-border);
  color: var(--text-main);
}

.btn-secondary:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(255, 255, 255, 0.2);
}

.btn-secondary:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.btn-block {
  display: flex;
  width: 100%;
}

.btn-eval {
  height: 52px;
  font-size: 1.05rem;
}

/* Divider */
.divider {
  height: 1px;
  background: radial-gradient(circle, rgba(255, 255, 255, 0.1) 0%, rgba(255, 255, 255, 0) 100%);
  margin: 25px 0;
}

/* Hints System Layout */
.hints-system {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.hints-buttons {
  display: flex;
  gap: 12px;
}

@media (max-width: 600px) {
  .hints-buttons {
    flex-direction: column;
  }
}

.hint-bubble {
  border-radius: 14px;
  padding: 16px;
  font-size: 0.95rem;
  line-height: 1.5;
  border: 1px solid;
  margin-top: 10px;
}

.hint-bubble.tier1 {
  background: rgba(16, 185, 129, 0.05);
  border-color: rgba(16, 185, 129, 0.2);
}

.hint-bubble.tier2 {
  background: rgba(139, 92, 246, 0.05);
  border-color: rgba(139, 92, 246, 0.2);
}

.bubble-header {
  font-size: 0.8rem;
  font-weight: 700;
  text-transform: uppercase;
  margin-bottom: 6px;
  letter-spacing: 0.5px;
}

.hint-bubble p {
  margin: 0;
}

/* Active card display */
.active-card-display {
  background: linear-gradient(135deg, rgba(236, 72, 153, 0.08) 0%, rgba(139, 92, 246, 0.08) 100%);
  border: 1px solid rgba(236, 72, 153, 0.15);
  border-radius: 16px;
  padding: 20px;
  text-align: center;
  margin-bottom: 25px;
}

.card-sub {
  font-size: 0.8rem;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 1px;
}

.card-main-vocab {
  font-size: 2.2rem;
  font-weight: 800;
  margin: 10px 0;
  letter-spacing: 1px;
  text-shadow: 0 0 15px var(--primary-glow);
}

.card-anime-ref {
  font-size: 0.85rem;
  color: var(--text-muted);
  background: rgba(0, 0, 0, 0.2);
  padding: 4px 10px;
  border-radius: 20px;
  display: inline-block;
}

/* Spinners & Loaders */
.spinner {
  width: 18px;
  height: 18px;
  border: 2px solid rgba(255, 255, 255, 0.2);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  display: inline-block;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.eval-loading-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
}

.pulsing-ring {
  width: 18px;
  height: 18px;
  border: 2.5px solid rgba(255, 255, 255, 0.3);
  border-radius: 50%;
  border-top-color: #fff;
  animation: spin 1s linear infinite;
}

/* Error Alert */
.error-alert {
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: 12px;
  padding: 14px;
  display: flex;
  gap: 12px;
  margin-top: 15px;
  color: #fca5a5;
  font-size: 0.9rem;
  line-height: 1.5;
}

.error-alert p {
  margin: 0;
  white-space: pre-line;
}

/* Evaluation Results Layout */
.evaluation-result {
  margin-top: 10px;
}

.result-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 18px;
}

.result-header h3 {
  font-size: 1.25rem;
  margin: 0;
  font-weight: 700;
}

.score-badge {
  display: flex;
  align-items: baseline;
  padding: 8px 16px;
  border-radius: 16px;
  font-weight: 800;
  border: 1px solid;
}

.score-num {
  font-size: 1.6rem;
}

.score-label {
  font-size: 0.85rem;
  opacity: 0.7;
  margin-left: 2px;
}

.score-success {
  background: rgba(16, 185, 129, 0.1);
  border-color: var(--accent);
  color: var(--accent);
  box-shadow: 0 0 10px var(--accent-glow);
}

.score-warning {
  background: rgba(245, 158, 11, 0.1);
  border-color: #f59e0b;
  color: #fbbf24;
}

.score-danger {
  background: rgba(239, 68, 68, 0.1);
  border-color: var(--error);
  color: #fca5a5;
  box-shadow: 0 0 10px var(--error-glow);
}

.status-banner {
  border-radius: 12px;
  padding: 12px;
  text-align: center;
  font-weight: 700;
  font-size: 1rem;
  margin-bottom: 20px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.status-banner.success {
  background: rgba(16, 185, 129, 0.15);
  border: 1px solid rgba(16, 185, 129, 0.25);
  color: var(--accent);
}

.status-banner.failure {
  background: rgba(239, 68, 68, 0.15);
  border: 1px solid rgba(239, 68, 68, 0.25);
  color: #f87171;
}

.result-body {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.result-block h4 {
  font-size: 0.9rem;
  color: var(--text-muted);
  margin: 0 0 6px 0;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.explanation-text {
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid var(--glass-border);
  border-radius: 12px;
  padding: 14px;
  font-size: 0.95rem;
  line-height: 1.5;
  margin: 0;
}

.correction-block {
  background: rgba(139, 92, 246, 0.04);
  border: 1px dashed rgba(139, 92, 246, 0.2);
  border-radius: 14px;
  padding: 14px;
}

.correction-text {
  font-size: 1.2rem;
  font-weight: 700;
  color: #c084fc;
  margin: 0;
}

/* Japanese Font Styling */
.japanese-font {
  font-family: 'Noto Sans JP', 'Outfit', sans-serif;
}

/* Transitions */
.expand-enter-active,
.expand-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  max-height: 200px;
}

.expand-enter-from,
.expand-leave-to {
  max-height: 0;
  opacity: 0;
  margin-bottom: 0;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.25s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

@keyframes slide-in {
  from {
    opacity: 0;
    transform: translateY(-20px) scale(0.95);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

.slide-up-enter-active {
  transition: all 0.35s cubic-bezier(0.16, 1, 0.3, 1);
}

.slide-up-enter-from {
  opacity: 0;
  transform: translateY(20px);
}

/* Custom Settings Styles */
.presets-row {
  display: flex;
  gap: 8px;
  margin-top: 8px;
  flex-wrap: wrap;
}

.preset-badge {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid var(--glass-border);
  color: var(--text-muted);
  font-size: 0.75rem;
  padding: 4px 10px;
  border-radius: 15px;
  cursor: pointer;
  transition: all 0.2s ease;
  font-family: inherit;
}

.preset-badge:hover {
  background: rgba(139, 92, 246, 0.15);
  border-color: var(--secondary);
  color: var(--text-main);
  transform: translateY(-1px);
}

.auto-detect-section {
  margin-top: 15px;
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid var(--glass-border);
  border-radius: 16px;
  padding: 16px;
}

.sub-label {
  display: block;
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--text-muted);
  margin-bottom: 8px;
}

/* Custom Settings Styles */
.settings-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(340px, 1fr));
  gap: 30px;
  align-items: start;
}

.settings-card {
  height: auto;
  box-sizing: border-box;
}

.success-border {
  border-color: rgba(16, 185, 129, 0.3) !important;
}

.success-border:hover {
  border-color: rgba(16, 185, 129, 0.5) !important;
  box-shadow: 0 15px 40px rgba(16, 185, 129, 0.15) !important;
}

.btn-connect {
  height: 48px;
}

.btn-save {
  height: 48px;
  background: linear-gradient(135deg, #10b981 0%, #059669 100%) !important;
  box-shadow: 0 4px 15px rgba(16, 185, 129, 0.3) !important;
}

.btn-save:hover:not(:disabled) {
  box-shadow: 0 6px 20px rgba(16, 185, 129, 0.5) !important;
}

.warning-alert-box {
  background: rgba(245, 158, 11, 0.07);
  border: 1px solid rgba(245, 158, 11, 0.25);
  border-radius: 14px;
  padding: 14px 20px;
  display: flex;
  gap: 15px;
  align-items: flex-start;
  color: #fef08a;
  font-size: 0.9rem;
}

.warning-text {
  margin: 0;
  line-height: 1.5;
}

.advanced-settings-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 24px;
}

.blinking-badge {
  background: rgba(239, 68, 68, 0.15);
  border: 1px dashed rgba(239, 68, 68, 0.4);
  color: #fca5a5;
  padding: 6px 12px;
  border-radius: 20px;
  font-size: 0.85rem;
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  animation: pulse-blinking 2s infinite ease-in-out;
  transition: all 0.3s ease;
  margin-right: 15px;
}
.blinking-badge:hover {
  background: rgba(239, 68, 68, 0.25);
  border-color: rgba(239, 68, 68, 0.6);
  transform: translateY(-1px);
}

@keyframes pulse-blinking {
  0% { opacity: 0.8; box-shadow: 0 0 0 0 rgba(239, 68, 68, 0.4); }
  50% { opacity: 1; box-shadow: 0 0 0 6px rgba(239, 68, 68, 0); }
  100% { opacity: 0.8; box-shadow: 0 0 0 0 rgba(239, 68, 68, 0); }
}

.header-config-selector-wrapper {
  display: flex;
  align-items: center;
}

.header-config-select {
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid var(--glass-border);
  color: #e2e8f0;
  padding: 0 14px;
  border-radius: 12px;
  height: 40px;
  font-size: 0.85rem;
  font-weight: 600;
  outline: none;
  cursor: pointer;
  transition: all 0.3s ease;
  min-width: 180px;
  box-sizing: border-box;
}

.header-config-select:hover {
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(255, 255, 255, 0.2);
  transform: translateY(-1px);
}

.header-config-select option {
  background: #1e293b;
  color: #f1f5f9;
}

.saved-configs-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 12px;
  padding: 10px;
  margin-top: 8px;
}

.config-list-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.05);
  transition: all 0.2s ease;
}

.config-list-item:hover {
  background: rgba(255, 255, 255, 0.06);
}

.config-list-item.active-item {
  background: rgba(16, 185, 129, 0.08);
  border-color: rgba(16, 185, 129, 0.3);
}

.config-item-name {
  cursor: pointer;
  font-size: 0.9rem;
  color: #cbd5e1;
  flex: 1;
  text-align: left;
}

.config-item-name strong {
  color: #f1f5f9;
}

.delete-config-btn {
  background: none;
  border: none;
  cursor: pointer;
  padding: 4px;
  font-size: 0.9rem;
  opacity: 0.6;
  transition: opacity 0.2s ease;
}

.delete-config-btn:hover {
  opacity: 1;
}

.edit-badge {
  font-size: 0.8rem;
  font-weight: 600;
  color: #fbbf24;
  background: rgba(251, 191, 36, 0.1);
  padding: 3px 10px;
  border-radius: 999px;
  border: 1px solid rgba(251, 191, 36, 0.2);
  margin-left: auto;
  white-space: nowrap;
}

.edit-badge.create-badge {
  color: #22c55e;
  background: rgba(34, 197, 94, 0.1);
  border-color: rgba(34, 197, 94, 0.2);
}

.config-item-actions {
  display: flex;
  gap: 4px;
  align-items: center;
}

.edit-config-btn {
  background: none;
  border: none;
  cursor: pointer;
  padding: 4px;
  font-size: 0.9rem;
  opacity: 0.5;
  transition: opacity 0.2s ease;
}

.edit-config-btn:hover {
  opacity: 1;
}
</style>
