<template>
  <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
    <div class="bg-gray-800 rounded-lg shadow-xl w-full max-w-2xl mx-4 max-h-[90vh] overflow-y-auto">
      <!-- Header -->
      <div class="flex items-center justify-between p-6 border-b border-gray-700">
        <h2 class="text-xl font-semibold text-white">
          {{ isEditing ? 'Edit Snippet' : 'Create New Snippet' }}
        </h2>
        <button
          @click="$emit('close')"
          class="text-gray-400 hover:text-white transition-colors"
        >
          <X class="w-6 h-6" />
        </button>
      </div>

      <!-- Form -->
      <form @submit.prevent="save" class="p-6 space-y-6">
        <!-- Basic Info -->
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div>
            <label class="block text-sm font-medium text-gray-300 mb-2">
              Name *
            </label>
            <input
              v-model="formData.name"
              type="text"
              required
              class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-white placeholder-gray-400 focus:outline-none focus:border-blue-500"
              placeholder="Snippet name"
            />
          </div>
          
          <div>
            <label class="block text-sm font-medium text-gray-300 mb-2">
              Category *
            </label>
            <select
              v-model="formData.category"
              required
              class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-white focus:outline-none focus:border-blue-500"
            >
              <option value="">Select category</option>
              <option value="functions">Functions</option>
              <option value="loops">Loops</option>
              <option value="conditionals">Conditionals</option>
              <option value="classes">Classes</option>
              <option value="async">Async</option>
              <option value="testing">Testing</option>
              <option value="debugging">Debugging</option>
              <option value="database">Database</option>
              <option value="api">API</option>
              <option value="utils">Utilities</option>
            </select>
          </div>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">
            Description
          </label>
          <textarea
            v-model="formData.description"
            rows="2"
            class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-white placeholder-gray-400 focus:outline-none focus:border-blue-500"
            placeholder="Brief description of what this snippet does"
          ></textarea>
        </div>

        <!-- Languages -->
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">
            Languages *
          </label>
          <div class="grid grid-cols-2 md:grid-cols-4 gap-2">
            <label
              v-for="lang in availableLanguages"
              :key="lang.value"
              class="flex items-center space-x-2 text-sm"
            >
              <input
                v-model="formData.languages"
                type="checkbox"
                :value="lang.value"
                class="rounded border-gray-600 bg-gray-700 text-blue-600 focus:ring-blue-500"
              />
              <span class="text-gray-300">{{ lang.label }}</span>
            </label>
          </div>
        </div>

        <!-- Code -->
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">
            Code *
          </label>
          <div class="relative">
            <textarea
              v-model="formData.code"
              required
              rows="12"
              class="w-full px-3 py-2 bg-gray-900 border border-gray-600 rounded text-white placeholder-gray-400 focus:outline-none focus:border-blue-500 font-mono text-sm"
              placeholder="Enter your code snippet here..."
              @input="updatePreview"
            ></textarea>
            
            <!-- Language indicator -->
            <div class="absolute top-2 right-2">
              <select
                v-model="formData.language"
                class="px-2 py-1 bg-gray-700 border border-gray-600 rounded text-xs text-gray-300 focus:outline-none focus:border-blue-500"
              >
                <option v-for="lang in availableLanguages" :key="lang.value" :value="lang.value">
                  {{ lang.label }}
                </option>
              </select>
            </div>
          </div>
        </div>

        <!-- Preview -->
        <div v-if="formData.code" class="border border-gray-600 rounded">
          <div class="flex items-center justify-between px-3 py-2 bg-gray-700 border-b border-gray-600">
            <span class="text-sm font-medium text-gray-300">Preview</span>
            <span class="text-xs text-gray-400">{{ formData.language }}</span>
          </div>
          <div class="p-3 bg-gray-900">
            <pre class="text-sm text-gray-300 font-mono whitespace-pre-wrap">{{ formData.code }}</pre>
          </div>
        </div>

        <!-- Tags -->
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">
            Tags
          </label>
          <div class="flex flex-wrap gap-2 mb-2">
            <span
              v-for="tag in formData.tags"
              :key="tag"
              class="px-2 py-1 bg-blue-600 text-white text-xs rounded flex items-center"
            >
              {{ tag }}
              <button
                type="button"
                @click="removeTag(tag)"
                class="ml-1 hover:text-blue-200"
              >
                <X class="w-3 h-3" />
              </button>
            </span>
          </div>
          <input
            v-model="newTag"
            type="text"
            placeholder="Add tag and press Enter"
            class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-white placeholder-gray-400 focus:outline-none focus:border-blue-500"
            @keydown.enter.prevent="addTag"
          />
        </div>

        <!-- Variables (Optional) -->
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">
            Variables (Optional)
          </label>
          <div class="space-y-3">
            <div
              v-for="(variable, index) in formData.variables"
              :key="index"
              class="grid grid-cols-1 md:grid-cols-3 gap-3 p-3 bg-gray-700 rounded"
            >
              <input
                v-model="variable.name"
                type="text"
                placeholder="Variable name"
                class="px-3 py-2 bg-gray-600 border border-gray-500 rounded text-white placeholder-gray-400 focus:outline-none focus:border-blue-500"
              />
              <input
                v-model="variable.description"
                type="text"
                placeholder="Description"
                class="px-3 py-2 bg-gray-600 border border-gray-500 rounded text-white placeholder-gray-400 focus:outline-none focus:border-blue-500"
              />
              <div class="flex items-center space-x-2">
                <input
                  v-model="variable.defaultValue"
                  type="text"
                  placeholder="Default value"
                  class="flex-1 px-3 py-2 bg-gray-600 border border-gray-500 rounded text-white placeholder-gray-400 focus:outline-none focus:border-blue-500"
                />
                <button
                  type="button"
                  @click="removeVariable(index)"
                  class="p-2 text-red-400 hover:text-red-300"
                >
                  <X class="w-4 h-4" />
                </button>
              </div>
            </div>
          </div>
          
          <button
            type="button"
            @click="addVariable"
            class="mt-2 px-3 py-2 bg-gray-700 border border-gray-600 rounded text-gray-300 hover:bg-gray-600 transition-colors text-sm"
          >
            + Add Variable
          </button>
        </div>

        <!-- Actions -->
        <div class="flex items-center justify-end space-x-3 pt-6 border-t border-gray-700">
          <button
            type="button"
            @click="$emit('close')"
            class="px-4 py-2 text-gray-300 hover:text-white transition-colors"
          >
            Cancel
          </button>
          <button
            type="submit"
            :disabled="!isValid"
            class="px-6 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {{ isEditing ? 'Update' : 'Create' }} Snippet
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { X } from 'lucide-vue-next'
import type { Snippet, SnippetVariable } from '../types'

interface Props {
  snippet?: Snippet | null
}

interface Emits {
  (e: 'close'): void
  (e: 'save', snippet: Snippet): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

// Form data
const formData = ref({
  name: '',
  description: '',
  code: '',
  language: 'javascript',
  languages: [] as string[],
  category: '',
  tags: [] as string[],
  variables: [] as SnippetVariable[]
})

const newTag = ref('')

// Available languages
const availableLanguages = [
  { value: 'javascript', label: 'JavaScript' },
  { value: 'typescript', label: 'TypeScript' },
  { value: 'python', label: 'Python' },
  { value: 'rust', label: 'Rust' },
  { value: 'java', label: 'Java' },
  { value: 'cpp', label: 'C++' },
  { value: 'csharp', label: 'C#' },
  { value: 'go', label: 'Go' },
  { value: 'php', label: 'PHP' },
  { value: 'ruby', label: 'Ruby' },
  { value: 'swift', label: 'Swift' },
  { value: 'kotlin', label: 'Kotlin' }
]

// Computed
const isEditing = computed(() => !!props.snippet)

const isValid = computed(() => {
  return formData.value.name.trim() !== '' &&
         formData.value.code.trim() !== '' &&
         formData.value.languages.length > 0 &&
         formData.value.category !== ''
})

// Methods
const addTag = () => {
  const tag = newTag.value.trim()
  if (tag && !formData.value.tags.includes(tag)) {
    formData.value.tags.push(tag)
    newTag.value = ''
  }
}

const removeTag = (tag: string) => {
  const index = formData.value.tags.indexOf(tag)
  if (index > -1) {
    formData.value.tags.splice(index, 1)
  }
}

const addVariable = () => {
  formData.value.variables.push({
    name: '',
    description: '',
    defaultValue: ''
  })
}

const removeVariable = (index: number) => {
  formData.value.variables.splice(index, 1)
}

const updatePreview = () => {
  // Update preview when code changes
  // This could trigger syntax highlighting or other preview features
}

const save = () => {
  if (!isValid.value) return

  const snippet: Snippet = {
    id: props.snippet?.id || Date.now().toString(),
    name: formData.value.name.trim(),
    description: formData.value.description.trim(),
    code: formData.value.code.trim(),
    language: formData.value.language,
    languages: formData.value.languages,
    category: formData.value.category,
    tags: formData.value.tags,
    variables: formData.value.variables.length > 0 ? formData.value.variables : undefined,
    favorite: props.snippet?.favorite || false,
    usageCount: props.snippet?.usageCount || 0,
    lastUsed: props.snippet?.lastUsed || '',
    createdAt: props.snippet?.createdAt || new Date().toISOString(),
    updatedAt: new Date().toISOString()
  }

  emit('save', snippet)
}

// Initialize form
const initializeForm = () => {
  if (props.snippet) {
    formData.value = {
      name: props.snippet.name,
      description: props.snippet.description,
      code: props.snippet.code,
      language: props.snippet.language,
      languages: [...props.snippet.languages],
      category: props.snippet.category,
      tags: [...props.snippet.tags],
      variables: props.snippet.variables ? [...props.snippet.variables] : []
    }
  } else {
    // Reset form for new snippet
    formData.value = {
      name: '',
      description: '',
      code: '',
      language: 'javascript',
      languages: [],
      category: '',
      tags: [],
      variables: []
    }
  }
}

// Watch for prop changes
watch(() => props.snippet, initializeForm, { immediate: true })

// Initialize on mount
onMounted(() => {
  initializeForm()
})
</script>

<style scoped>
/* Custom scrollbar for modal */
::-webkit-scrollbar {
  width: 8px;
}

::-webkit-scrollbar-track {
  background: #374151;
  border-radius: 4px;
}

::-webkit-scrollbar-thumb {
  background: #6B7280;
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: #9CA3AF;
}

/* Focus styles */
input:focus,
textarea:focus,
select:focus {
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.5);
}

/* Animation for modal */
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
</style>