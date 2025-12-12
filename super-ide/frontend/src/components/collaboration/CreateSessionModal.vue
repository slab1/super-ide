<template>
  <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-gray-900 rounded-lg p-6 w-full max-w-md mx-4 border border-gray-700">
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-xl font-semibold text-white">Create Collaboration Session</h2>
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
            Session Name *
          </label>
          <input
            v-model="form.name"
            type="text"
            required
            placeholder="My Awesome Project"
            class="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">
            Description
          </label>
          <textarea
            v-model="form.description"
            placeholder="Optional description of what you'll be working on..."
            rows="3"
            class="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none"
          />
        </div>

        <div class="flex items-center">
          <input
            v-model="form.isPublic"
            type="checkbox"
            id="isPublic"
            class="w-4 h-4 text-blue-600 bg-gray-800 border-gray-700 rounded focus:ring-blue-500 focus:ring-2"
          />
          <label for="isPublic" class="ml-2 text-sm text-gray-300">
            Make this session public (anyone with the link can join)
          </label>
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
            :disabled="!form.name.trim()"
            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 disabled:cursor-not-allowed text-white rounded-lg font-medium transition-colors"
          >
            Create Session
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

interface FormData {
  name: string
  description: string
  isPublic: boolean
}

const emit = defineEmits<{
  close: []
  create: [data: FormData]
}>()

const form = ref<FormData>({
  name: '',
  description: '',
  isPublic: false
})

function handleSubmit() {
  if (!form.value.name.trim()) return
  
  emit('create', {
    ...form.value,
    name: form.value.name.trim()
  })
}
</script>