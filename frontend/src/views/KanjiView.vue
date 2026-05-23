<template>
  <div class="kanji-grid">
    <section class="radical-section">
      <div class="glass-panel">
        <div class="panel-header">
          <span class="step-num">01</span>
          <h2>Radicaux Kangxi (214)</h2>
        </div>

        <div class="filter-bar">
          <div class="jlpt-filters">
            <span class="filter-label">JLPT :</span>
            <button
              v-for="level in jlptLevels"
              :key="level.label"
              @click="selectedJlpt = level.value"
              :class="['jlpt-btn', { active: selectedJlpt === level.value }]"
            >
              {{ level.label }}
            </button>
          </div>
          <input
            type="text"
            v-model="radicalSearch"
            placeholder="Filtrer par sens (eau, feu, main...)"
            class="glass-input search-input"
          />
        </div>

        <div v-if="loading" class="loading-state">
          <span class="spinner"></span>
          <span>Chargement des radicaux...</span>
        </div>

        <div v-else class="radical-grid">
          <button
            v-for="rad in filteredRadicals"
            :key="rad.number"
            @click="selectRadical(rad)"
            :class="[
              'radical-btn',
              { active: selectedRadical?.number === rad.number },
            ]"
          >
            <span class="radical-char">{{ rad.char }}</span>
            <span class="radical-num">{{ rad.number }}</span>
            <span class="radical-meaning">{{ rad.meaning }}</span>
          </button>
        </div>
      </div>
    </section>

    <section class="detail-section">
      <div v-if="!selectedRadical && !selectedKanjiChar" class="glass-panel detail-placeholder">
        <div class="placeholder-content">
          <span class="placeholder-icon">🔍</span>
          <h3>Sélectionnez un radical</h3>
          <p>Parcourez les 214 radicaux Kangxi pour explorer les kanji.</p>
        </div>
      </div>

      <div v-else-if="selectedKanji" class="glass-panel">
        <div class="detail-header">
          <button @click="selectedKanjiChar = null" class="back-btn">◀ Retour</button>
        </div>

        <div class="kanji-detail">
          <div class="kanji-hero">
            <span class="kanji-char-display">{{ selectedKanji.char }}</span>
          </div>

          <div class="kanji-info-grid">
            <div class="info-card">
              <span class="info-label">JLPT</span>
              <span class="info-value">N{{ selectedKanji.jlpt_level }}</span>
            </div>
            <div class="info-card">
              <span class="info-label">Traits</span>
              <span class="info-value">{{ selectedKanji.stroke_count || '?' }}</span>
            </div>
            <div class="info-card">
              <span class="info-label">Grade</span>
              <span class="info-value">{{ selectedKanji.grade ? `第${selectedKanji.grade}` : '—' }}</span>
            </div>
            <div class="info-card">
              <span class="info-label">Fréquence</span>
              <span class="info-value">{{ selectedKanji.frequency_rank ? `#${selectedKanji.frequency_rank}` : '—' }}</span>
            </div>
          </div>

          <div class="detail-block">
            <h4>Lecture on'yomi (音読み)</h4>
            <div class="yomi-tags">
              <span v-for="y in selectedKanji.on_yomi" :key="y" class="yomi-tag on-yomi">{{ y }}</span>
              <span v-if="selectedKanji.on_yomi.length === 0" class="no-data">—</span>
            </div>
          </div>

          <div class="detail-block">
            <h4>Lecture kun'yomi (訓読み)</h4>
            <div class="yomi-tags">
              <span v-for="y in selectedKanji.kun_yomi" :key="y" class="yomi-tag kun-yomi">{{ y }}</span>
              <span v-if="selectedKanji.kun_yomi.length === 0" class="no-data">—</span>
            </div>
          </div>

          <div class="detail-block">
            <h4>Sens (EN)</h4>
            <div class="meanings-list">
              <span v-for="m in selectedKanji.meanings_en" :key="m" class="meaning-item">{{ m }}</span>
            </div>
          </div>

          <div v-if="selectedKanji.components.length > 0" class="detail-block">
            <h4>Composants</h4>
            <div class="components-grid">
              <span
                v-for="c in selectedKanji.components"
                :key="c"
                class="component-char"
                @click="openKanjiFromComponent(c)"
              >
                {{ c }}
              </span>
            </div>
          </div>

          <div class="detail-block">
            <h4>Radical</h4>
            <div class="radical-ref" @click="goToRadical(selectedKanji.radical_number!)">
              <span class="rr-char">{{ radicalForCurrentKanji?.char }}</span>
              <span class="rr-number">Radical {{ selectedKanji.radical_number }}</span>
              <span class="rr-meaning">{{ radicalForCurrentKanji?.meaning }}</span>
            </div>
          </div>
        </div>
      </div>

      <div v-else class="glass-panel">
        <div class="detail-header">
          <span class="radical-title-display">
            <span class="rr-big-char">{{ selectedRadical!.char }}</span>
            <span class="rr-info">
              <strong>{{ selectedRadical!.meaning }}</strong>
              <span class="rr-meta">Radical {{ selectedRadical!.number }} — {{ selectedRadical!.stroke_count }} traits</span>
            </span>
          </span>
        </div>

        <div v-if="selectedRadical!.variants.length > 0" class="detail-block">
          <h4>Variantes</h4>
          <div class="variants-list">
            <span v-for="v in selectedRadical!.variants" :key="v" class="variant-char">{{ v }}</span>
          </div>
        </div>

        <div class="detail-block">
          <h4>Kanji liés à ce radical ({{ radicalKanjiList.length }})</h4>
          <div v-if="loadingKanjiForRadical" class="loading-state">
            <span class="spinner"></span>
            <span>Chargement...</span>
          </div>
          <div v-else class="kanji-grid-mini">
            <button
              v-for="k in radicalKanjiList"
              :key="k.char"
              @click="selectKanji(k.char)"
              :class="['kanji-mini-btn', { active: selectedKanjiChar === k.char }]"
            >
              <span class="km-char">{{ k.char }}</span>
              <span class="km-meta">{{ k.stroke_count }}t</span>
              <span class="km-jlpt">N{{ k.jlpt_level }}</span>
            </button>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { fetchRadicals, fetchKanjiList, fetchKanjiDetail } from '../services/kanji'
import type { Radical, KanjiMeta, KanjiEntry } from '../services/kanji'

const loading = ref(true)
const radicals = ref<Radical[]>([])
const allKanjiMeta = ref<KanjiMeta[]>([])
const selectedRadical = ref<Radical | null>(null)
const selectedKanjiChar = ref<string | null>(null)
const selectedKanji = ref<KanjiEntry | null>(null)
const selectedJlpt = ref<number | null>(null)
const radicalSearch = ref('')
const loadingKanjiForRadical = ref(false)

const jlptLevels = [
  { label: 'Tous', value: null },
  { label: 'N5', value: 5 },
  { label: 'N4', value: 4 },
  { label: 'N3', value: 3 },
  { label: 'N2', value: 2 },
  { label: 'N1', value: 1 },
]

const filteredRadicals = computed(() => {
  let list = radicals.value
  if (radicalSearch.value.trim()) {
    const q = radicalSearch.value.toLowerCase()
    list = list.filter(
      (r) =>
        r.meaning.toLowerCase().includes(q) ||
        r.char === q ||
        r.number.toString() === q ||
        r.variants.some((v) => v === q),
    )
  }
  return list
})

const radicalKanjiList = computed(() => {
  if (!selectedRadical.value) return []
  const num = selectedRadical.value.number
  let list = allKanjiMeta.value.filter((k) => k.radical_number === num)
  if (selectedJlpt.value !== null) {
    list = list.filter((k) => k.jlpt_level === selectedJlpt.value)
  }
  list.sort((a, b) => (a.frequency_rank ?? 9999) - (b.frequency_rank ?? 9999))
  return list
})

const radicalForCurrentKanji = computed(() => {
  if (!selectedKanji.value?.radical_number) return null
  return radicals.value.find((r) => r.number === selectedKanji.value!.radical_number) || null
})

function selectRadical(rad: Radical) {
  selectedRadical.value = rad
  selectedKanjiChar.value = null
  selectedKanji.value = null
}

function selectKanji(char: string) {
  selectedKanjiChar.value = char
}

function goToRadical(number: number) {
  const rad = radicals.value.find((r) => r.number === number)
  if (rad) {
    selectRadical(rad)
  }
}

function openKanjiFromComponent(char: string) {
  const rad = radicals.value.find(
    (r) => r.char === char || r.variants.includes(char),
  )
  if (rad) {
    goToRadical(rad.number)
    return
  }
  if (allKanjiMeta.value.some((k) => k.char === char)) {
    selectKanji(char)
  }
}

watch(selectedKanjiChar, async (char) => {
  if (!char) {
    selectedKanji.value = null
    return
  }
  try {
    selectedKanji.value = await fetchKanjiDetail(char)
  } catch {
    selectedKanji.value = null
  }
})

watch(selectedRadical, async () => {
  selectedKanji.value = null
})

onMounted(async () => {
  try {
    const [rads, kanjiList] = await Promise.all([fetchRadicals(), fetchKanjiList()])
    radicals.value = rads
    allKanjiMeta.value = kanjiList
  } catch {
    radicals.value = []
    allKanjiMeta.value = []
  } finally {
    loading.value = false
  }
})
</script>

<style scoped>
.kanji-grid {
  display: grid;
  grid-template-columns: 1fr 1.1fr;
  gap: 30px;
  align-items: start;
}
@media (max-width: 900px) {
  .kanji-grid {
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

.filter-bar {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 20px;
}

.jlpt-filters {
  display: flex;
  gap: 6px;
  align-items: center;
  flex-wrap: wrap;
}

.filter-label {
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--text-muted);
  margin-right: 6px;
}

.jlpt-btn {
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid var(--glass-border);
  color: var(--text-muted);
  font-size: 0.8rem;
  font-weight: 600;
  padding: 4px 12px;
  border-radius: 20px;
  cursor: pointer;
  transition: all 0.2s ease;
  font-family: inherit;
}

.jlpt-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  color: var(--text-main);
}

.jlpt-btn.active {
  background: rgba(236, 72, 153, 0.12);
  border-color: var(--primary);
  color: var(--primary);
}

.search-input {
  width: 100%;
  box-sizing: border-box;
}

.glass-input {
  background: rgba(0, 0, 0, 0.25);
  border: 1px solid var(--glass-border);
  border-radius: 12px;
  padding: 10px 14px;
  color: var(--text-main);
  font-size: 0.9rem;
  transition: all 0.2s ease;
  outline: none;
  font-family: inherit;
}

.glass-input:focus {
  border-color: var(--secondary);
  box-shadow: 0 0 10px var(--secondary-glow);
}

.loading-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 40px;
  color: var(--text-muted);
  font-size: 0.95rem;
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

.radical-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(90px, 1fr));
  gap: 8px;
  max-height: 600px;
  overflow-y: auto;
  padding-right: 4px;
}

.radical-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  padding: 8px 4px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid var(--glass-border);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
  color: var(--text-main);
  font-family: inherit;
  outline: none;
}

.radical-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(255, 255, 255, 0.15);
  transform: translateY(-2px);
}

.radical-btn.active {
  background: rgba(236, 72, 153, 0.08);
  border-color: var(--primary);
  box-shadow: 0 0 8px var(--primary-glow);
}

.radical-char {
  font-size: 1.5rem;
  font-weight: 700;
  line-height: 1.2;
}

.radical-num {
  font-size: 0.65rem;
  color: var(--text-muted);
  font-weight: 600;
}

.radical-meaning {
  font-size: 0.65rem;
  color: var(--text-muted);
  text-align: center;
  line-height: 1.2;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 100%;
  white-space: nowrap;
}

.detail-placeholder .placeholder-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  text-align: center;
  color: var(--text-muted);
}

.placeholder-icon {
  font-size: 3rem;
  margin-bottom: 16px;
}

.placeholder-content h3 {
  font-size: 1.3rem;
  margin: 0 0 8px 0;
  color: var(--text-main);
}

.placeholder-content p {
  font-size: 0.95rem;
  margin: 0;
}

.detail-header {
  margin-bottom: 20px;
  display: flex;
  align-items: center;
}

.back-btn {
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid var(--glass-border);
  color: var(--text-muted);
  font-size: 0.85rem;
  font-weight: 600;
  padding: 6px 14px;
  border-radius: 20px;
  cursor: pointer;
  transition: all 0.2s ease;
  font-family: inherit;
}

.back-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  color: var(--text-main);
}

.radical-title-display {
  display: flex;
  align-items: center;
  gap: 16px;
}

.rr-big-char {
  font-size: 3rem;
  font-weight: 800;
  line-height: 1;
}

.rr-info {
  display: flex;
  flex-direction: column;
}

.rr-info strong {
  font-size: 1.2rem;
  color: var(--text-main);
}

.rr-meta {
  font-size: 0.8rem;
  color: var(--text-muted);
}

.detail-block {
  margin-bottom: 20px;
}

.detail-block h4 {
  font-size: 0.8rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--text-muted);
  margin: 0 0 8px 0;
}

.variants-list {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.variant-char {
  font-size: 1.5rem;
  font-weight: 700;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid var(--glass-border);
  padding: 6px 10px;
  border-radius: 10px;
  line-height: 1;
}

.kanji-grid-mini {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(72px, 1fr));
  gap: 8px;
  max-height: 500px;
  overflow-y: auto;
  padding-right: 4px;
}

.kanji-mini-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  padding: 8px 4px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid var(--glass-border);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
  color: var(--text-main);
  font-family: inherit;
  outline: none;
}

.kanji-mini-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(255, 255, 255, 0.15);
  transform: translateY(-2px);
}

.kanji-mini-btn.active {
  background: rgba(139, 92, 246, 0.1);
  border-color: var(--secondary);
  box-shadow: 0 0 8px var(--secondary-glow);
}

.km-char {
  font-size: 1.4rem;
  font-weight: 700;
  line-height: 1.2;
}

.km-meta {
  font-size: 0.6rem;
  color: var(--text-muted);
}

.km-jlpt {
  font-size: 0.6rem;
  color: var(--primary);
  font-weight: 700;
}

.kanji-hero {
  text-align: center;
  padding: 20px 0;
}

.kanji-char-display {
  font-size: 4.5rem;
  font-weight: 800;
  text-shadow: 0 0 20px var(--primary-glow);
  line-height: 1.2;
}

.kanji-info-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 10px;
  margin-bottom: 24px;
}

.info-card {
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid var(--glass-border);
  border-radius: 12px;
  padding: 12px;
  text-align: center;
}

.info-label {
  display: block;
  font-size: 0.7rem;
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.3px;
  margin-bottom: 4px;
}

.info-value {
  font-size: 1.1rem;
  font-weight: 700;
  color: var(--text-main);
}

.yomi-tags {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.yomi-tag {
  font-size: 0.95rem;
  font-weight: 700;
  padding: 4px 12px;
  border-radius: 20px;
  line-height: 1.3;
}

.yomi-tag.on-yomi {
  background: rgba(139, 92, 246, 0.1);
  border: 1px solid rgba(139, 92, 246, 0.25);
  color: #c084fc;
}

.yomi-tag.kun-yomi {
  background: rgba(236, 72, 153, 0.1);
  border: 1px solid rgba(236, 72, 153, 0.25);
  color: #f9a8d4;
}

.no-data {
  color: var(--text-muted);
  font-size: 0.95rem;
}

.meanings-list {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.meaning-item {
  font-size: 0.9rem;
  font-weight: 500;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid var(--glass-border);
  padding: 6px 14px;
  border-radius: 20px;
  line-height: 1.3;
}

.components-grid {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.component-char {
  font-size: 1.6rem;
  font-weight: 700;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid var(--glass-border);
  padding: 6px 12px;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
  line-height: 1;
}

.component-char:hover {
  background: rgba(139, 92, 246, 0.1);
  border-color: var(--secondary);
  transform: translateY(-2px);
}

.radical-ref {
  display: flex;
  align-items: center;
  gap: 12px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid var(--glass-border);
  border-radius: 14px;
  padding: 12px 16px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.radical-ref:hover {
  background: rgba(236, 72, 153, 0.06);
  border-color: var(--primary);
}

.rr-char {
  font-size: 2rem;
  font-weight: 800;
  line-height: 1;
}

.rr-number {
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--text-muted);
}

.rr-meaning {
  font-size: 0.9rem;
  font-weight: 500;
  color: var(--text-main);
}
</style>
