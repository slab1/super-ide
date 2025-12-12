<template>
  <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-gray-900 rounded-lg p-6 w-full max-w-2xl mx-4 border border-gray-700 max-h-[90vh] overflow-y-auto">
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-xl font-semibold text-white">Create Shareable Link</h2>
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
            Title *
          </label>
          <input
            v-model="form.title"
            type="text"
            required
            placeholder="My Awesome Code Snippet"
            class="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">
            Description
          </label>
          <textarea
            v-model="form.description"
            placeholder="Optional description of what this code does..."
            rows="2"
            class="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none"
          />
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">
            Code *
          </label>
          <textarea
            v-model="form.code"
            required
            placeholder="Paste your code here..."
            rows="8"
            class="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none font-mono text-sm"
          />
          <div class="flex items-center justify-between mt-2">
            <div class="flex items-center space-x-4">
              <label class="flex items-center">
                <input
                  v-model="form.syntaxHighlighted"
                  type="checkbox"
                  class="w-4 h-4 text-blue-600 bg-gray-800 border-gray-700 rounded focus:ring-blue-500 focus:ring-2"
                />
                <span class="ml-2 text-sm text-gray-300">Enable syntax highlighting</span>
              </label>
            </div>
            <span class="text-xs text-gray-500">{{ form.code.length }} characters</span>
          </div>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div>
            <label class="block text-sm font-medium text-gray-300 mb-2">
              Language
            </label>
            <select
              v-model="form.language"
              class="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            >
              <option value="javascript">JavaScript</option>
              <option value="typescript">TypeScript</option>
              <option value="python">Python</option>
              <option value="rust">Rust</option>
              <option value="go">Go</option>
              <option value="java">Java</option>
              <option value="csharp">C#</option>
              <option value="cpp">C++</option>
              <option value="php">PHP</option>
              <option value="ruby">Ruby</option>
              <option value="swift">Swift</option>
              <option value="kotlin">Kotlin</option>
              <option value="html">HTML</option>
              <option value="css">CSS</option>
              <option value="json">JSON</option>
              <option value="yaml">YAML</option>
              <option value="markdown">Markdown</option>
              <option value="text">Plain Text</option>
            </select>
          </div>

          <div>
            <label class="block text-sm font-medium text-gray-300 mb-2">
              Visibility
            </label>
            <select
              v-model="form.isPublic"
              class="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            >
              <option :value="true">Public (Anyone with link can view)</option>
              <option :value="false">Private (Only invited users)</option>
            </select>
          </div>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div>
            <label class="block text-sm font-medium text-gray-300 mb-2">
              Expiration (Optional)
            </label>
            <input
              v-model="form.expiresAt"
              type="datetime-local"
              class="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
            <p class="text-xs text-gray-500 mt-1">Leave empty for no expiration</p>
          </div>

          <div>
            <label class="block text-sm font-medium text-gray-300 mb-2">
              Password Protection (Optional)
            </label>
            <input
              v-model="form.password"
              type="password"
              placeholder="Set a password"
              class="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
            <p class="text-xs text-gray-500 mt-1">Leave empty for no password</p>
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
            :disabled="!form.title.trim() || !form.code.trim()"
            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 disabled:cursor-not-allowed text-white rounded-lg font-medium transition-colors"
          >
            Create Link
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

interface FormData {
  title: string
  description: string
  code: string
  language: string
  syntaxHighlighted: boolean
  isPublic: boolean
  expiresAt: string
  password: string
}

const emit = defineEmits<{
  close: []
  create: [data: Omit<FormData, 'expiresAt' | 'password'> & { expiresAt?: Date; password?: string }]
}>()

const form = ref<FormData>({
  title: '',
  description: '',
  code: '',
  language: 'javascript',
  syntaxHighlighted: true,
  isPublic: true,
  expiresAt: '',
  password: ''
})

function handleSubmit() {
  if (!form.value.title.trim() || !form.value.code.trim()) return

  const data = {
    title: form.value.title.trim(),
    description: form.value.description.trim() || undefined,
    code: form.value.code.trim(),
    language: form.value.language,
    syntaxHighlighted: form.value.syntaxHighlighted,
    isPublic: form.value.isPublic,
    expiresAt: form.value.expiresAt ? new Date(form.value.expiresAt) : undefined,
    password: form.value.password || undefined
  }

  emit('create', data)
}
</script>