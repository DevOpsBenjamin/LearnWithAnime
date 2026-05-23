<template>
  <div class="auth-container">
    <div class="auth-card glass-panel">
      <!-- Logo Header -->
      <div class="auth-header">
        <span class="auth-logo-icon">🌸</span>
        <h1 class="auth-title">LearnWithAnime</h1>
        <p class="auth-subtitle">
          Activez vos connaissances de japonais issus de vos animés favoris
        </p>
      </div>

      <!-- Tab Toggle -->
      <div class="tab-toggle">
        <button
          @click="activeTab = 'signin'"
          :class="['tab-btn', { active: activeTab === 'signin' }]"
        >
          Connexion
        </button>
        <button
          @click="activeTab = 'signup'"
          :class="['tab-btn', { active: activeTab === 'signup' }]"
        >
          Inscription
        </button>
      </div>

      <!-- Social Logins -->
      <div class="social-login">
        <button
          @click="handleGoogleLogin"
          class="btn btn-secondary btn-block btn-google"
          :disabled="loading"
        >
          <svg class="google-icon" viewBox="0 0 24 24" width="20" height="20">
            <path
              fill="#4285F4"
              d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"
            />
            <path
              fill="#34A853"
              d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"
            />
            <path
              fill="#FBBC05"
              d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.06H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.94l3.66-2.85z"
            />
            <path
              fill="#EA4335"
              d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.06l3.66 2.85c.87-2.6 3.3-4.53 6.16-4.53z"
            />
          </svg>
          Continuer avec Google
        </button>
      </div>

      <div class="or-separator">
        <span class="separator-line"></span>
        <span class="separator-text">OU</span>
        <span class="separator-line"></span>
      </div>

      <!-- Credentials Form -->
      <form @submit.prevent="handleSubmit" class="auth-form">
        <!-- Username (uniquement en inscription) -->
        <Transition name="expand">
          <div v-if="activeTab === 'signup'" class="input-group">
            <label for="username">Nom d'utilisateur</label>
            <input
              id="username"
              type="text"
              v-model="username"
              placeholder="Ex: NarutoFan99"
              required
              class="glass-input"
              :disabled="loading"
            />
          </div>
        </Transition>

        <!-- Email -->
        <div class="input-group">
          <label for="email">Adresse email</label>
          <input
            id="email"
            type="email"
            v-model="email"
            placeholder="votre@adresse.com"
            required
            class="glass-input"
            :disabled="loading"
          />
        </div>

        <!-- Password -->
        <div class="input-group">
          <label for="password">Mot de passe</label>
          <input
            id="password"
            type="password"
            v-model="password"
            placeholder="••••••••"
            required
            class="glass-input"
            :disabled="loading"
          />
        </div>

        <!-- Submit Button -->
        <button
          type="submit"
          class="btn btn-primary btn-block btn-auth"
          :disabled="loading"
        >
          <span v-if="loading" class="spinner"></span>
          <span v-else>{{
            activeTab === 'signin' ? 'Se connecter' : 'Créer un compte'
          }}</span>
        </button>
      </form>

      <!-- Feedback Messages -->
      <Transition name="fade">
        <div v-if="successMessage" class="feedback-alert success">
          <span class="alert-icon">✉️</span>
          <p>{{ successMessage }}</p>
        </div>
      </Transition>

      <Transition name="fade">
        <div v-if="errorMessage" class="feedback-alert error">
          <span class="alert-icon">⚠️</span>
          <p>{{ errorMessage }}</p>
        </div>
      </Transition>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { supabase } from '../supabase'

const activeTab = ref<'signin' | 'signup'>('signin')
const email = ref('')
const password = ref('')
const username = ref('')

const loading = ref(false)
const errorMessage = ref('')
const successMessage = ref('')

// Vider les alertes au changement d'onglet
watch(activeTab, () => {
  errorMessage.value = ''
  successMessage.value = ''
})

const handleGoogleLogin = async () => {
  loading.value = true
  errorMessage.value = ''
  successMessage.value = ''

  try {
    const { error } = await supabase.auth.signInWithOAuth({
      provider: 'google',
      options: {
        redirectTo: window.location.origin,
      },
    })

    if (error) throw error
  } catch (err: any) {
    errorMessage.value = `Erreur de connexion Google : ${err.message || err}`
    loading.value = false
  }
}

const handleSubmit = async () => {
  loading.value = true
  errorMessage.value = ''
  successMessage.value = ''

  try {
    if (activeTab.value === 'signin') {
      // Connexion
      const { error } = await supabase.auth.signInWithPassword({
        email: email.value,
        password: password.value,
      })
      if (error) throw error
    } else {
      // Inscription
      const { error } = await supabase.auth.signUp({
        email: email.value,
        password: password.value,
        options: {
          data: {
            username: username.value,
          },
        },
      })
      if (error) throw error
      successMessage.value =
        'Compte créé ! Veuillez vérifier vos e-mails pour valider votre inscription (ou connectez-vous si la confirmation mail est désactivée dans Supabase).'
    }
  } catch (err: any) {
    errorMessage.value =
      err.message || "Une erreur s'est produite lors de l'authentification."
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.auth-container {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 100vh;
  width: 100%;
  padding: 20px;
  background: radial-gradient(circle at 50% 0%, #1a1528 0%, #0d0a15 100%);
  box-sizing: border-box;
}

.auth-card {
  width: 100%;
  max-width: 440px;
  box-sizing: border-box;
  animation: slide-up 0.4s cubic-bezier(0.16, 1, 0.3, 1);
}

.auth-header {
  text-align: center;
  margin-bottom: 30px;
}

.auth-logo-icon {
  font-size: 3rem;
  display: block;
  margin-bottom: 12px;
}

.auth-title {
  font-size: 2rem;
  font-weight: 800;
  margin: 0 0 6px 0;
  background: linear-gradient(135deg, #ec4899 0%, #8b5cf6 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.auth-subtitle {
  font-size: 0.9rem;
  color: var(--text-muted);
  margin: 0;
  line-height: 1.5;
}

/* Tabs Toggle */
.tab-toggle {
  display: flex;
  background: rgba(0, 0, 0, 0.25);
  border: 1px solid var(--glass-border);
  border-radius: 12px;
  padding: 4px;
  margin-bottom: 25px;
}

.tab-btn {
  flex: 1;
  background: transparent;
  border: none;
  color: var(--text-muted);
  font-weight: 600;
  padding: 10px;
  border-radius: 9px;
  cursor: pointer;
  transition: all 0.2s ease;
  font-family: inherit;
  outline: none;
}

.tab-btn.active {
  background: rgba(255, 255, 255, 0.05);
  color: var(--text-main);
  box-shadow: 0 4px 10px rgba(0, 0, 0, 0.2);
}

/* Social Google Button */
.social-login {
  margin-bottom: 20px;
}

.btn-google {
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid var(--glass-border);
  font-size: 0.95rem;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
}

.btn-google:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(255, 255, 255, 0.2);
  transform: translateY(-1px);
}

.google-icon {
  display: inline-block;
}

/* Separator */
.or-separator {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 15px;
  margin-bottom: 20px;
}

.separator-line {
  flex: 1;
  height: 1px;
  background: rgba(255, 255, 255, 0.08);
}

.separator-text {
  font-size: 0.75rem;
  font-weight: 700;
  color: var(--text-muted);
  letter-spacing: 1px;
}

/* Form Styles */
.auth-form {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.btn-auth {
  height: 48px;
  margin-top: 10px;
}

/* Alerts */
.feedback-alert {
  margin-top: 20px;
  border-radius: 12px;
  padding: 14px;
  display: flex;
  gap: 12px;
  font-size: 0.9rem;
  line-height: 1.5;
  border: 1px solid;
}

.feedback-alert p {
  margin: 0;
}

.feedback-alert.success {
  background: rgba(16, 185, 129, 0.08);
  border-color: rgba(16, 185, 129, 0.2);
  color: #a7f3d0;
}

.feedback-alert.error {
  background: rgba(239, 68, 68, 0.08);
  border-color: rgba(239, 68, 68, 0.2);
  color: #fca5a5;
}

.alert-icon {
  font-size: 1.1rem;
}

/* Glassmorphism Panel */
.glass-panel {
  background: var(--glass-bg);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border: 1px solid var(--glass-border);
  border-radius: 24px;
  padding: 35px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.4);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.glass-panel:hover {
  border-color: rgba(139, 92, 246, 0.2);
  box-shadow: 0 15px 40px rgba(139, 92, 246, 0.15);
}

/* Inputs */
.input-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.input-group label {
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
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

.btn-block {
  display: flex;
  width: 100%;
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

/* Transitions */
.expand-enter-active,
.expand-leave-active {
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  max-height: 100px;
}

.expand-enter-from,
.expand-leave-to {
  max-height: 0;
  opacity: 0;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.25s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

@keyframes slide-up {
  0% {
    opacity: 0;
    transform: translateY(20px);
  }
  100% {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
