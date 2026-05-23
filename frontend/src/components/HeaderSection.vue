<template>
  <header class="playground-header">
    <div class="header-top-row">
      <div class="logo-wrapper">
        <span class="logo-icon">🌸</span>
        <h1 class="logo-text">LearnWithAnime</h1>
      </div>

      <div class="header-actions">
        <!-- Sélecteur de Configuration IA ou Alerte d'Absence -->
        <div v-if="user">
          <div
            v-if="!hasActiveConfig"
            @click="$emit('go-to-settings')"
            class="blinking-badge"
            title="Aucune configuration LLM active ! Cliquez pour en configurer une."
          >
            ⚠️ Configuration IA manquante - Cliquer ici
          </div>
          <div v-else class="header-config-selector-wrapper">
            <select
              id="header-config-select"
              :value="activeConfigName"
              @change="
                $emit(
                  'update:activeConfigName',
                  ($event.target as HTMLSelectElement).value,
                )
                $emit(
                  'activate-config',
                  ($event.target as HTMLSelectElement).value,
                )
              "
              class="header-config-select"
              title="Configuration de l'IA active"
            >
              <option
                v-for="cfg in userConfigs"
                :key="cfg.config_name"
                :value="cfg.config_name"
              >
                🤖 {{ cfg.config_name }}
              </option>
            </select>
          </div>
        </div>

        <!-- Bouton Paramètres -->
        <button
          v-if="currentView === 'playground'"
          @click="$emit('go-to-settings')"
          class="action-icon-btn btn-settings"
          title="Paramètres de l'IA"
        >
          ⚙️
        </button>
        <button
          v-else
          @click="$emit('go-to-playground')"
          class="action-icon-btn btn-settings"
          title="Retour au défi de japonais"
        >
          ◀️
        </button>

        <!-- Bouton Admin (visible uniquement pour les admins) -->
        <button
          v-if="isAdmin"
          @click="$emit('go-to-admin')"
          class="action-icon-btn btn-admin"
          title="Panneau d'administration"
        >
          🔐
        </button>

        <!-- Profil Utilisateur -->
        <div v-if="user" class="user-profile">
          <img
            v-if="avatarUrl"
            :src="avatarUrl"
            class="user-avatar"
            alt="Avatar"
          />
          <div v-else class="user-avatar-placeholder">{{ userInitial }}</div>
          <span class="user-display-name">{{ userDisplayName }}</span>
        </div>

        <!-- Bouton Déconnexion -->
        <button
          @click="$emit('sign-out')"
          class="logout-btn"
          title="Se déconnecter de votre compte"
        >
          🚪
        </button>
      </div>
    </div>
  </header>
</template>

<script setup lang="ts">
interface UserLlmSettings {
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

defineProps<{
  user: any
  avatarUrl: string | null
  userInitial: string
  userDisplayName: string
  hasActiveConfig: boolean
  userConfigs: UserLlmSettings[]
  activeConfigName: string
  currentView: 'playground' | 'settings' | 'admin'
  isAdmin: boolean
}>()

defineEmits<{
  'update:activeConfigName': [value: string]
  'activate-config': [configName: string]
  'go-to-settings': []
  'go-to-playground': []
  'go-to-admin': []
  'sign-out': []
}>()
</script>

<style scoped>
.playground-header {
  margin-bottom: 40px;
}

.logo-wrapper {
  display: flex;
  align-items: center;
  gap: 12px;
}

.logo-icon {
  font-size: 2.5rem;
}

.logo-text {
  font-size: 2.5rem;
  font-weight: 800;
  background: linear-gradient(135deg, #ec4899 0%, #8b5cf6 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  margin: 0;
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-top-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  max-width: var(--content-max-width);
  margin: 0 auto 12px auto;
}

@media (max-width: 768px) {
  .header-top-row {
    flex-direction: column;
    gap: 18px;
  }
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 15px;
}

@media (max-width: 500px) {
  .header-actions {
    flex-direction: column;
    width: 100%;
  }
  .user-profile,
  .action-icon-btn,
  .logout-btn {
    width: 100%;
    justify-content: center;
    box-sizing: border-box;
  }
}

.user-profile {
  display: flex;
  align-items: center;
  gap: 8px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid var(--glass-border);
  height: 40px;
  padding: 0 14px;
  border-radius: 12px;
  box-sizing: border-box;
}

.user-avatar {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  border: 1.5px solid var(--primary);
  object-fit: cover;
}

.user-avatar-placeholder {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: linear-gradient(135deg, #ec4899 0%, #8b5cf6 100%);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  font-size: 0.85rem;
}

.user-display-name {
  font-size: 0.9rem;
  font-weight: 600;
  color: var(--text-main);
}

.action-icon-btn {
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid var(--glass-border);
  color: var(--text-main);
  border-radius: 12px;
  width: 40px;
  height: 40px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 1.1rem;
  cursor: pointer;
  transition: all 0.2s ease;
  font-family: inherit;
  outline: none;
  box-sizing: border-box;
}

.action-icon-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(255, 255, 255, 0.2);
  transform: translateY(-1px);
}

.logout-btn {
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid rgba(239, 68, 68, 0.2);
  color: #fca5a5;
  border-radius: 12px;
  width: 40px;
  height: 40px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 1.1rem;
  cursor: pointer;
  transition: all 0.2s ease;
  outline: none;
  font-family: inherit;
  box-sizing: border-box;
}

.logout-btn:hover {
  background: rgba(239, 68, 68, 0.18);
  border-color: #ef4444;
  transform: translateY(-1px);
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
  0% {
    opacity: 0.8;
    box-shadow: 0 0 0 0 rgba(239, 68, 68, 0.4);
  }
  50% {
    opacity: 1;
    box-shadow: 0 0 0 6px rgba(239, 68, 68, 0);
  }
  100% {
    opacity: 0.8;
    box-shadow: 0 0 0 0 rgba(239, 68, 68, 0);
  }
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
</style>
