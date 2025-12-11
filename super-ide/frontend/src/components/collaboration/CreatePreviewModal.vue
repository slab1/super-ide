<template>
  <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-gray-900 rounded-lg p-6 w-full max-w-md mx-4 border border-gray-700">
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-xl font-semibold text-white">Create Live Preview</h2>
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
            Preview Name *
          </label>
          <input
            v-model="form.name"
            type="text"
            required
            placeholder="My Web Application"
            class="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">
            Application URL *
          </label>
          <input
            v-model="form.url"
            type="url"
            required
            placeholder="http://localhost:3000 or https://myapp.com"
            class="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
          <p class="text-xs text-gray-500 mt-1">
            Enter the URL of your running application
          </p>
        </div>

        <div class="flex items-center">
          <input
            v-model="form.isPublic"
            type="checkbox"
            id="isPublic"
            class="w-4 h-4 text-blue-600 bg-gray-800 border-gray-700 rounded focus:ring-blue-500 focus:ring-2"
          />
          <label for="isPublic" class="ml-2 text-sm text-gray-300">
            Make this preview public (anyone with the link can view)
          </label>
        </div>

        <!-- Quick URL examples -->
        <div class="bg-gray-800 rounded-lg p-3">
          <h3 class="text-sm font-medium text-gray-300 mb-2">Quick Examples:</h3>
          <div class="space-y-2">
            <button
              v-for="example in examples"
              :key="example.url"
              @click="form.url = example.url"
              type="button"
              class="w-full text-left px-2 py-1 text-xs bg-gray-700 hover:bg-gray-600 rounded transition-colors"
            >
              <div class="text-white font-medium">{{ example.name }}</div>
              <div class="text-gray-400">{{ example.url }}</div>
            </button>
          </div>
        </div>

        <div class="bg-blue-900/20 border border-blue-700 rounded-lg p-3">
          <div class="flex items-start space-x-2">
            <svg class="w-5 h-5 text-blue-400 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <div>
              <h4 class="text-sm font-medium text-blue-300">Live Preview Info</h4>
              <p class="text-xs text-blue-200 mt-1">
                Your application must be running and accessible via the provided URL. 
                Collaborators will see the real-time state of your application.
              </p>
            </div>
          </div>
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
            :disabled="!form.name.trim() || !form.url.trim()"
            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 disabled:cursor-not-allowed text-white rounded-lg font-medium transition-colors"
          >
            Create Preview
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
  url: string
  isPublic: boolean
}

const emit = defineEmits<{
  close: []
  create: [data: FormData]
}>()

const form = ref<FormData>({
  name: '',
  url: '',
  isPublic: false
})

const examples = [
  {
    name: 'Local Development Server',
    url: 'http://localhost:3000'
  },
  {
    name: 'Next.js/Vite Dev Server',
    url: 'http://localhost:5173'
  },
  {
    name: 'React Native Metro',
    url: 'http://localhost:8081'
  },
  {
    name: 'Django Development',
    url: 'http://localhost:8000'
  },
  {
    name: 'Flask Development',
    url: 'http://localhost:5000'
  },
  {
    name: 'Node.js Express',
    url: 'http://localhost:3001'
  }
]

function handleSubmit() {
  if (!form.value.name.trim() || !form.value.url.trim()) return
  
  emit('create', {
    ...form.value,
    name: form.value.name.trim(),
    url: form.value.url.trim()
  })
}
</script>