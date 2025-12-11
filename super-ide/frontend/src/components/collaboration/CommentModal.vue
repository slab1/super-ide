<template>
  <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-gray-900 rounded-lg p-6 w-full max-w-lg mx-4 border border-gray-700">
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-xl font-semibold text-white">Add Code Comment</h2>
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
        <!-- File and line info (read-only, would be populated from current editor state) -->
        <div class="bg-gray-800 rounded-lg p-3">
          <div class="flex items-center justify-between">
            <div class="flex items-center space-x-2">
              <svg class="w-4 h-4 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
              <span class="text-sm text-gray-300">{{ form.filePath }}</span>
            </div>
            <span class="text-sm text-blue-400">Line {{ form.lineNumber }}</span>
          </div>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">
            Comment *
          </label>
          <textarea
            v-model="form.content"
            required
            placeholder="What would you like to discuss about this code?"
            rows="4"
            class="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none"
          />
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div>
            <label class="block text-sm font-medium text-gray-300 mb-2">
              Priority
            </label>
            <select
              v-model="form.priority"
              class="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            >
              <option value="low">Low</option>
              <option value="normal">Normal</option>
              <option value="high">High</option>
            </select>
          </div>

          <div>
            <label class="block text-sm font-medium text-gray-300 mb-2">
              Tags
            </label>
            <input
              v-model="tagInput"
              @keydown.enter.prevent="addTag"
              @keydown.comma.prevent="addTag"
              placeholder="bug, enhancement, question"
              class="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
            <p class="text-xs text-gray-500 mt-1">Press Enter or comma to add tags</p>
          </div>
        </div>

        <!-- Tag display -->
        <div v-if="form.tags.length > 0" class="flex flex-wrap gap-2">
          <span
            v-for="tag in form.tags"
            :key="tag"
            class="inline-flex items-center px-2 py-1 bg-blue-900 text-blue-200 rounded text-xs"
          >
            {{ tag }}
            <button
              @click="removeTag(tag)"
              type="button"
              class="ml-1 text-blue-300 hover:text-blue-100 transition-colors"
            >
              <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </span>
        </div>

        <!-- Suggested actions -->
        <div class="bg-gray-800 rounded-lg p-3">
          <h3 class="text-sm font-medium text-gray-300 mb-2">Quick Actions:</h3>
          <div class="grid grid-cols-2 gap-2">
            <button
              v-for="action in quickActions"
              :key="action.text"
              @click="applyQuickAction(action)"
              type="button"
              class="text-left px-2 py-1 text-xs bg-gray-700 hover:bg-gray-600 rounded transition-colors"
            >
              <div class="text-white font-medium">{{ action.text }}</div>
              <div class="text-gray-400">{{ action.description }}</div>
            </button>
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
            :disabled="!form.content.trim()"
            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 disabled:cursor-not-allowed text-white rounded-lg font-medium transition-colors"
          >
            Add Comment
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

interface FormData {
  filePath: string
  lineNumber: number
  content: string
  tags: string[]
  priority: 'low' | 'normal' | 'high'
}

const emit = defineEmits<{
  close: []
  submit: [data: FormData]
}>()

const props = defineProps<{
  filePath?: string
  lineNumber?: number
}>()

const form = ref<FormData>({
  filePath: props.filePath || '/src/components/App.vue',
  lineNumber: props.lineNumber || 1,
  content: '',
  tags: [],
  priority: 'normal'
})

const tagInput = ref('')

const quickActions = [
  {
    text: 'Bug Report',
    description: 'This code has a bug',
    content: 'I found a potential bug here. This might cause issues with '
  },
  {
    text: 'Enhancement',
    description: 'Could be improved',
    content: 'This could be enhanced by '
  },
  {
    text: 'Question',
    description: 'Need clarification',
    content: 'Could you clarify why this approach was chosen? '
  },
  {
    text: 'Documentation',
    description: 'Needs better docs',
    content: 'This code would benefit from more documentation explaining '
  }
]

function addTag() {
  const tag = tagInput.value.trim().replace(/,$/, '')
  if (tag && !form.value.tags.includes(tag)) {
    form.value.tags.push(tag)
  }
  tagInput.value = ''
}

function removeTag(tag: string) {
  const index = form.value.tags.indexOf(tag)
  if (index !== -1) {
    form.value.tags.splice(index, 1)
  }
}

function applyQuickAction(action: any) {
  form.value.content = action.content
}

function handleSubmit() {
  if (!form.value.content.trim()) return
  
  emit('submit', {
    ...form.value,
    content: form.value.content.trim()
  })
}
</script>