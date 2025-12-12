<template>
  <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-gray-900 rounded-lg p-6 w-full max-w-md mx-4 border border-gray-700">
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-xl font-semibold text-white">Join Collaboration Session</h2>
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
            Session ID or Link *
          </label>
          <input
            v-model="sessionId"
            type="text"
            required
            placeholder="Enter session ID or paste invite link"
            class="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
          <p class="text-xs text-gray-500 mt-1">
            You can paste a full invite link or just the session ID
          </p>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">
            Your Display Name
          </label>
          <input
            v-model="displayName"
            type="text"
            placeholder="Your name"
            class="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
        </div>

        <div class="flex items-center justify-end space-x-3 pt-4">
          <button
            type="button"
            @click="$emit('close')"
            class="px-4 py-2 text-gray-300 hover:text-white transition-colors"
          >
            Cancel
          </button>
          <button
            type="submit"
            :disabled="!sessionId.trim()"
            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 disabled:cursor-not-allowed text-white rounded-lg font-medium transition-colors"
          >
            Join Session
          </button>
        </div>
      </form>

      <!-- Quick join examples -->
      <div class="mt-6 pt-4 border-t border-gray-700">
        <h3 class="text-sm font-medium text-gray-300 mb-2">Quick Join Examples:</h3>
        <div class="space-y-2">
          <button
            v-for="example in examples"
            :key="example.id"
            @click="sessionId = example.id"
            class="w-full text-left px-3 py-2 bg-gray-800 hover:bg-gray-700 rounded border border-gray-700 transition-colors"
          >
            <div class="text-sm font-medium text-white">{{ example.name }}</div>
            <div class="text-xs text-gray-400">{{ example.description }}</div>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const emit = defineEmits<{
  close: []
  join: [sessionId: string]
}>()

const sessionId = ref('')
const displayName = ref('')

const examples = [
  {
    id: 'demo-session-123',
    name: 'Demo Session',
    description: 'A sample collaboration session for testing'
  },
  {
    id: 'tutorial-room',
    name: 'Learning Tutorial',
    description: 'Interactive coding tutorial session'
  },
  {
    id: 'team-project',
    name: 'Team Project',
    description: 'Collaborative development session'
  }
]

function extractSessionId(input: string): string {
  // Extract session ID from full URL or use input directly
  const urlMatch = input.match(/\/collaborate\/([a-zA-Z0-9-_]+)/)
  if (urlMatch) {
    return urlMatch[1]
  }
  return input.trim()
}

function handleSubmit() {
  if (!sessionId.value.trim()) return
  
  const extractedId = extractSessionId(sessionId.value)
  emit('join', extractedId)
}
</script>