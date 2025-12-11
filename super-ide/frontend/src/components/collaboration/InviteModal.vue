<template>
  <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-gray-900 rounded-lg p-6 w-full max-w-lg mx-4 border border-gray-700">
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-xl font-semibold text-white">Invite Collaborators</h2>
        <button
          @click="$emit('close')"
          class="text-gray-400 hover:text-gray-300 transition-colors"
        >
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <form @submit.prevent="handleSubmit" class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">
            Email Addresses *
          </label>
          <div class="relative">
            <input
              v-model="emailInput"
              @keydown.enter.prevent="addEmail"
              @keydown.comma.prevent="addEmail"
              type="email"
              placeholder="colleague@example.com, teammate@company.com"
              class="w-full px-3 py-2 pr-20 bg-gray-800 border border-gray-700 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
            <button
              type="button"
              @click="addEmail"
              class="absolute right-2 top-1/2 transform -translate-y-1/2 px-2 py-1 bg-blue-600 hover:bg-blue-700 text-white text-xs rounded transition-colors"
            >
              Add
            </button>
          </div>
          <p class="text-xs text-gray-500 mt-1">Press Enter or comma to add multiple emails</p>
        </div>

        <!-- Email list -->
        <div v-if="emails.length > 0" class="space-y-2">
          <div
            v-for="email in emails"
            :key="email"
            class="flex items-center justify-between px-3 py-2 bg-gray-800 rounded-lg"
          >
            <span class="text-sm text-white">{{ email }}</span>
            <button
              @click="removeEmail(email)"
              type="button"
              class="text-red-400 hover:text-red-300 transition-colors"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">
            Message (Optional)
          </label>
          <textarea
            v-model="message"
            placeholder="Add a personal message to your invitation..."
            rows="3"
            class="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none"
          />
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">
            Permission Level
          </label>
          <select
            v-model="permissionLevel"
            class="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          >
            <option value="viewer">Viewer - Can view and comment</option>
            <option value="editor">Editor - Can view, comment, and edit</option>
            <option value="admin">Admin - Full access including inviting others</option>
          </select>
        </div>

        <!-- Quick invite templates -->
        <div class="bg-gray-800 rounded-lg p-3">
          <h3 class="text-sm font-medium text-gray-300 mb-2">Quick Templates:</h3>
          <div class="space-y-2">
            <button
              v-for="template in inviteTemplates"
              :key="template.title"
              @click="applyTemplate(template)"
              type="button"
              class="w-full text-left px-3 py-2 bg-gray-700 hover:bg-gray-600 rounded text-sm transition-colors"
            >
              <div class="text-white font-medium">{{ template.title }}</div>
              <div class="text-gray-400 text-xs">{{ template.description }}</div>
            </button>
          </div>
        </div>

        <!-- Share link fallback -->
        <div class="bg-blue-900/20 border border-blue-700 rounded-lg p-3">
          <div class="flex items-start justify-between">
            <div class="flex-1">
              <h4 class="text-sm font-medium text-blue-300">Share Link Instead</h4>
              <p class="text-xs text-blue-200 mt-1">
                Alternatively, you can share this session link directly:
              </p>
              <div class="mt-2 flex space-x-2">
                <code class="text-xs bg-blue-900 px-2 py-1 rounded text-blue-200 flex-1">
                  {{ sessionShareUrl }}
                </code>
                <button
                  @click="copySessionUrl"
                  type="button"
                  class="px-2 py-1 bg-blue-600 hover:bg-blue-700 text-white text-xs rounded transition-colors"
                >
                  Copy
                </button>
              </div>
            </div>
          </div>
        </div>

        <div class="flex items-center justify-end space-x-3 pt-4 border-t border-gray-700">
          <button
            type="button"
            @click="$emit('close')"
            class="px-4 py-2 text-gray-300 hover:text-white transition-colors"
          >
            Cancel
          </button>
          <button
            type="submit"
            :disabled="emails.length === 0"
            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 disabled:cursor-not-allowed text-white rounded-lg font-medium transition-colors"
          >
            Send Invitations
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

const emit = defineEmits<{
  close: []
  invite: [emails: string[]]
}>()

const props = defineProps<{
  sessionId?: string
}>()

const emailInput = ref('')
const emails = ref<string[]>([])
const message = ref('')
const permissionLevel = ref('editor')

const sessionShareUrl = computed(() => {
  return props.sessionId ? `https://super-ide.dev/collaborate/${props.sessionId}` : 'Session link will be generated'
})

const inviteTemplates = [
  {
    title: 'Code Review Request',
    description: 'Please review this code and provide feedback',
    message: 'Hi! I\'d love your feedback on this code. Could you take a look and let me know what you think?'
  },
  {
    title: 'Pair Programming Session',
    description: 'Join me for a pair programming session',
    message: 'Want to pair program on this project together? It\'ll be fun and productive!'
  },
  {
    title: 'Learning Session',
    description: 'Join me for a learning session',
    message: 'I\'m working on learning something new. Would you like to join me and we can learn together?'
  },
  {
    title: 'Project Collaboration',
    description: 'Work together on this project',
    message: 'I\'m starting a new project and would love to have you as a collaborator. Let\'s build something amazing together!'
  }
]

function addEmail() {
  const email = emailInput.value.trim().replace(/,$/, '')
  if (email && isValidEmail(email) && !emails.value.includes(email)) {
    emails.value.push(email)
  }
  emailInput.value = ''
}

function removeEmail(email: string) {
  const index = emails.value.indexOf(email)
  if (index !== -1) {
    emails.value.splice(index, 1)
  }
}

function isValidEmail(email: string): boolean {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
  return emailRegex.test(email)
}

function applyTemplate(template: any) {
  message.value = template.message
}

function copySessionUrl() {
  navigator.clipboard.writeText(sessionShareUrl.value)
  // Show toast notification in real implementation
}

function handleSubmit() {
  if (emails.value.length === 0) return
  
  emit('invite', [...emails.value])
}
</script>