<template>
  <div class="playground-grid">
    <section class="card-section">
      <div class="glass-panel">
        <div class="panel-header">
          <span class="step-num">01</span>
          <h2>Choisissez un défi japonais</h2>
        </div>

        <div class="preset-selector">
          <div class="preset-header-inline">
            <label class="section-label"
              >Sélection depuis la base de données :</label
            >
            <button
              v-if="!isSeeded"
              @click="handleSeed"
              class="seed-btn"
              :disabled="loadingCards"
              title="Initialiser la base PostgreSQL/Supabase avec nos exemples cultes"
            >
              🌱 Remplir la BDD
            </button>
          </div>

          <div v-if="loadingCards" class="presets-loader">
            <span class="spinner"></span>
            <span>Chargement des cartes depuis la base...</span>
          </div>

          <div v-else-if="presets.length === 0" class="empty-presets-alert">
            <p>La base de données est connectée mais vide.</p>
            <button
              @click="handleSeed"
              class="btn btn-secondary btn-sm"
              style="margin-top: 10px"
            >
              🌱 Charger le deck 'Animés Légendaires'
            </button>
          </div>

          <div v-else class="presets-grid">
            <button
              v-for="item in presets"
              :key="item.vocab"
              @click="selectPreset(item)"
              :class="[
                'preset-btn',
                { active: currentVocab === item.vocab && !isCustom },
              ]"
            >
              <div class="preset-vocab">{{ item.vocab }}</div>
              <div class="preset-meta">{{ item.anime }}</div>
            </button>
          </div>
        </div>

        <div class="custom-toggle-container">
          <button
            @click="toggleCustom"
            :class="['custom-toggle-btn', { active: isCustom }]"
          >
            ⌨️ Saisir mon propre mot / expression
          </button>
        </div>

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

        <div class="divider"></div>

        <div class="hints-system">
          <label class="section-label"
            >Besoin d'aide ? Demandez des indices progressifs :</label
          >
          <div class="hints-buttons">
            <button
              @click="handleFetchHint(1)"
              :disabled="loadingHint1 || !activeVocab"
              class="btn btn-secondary"
            >
              <span v-if="loadingHint1" class="spinner"></span>
              <span v-else>💡 Indice Tier 1 (Lecture)</span>
            </button>
            <button
              @click="handleFetchHint(2)"
              :disabled="loadingHint2 || !activeVocab"
              class="btn btn-secondary"
            >
              <span v-if="loadingHint2" class="spinner"></span>
              <span v-else>🎭 Indice Tier 2 (Contexte)</span>
            </button>
          </div>

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

    <section class="answer-section">
      <div class="glass-panel">
        <div class="panel-header">
          <span class="step-num">02</span>
          <h2>Saisissez votre traduction</h2>
        </div>

        <div class="active-card-display">
          <span class="card-sub">Expression ciblée :</span>
          <div class="card-main-vocab japanese-font">
            {{ activeVocab || 'Aucune sélection' }}
          </div>
          <div v-if="activeAnime" class="card-anime-ref">
            🎬 Manga : {{ activeAnime }}
          </div>
        </div>

        <div class="input-group">
          <label for="user-answer"
            >Votre traduction ou explication en français :</label
          >
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
          @click="handleEvaluate"
          :disabled="loadingEval || !userAnswer.trim() || !activeVocab"
          class="btn btn-primary btn-block btn-eval"
        >
          <div v-if="loadingEval" class="eval-loading-wrapper">
            <span class="pulsing-ring"></span>
            <span>Analyse IA en cours...</span>
          </div>
          <span v-else>🌸 Évaluer avec l'IA locale</span>
        </button>

        <div v-if="errorMessage" class="error-alert">
          <span class="alert-icon">⚠️</span>
          <p>{{ errorMessage }}</p>
        </div>

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
            <div
              :class="[
                'status-banner',
                evaluation.is_correct ? 'success' : 'failure',
              ]"
            >
              <span class="status-icon">{{
                evaluation.is_correct ? '✅ Réponse Validée !' : '❌ À corriger'
              }}</span>
            </div>
            <div class="result-body">
              <div class="result-block">
                <h4>💡 Analyse du tuteur :</h4>
                <p class="explanation-text">{{ evaluation.explanation }}</p>
              </div>
              <div
                v-if="evaluation.correction"
                class="result-block correction-block"
              >
                <h4>✍️ Correction suggérée :</h4>
                <p class="correction-text japanese-font">
                  {{ evaluation.correction }}
                </p>
              </div>
            </div>
          </div>
        </Transition>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { PresetItem, EvalData } from '../services/api'
import { fetchCards, seedDatabase } from '../services/cards'
import { fetchHint, evaluateAnswer } from '../services/llm'

const props = defineProps<{
  apiUrl: string
  apiKey: string
  selectedModel: string
  temperatureEval: number
  temperatureHint: number
  topP: number
  maxTokens: number
  frequencyPenalty: number
}>()

const presets = ref<PresetItem[]>([])
const loadingCards = ref(false)
const isSeeded = ref(false)

const currentVocab = ref('')
const currentAnime = ref('')
const isCustom = ref(false)
const customVocab = ref('')
const customAnime = ref('')

const activeVocab = computed(() =>
  isCustom.value ? customVocab.value : currentVocab.value,
)
const activeAnime = computed(() =>
  isCustom.value ? customAnime.value : currentAnime.value,
)

const hint1 = ref('')
const hint2 = ref('')
const loadingHint1 = ref(false)
const loadingHint2 = ref(false)
const userAnswer = ref('')
const loadingEval = ref(false)
const errorMessage = ref('')
const evaluation = ref<EvalData | null>(null)

watch(activeVocab, () => {
  hint1.value = ''
  hint2.value = ''
  userAnswer.value = ''
  evaluation.value = null
  errorMessage.value = ''
})

function selectPreset(item: PresetItem) {
  isCustom.value = false
  currentVocab.value = item.vocab
  currentAnime.value = item.anime
}

function toggleCustom() {
  isCustom.value = !isCustom.value
}

async function loadCards() {
  loadingCards.value = true
  try {
    const result = await fetchCards()
    presets.value = result.presets
    isSeeded.value = result.isSeeded
    if (result.isSeeded && !currentVocab.value) {
      currentVocab.value = result.presets[0].vocab
      currentAnime.value = result.presets[0].anime
    }
  } catch {
    isSeeded.value = false
  } finally {
    loadingCards.value = false
  }
}

async function handleSeed() {
  loadingCards.value = true
  errorMessage.value = ''
  try {
    await seedDatabase()
    await loadCards()
  } catch (err: any) {
    errorMessage.value = `Impossible d'initialiser la base de données : ${err.message}.`
  } finally {
    loadingCards.value = false
  }
}

async function handleFetchHint(tier: number) {
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
    const result = await fetchHint({
      vocab: activeVocab.value,
      tier,
      model: props.selectedModel || null,
      temperature: props.temperatureHint,
      api_url: props.apiUrl || null,
      api_key: props.apiKey || null,
      top_p: props.topP,
      max_tokens: props.maxTokens,
      frequency_penalty: props.frequencyPenalty,
    })
    if (tier === 1) hint1.value = result
    else hint2.value = result
  } catch (err: any) {
    errorMessage.value = `Impossible d'obtenir l'indice : ${err.message}.`
  } finally {
    loadingHint1.value = false
    loadingHint2.value = false
  }
}

async function handleEvaluate() {
  if (!activeVocab.value || !userAnswer.value.trim()) return
  loadingEval.value = true
  evaluation.value = null
  errorMessage.value = ''

  try {
    const result = await evaluateAnswer({
      vocab: activeVocab.value,
      user_answer: userAnswer.value,
      context: activeAnime.value
        ? `Manga de référence: ${activeAnime.value}`
        : null,
      model: props.selectedModel || null,
      temperature: props.temperatureEval,
      api_url: props.apiUrl || null,
      api_key: props.apiKey || null,
      top_p: props.topP,
      max_tokens: props.maxTokens,
      frequency_penalty: props.frequencyPenalty,
    })
    evaluation.value = result
  } catch (err: any) {
    errorMessage.value = `Échec de l'évaluation : ${err.message}.`
  } finally {
    loadingEval.value = false
  }
}

function getScoreClass(score: number) {
  if (score >= 80) return 'score-success'
  if (score >= 50) return 'score-warning'
  return 'score-danger'
}

onMounted(() => {
  loadCards()
})
</script>

<style scoped>
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
.btn-block {
  display: flex;
  width: 100%;
}
.btn-eval {
  height: 52px;
  font-size: 1.05rem;
}
.btn-sm {
  padding: 8px 14px;
  font-size: 0.85rem;
  border-radius: 10px;
}

.divider {
  height: 1px;
  background: radial-gradient(
    circle,
    rgba(255, 255, 255, 0.1) 0%,
    rgba(255, 255, 255, 0) 100%
  );
  margin: 25px 0;
}

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

.active-card-display {
  background: linear-gradient(
    135deg,
    rgba(236, 72, 153, 0.08) 0%,
    rgba(139, 92, 246, 0.08) 100%
  );
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
  to {
    transform: rotate(360deg);
  }
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
.japanese-font {
  font-family: 'Noto Sans JP', 'Outfit', sans-serif;
}

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
.slide-up-enter-active {
  transition: all 0.35s cubic-bezier(0.16, 1, 0.3, 1);
}
.slide-up-enter-from {
  opacity: 0;
  transform: translateY(20px);
}
</style>
