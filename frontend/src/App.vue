<template>
  <main class="app-main">
    <Transition name="fade" mode="out-in">
      <AiPlayground v-if="session" />
      <AuthPage v-else />
    </Transition>
  </main>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { supabase } from './supabase'
import AuthPage from './components/AuthPage.vue'
import AiPlayground from './components/AiPlayground.vue'

const session = ref<any>(null)

onMounted(() => {
  // Récupère la session actuelle au démarrage
  supabase.auth.getSession().then(({ data }) => {
    session.value = data.session
  })

  // Écoute les changements d'état (connexion, déconnexion, expiration de token...)
  supabase.auth.onAuthStateChange((_event, _session) => {
    session.value = _session
  })
})
</script>

<style scoped>
.app-main {
  min-height: 100vh;
  width: 100%;
  display: flex;
  flex-direction: column;
}

/* Animations de transition fluides */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.25s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
