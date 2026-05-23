<template>
  <div class="admin-container">
    <div class="glass-panel">
      <div class="panel-header">
        <span class="step-num">🔐</span>
        <h2>Administration</h2>
      </div>

      <!-- Claim Admin (first-time setup) -->
      <div v-if="!isAdmin && canClaim" class="input-group" style="margin-bottom: 20px;">
        <p style="color: var(--text-muted); margin-bottom: 12px;">
          Aucun administrateur n'existe encore. Tu peux réclamer le rôle !
        </p>
        <button @click="claimAdmin" :disabled="claiming" class="btn btn-primary btn-block">
          <span v-if="claiming" class="spinner"></span>
          <span v-else>👑 Devenir administrateur</span>
        </button>
      </div>

      <!-- Admin content -->
      <template v-if="isAdmin">
        <!-- Add admin by email -->
        <div class="input-group" style="margin-bottom: 20px;">
          <label for="admin-email-input">Ajouter un administrateur par email :</label>
          <div class="add-admin-row">
            <input
              id="admin-email-input"
              type="email"
              v-model="newAdminEmail"
              placeholder="ex: utilisateur@email.com"
              class="glass-input"
              style="flex: 1;"
            />
            <button @click="addAdmin" :disabled="!newAdminEmail.trim() || addingAdmin" class="btn btn-primary">
              <span v-if="addingAdmin" class="spinner"></span>
              <span v-else>➕ Ajouter</span>
            </button>
          </div>
          <p v-if="addError" class="error-text">{{ addError }}</p>
        </div>

        <!-- Admin list -->
        <div class="input-group">
          <label style="opacity: 0.85;">Administrateurs actuels :</label>
          <div v-if="admins.length === 0" class="empty-state">
            Aucun administrateur
          </div>
          <div v-else class="admin-list">
            <div v-for="admin in admins" :key="admin.email" class="admin-list-item">
              <div class="admin-info">
                <span class="admin-email">{{ admin.email }}</span>
                <span v-if="admin.user_id" class="admin-linked">✅ Lié</span>
                <span v-else class="admin-pending">⏳ En attente</span>
              </div>
              <button
                @click="removeAdmin(admin.email)"
                class="delete-config-btn"
                title="Retirer les droits admin"
              >
                🗑️
              </button>
            </div>
          </div>
        </div>
      </template>

      <!-- Link invited admin -->
      <div v-if="!isAdmin && !canClaim && canLink" class="input-group" style="margin-top: 20px;">
        <p style="color: var(--text-muted); margin-bottom: 12px;">
          Tu as été invité en tant qu'administrateur. Lie ton compte :
        </p>
        <button @click="linkAdmin" :disabled="linking" class="btn btn-primary btn-block">
          <span v-if="linking" class="spinner"></span>
          <span v-else>🔗 Lier mon compte admin</span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'

const API_BASE = import.meta.env.VITE_API_URL || 'http://localhost:3000/api'

interface AdminUser {
  user_id: string | null
  email: string
  role: string
  granted_at: string
}

const emit = defineEmits<{
  'admin-updated': []
}>()

const props = defineProps<{
  user: any
}>()

const admins = ref<AdminUser[]>([])
const isAdmin = ref(false)
const canClaim = ref(false)
const canLink = ref(false)
const claiming = ref(false)
const linking = ref(false)
const addingAdmin = ref(false)
const addError = ref('')
const newAdminEmail = ref('')

const fetchAdmins = async () => {
  try {
    const res = await fetch(`${API_BASE}/admin/admins`)
    if (!res.ok) throw new Error()
    const data: AdminUser[] = await res.json()
    admins.value = data
    
    const userEmail = props.user?.email
    const userId = props.user?.id
    
    isAdmin.value = data.some(
      a => (a.user_id === userId) || (a.email === userEmail)
    )
    canClaim.value = data.length === 0
    canLink.value = data.some(a => a.email === userEmail && !a.user_id)
  } catch {
    admins.value = []
    isAdmin.value = false
  }
}

const claimAdmin = async () => {
  claiming.value = true
  try {
    const res = await fetch(`${API_BASE}/admin/claim`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        user_id: props.user.id,
        email: props.user.email
      })
    })
    if (!res.ok) {
      const text = await res.text()
      alert(text)
      return
    }
    await fetchAdmins()
    emit('admin-updated')
  } catch {
    alert('Impossible de réclamer le rôle admin')
  } finally {
    claiming.value = false
  }
}

const addAdmin = async () => {
  addingAdmin.value = true
  addError.value = ''
  try {
    const res = await fetch(`${API_BASE}/admin/admins`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email: newAdminEmail.value.trim() })
    })
    if (!res.ok) {
      const text = await res.text()
      addError.value = text
      return
    }
    newAdminEmail.value = ''
    await fetchAdmins()
  } catch {
    addError.value = 'Erreur réseau'
  } finally {
    addingAdmin.value = false
  }
}

const removeAdmin = async (email: string) => {
  if (!confirm(`Retirer les droits admin de ${email} ?`)) return
  try {
    const res = await fetch(`${API_BASE}/admin/admins/${encodeURIComponent(email)}`, {
      method: 'DELETE'
    })
    if (!res.ok) throw new Error()
    await fetchAdmins()
    // If removed self
    if (email === props.user?.email) {
      isAdmin.value = false
      emit('admin-updated')
    }
  } catch {
    alert('Impossible de supprimer cet admin')
  }
}

const linkAdmin = async () => {
  linking.value = true
  try {
    const res = await fetch(`${API_BASE}/admin/link`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        user_id: props.user.id,
        email: props.user.email
      })
    })
    if (!res.ok) {
      alert('Impossible de lier le compte')
      return
    }
    await fetchAdmins()
    emit('admin-updated')
  } catch {
    alert('Erreur réseau')
  } finally {
    linking.value = false
  }
}

onMounted(fetchAdmins)
</script>

<style scoped>
.admin-container {
  max-width: 600px;
  margin: 0 auto;
}

.add-admin-row {
  display: flex;
  gap: 10px;
  align-items: center;
}

.error-text {
  color: #fca5a5;
  font-size: 0.85rem;
  margin: 6px 0 0;
}

.admin-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 8px;
}

.admin-list-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 14px;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.05);
}

.admin-info {
  display: flex;
  align-items: center;
  gap: 10px;
}

.admin-email {
  font-size: 0.9rem;
  font-weight: 500;
  color: #e2e8f0;
}

.admin-linked {
  font-size: 0.75rem;
  color: #10b981;
  font-weight: 600;
}

.admin-pending {
  font-size: 0.75rem;
  color: #fbbf24;
  font-weight: 600;
}

.empty-state {
  padding: 20px;
  text-align: center;
  color: var(--text-muted);
  font-size: 0.9rem;
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

.glass-panel {
  background: var(--glass-bg);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border: 1px solid var(--glass-border);
  border-radius: 24px;
  padding: 30px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
}

.panel-header {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 25px;
}

.panel-header h2 {
  font-size: 1.4rem;
  margin: 0;
  font-weight: 700;
  color: var(--text-main);
}

.step-num {
  font-size: 1.1rem;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
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
  color: #fff;
  box-shadow: 0 4px 15px var(--primary-glow);
}

.btn-primary:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(236, 72, 153, 0.5);
}

.btn-primary:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.btn-block {
  display: flex;
  width: 100%;
}

.spinner {
  width: 18px;
  height: 18px;
  border: 2px solid rgba(255, 255, 255, 0.2);
  border-top-color: #fff;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  display: inline-block;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
