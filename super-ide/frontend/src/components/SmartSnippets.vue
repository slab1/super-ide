<template>
  <div class="smart-snippets-panel">
    <!-- Header -->
    <div class="flex items-center justify-between p-3 border-b border-gray-700">
      <h3 class="text-sm font-medium text-gray-200">Smart Snippets</h3>
      <div class="flex items-center space-x-2">
        <button
          @click="toggleSearch"
          :class="[
            'p-1 rounded text-gray-400 hover:text-gray-200',
            searchVisible ? 'bg-gray-700' : ''
          ]"
          title="Search snippets"
        >
          <Search class="w-4 h-4" />
        </button>
        <button
          @click="addSnippet"
          class="p-1 rounded text-gray-400 hover:text-gray-200"
          title="Add new snippet"
        >
          <Plus class="w-4 h-4" />
        </button>
      </div>
    </div>

    <!-- Search Bar -->
    <div v-if="searchVisible" class="p-3 border-b border-gray-700">
      <input
        v-model="searchQuery"
        type="text"
        placeholder="Search snippets..."
        class="w-full px-3 py-2 bg-gray-800 border border-gray-600 rounded text-sm text-gray-200 placeholder-gray-400 focus:outline-none focus:border-blue-500"
        ref="searchInput"
      />
    </div>

    <!-- Snippet Categories -->
    <div class="flex border-b border-gray-700">
      <button
        v-for="category in categories"
        :key="category.id"
        @click="selectedCategory = category.id"
        :class="[
          'flex-1 px-3 py-2 text-xs font-medium transition-colors',
          selectedCategory === category.id
            ? 'bg-blue-600 text-white'
            : 'text-gray-400 hover:text-gray-200 hover:bg-gray-800'
        ]"
      >
        <component :is="category.icon" class="w-3 h-3 inline mr-1" />
        {{ category.name }}
      </button>
    </div>

    <!-- Snippets List -->
    <div class="flex-1 overflow-y-auto">
      <div
        v-for="snippet in filteredSnippets"
        :key="snippet.id"
        @click="insertSnippet(snippet)"
        class="p-3 border-b border-gray-700 hover:bg-gray-800 cursor-pointer transition-colors"
      >
        <div class="flex items-start justify-between">
          <div class="flex-1">
            <h4 class="text-sm font-medium text-gray-200 mb-1">{{ snippet.name }}</h4>
            <p class="text-xs text-gray-400 mb-2">{{ snippet.description }}</p>
            <div class="flex items-center space-x-2">
              <span
                v-for="lang in snippet.languages"
                :key="lang"
                class="px-2 py-1 bg-gray-700 text-gray-300 text-xs rounded"
              >
                {{ lang }}
              </span>
            </div>
          </div>
          <div class="flex items-center space-x-1">
            <button
              @click.stop="favoriteSnippet(snippet.id)"
              :class="[
                'p-1 rounded',
                snippet.favorite ? 'text-yellow-400' : 'text-gray-500 hover:text-gray-300'
              ]"
              title="Toggle favorite"
            >
              <Star class="w-3 h-3" />
            </button>
          </div>
        </div>
        
        <!-- Preview -->
        <div class="mt-2 p-2 bg-gray-900 rounded text-xs text-gray-300 font-mono">
          {{ snippet.code }}
        </div>
        
        <!-- Usage Context -->
        <div class="mt-2 flex items-center justify-between text-xs text-gray-500">
          <span>Used {{ snippet.usageCount }} times</span>
          <span>{{ snippet.lastUsed }}</span>
        </div>
      </div>

      <!-- Empty State -->
      <div v-if="filteredSnippets.length === 0" class="p-6 text-center text-gray-500">
        <Code class="w-8 h-8 mx-auto mb-2 opacity-50" />
        <p class="text-sm">No snippets found</p>
        <p class="text-xs">Try a different search or category</p>
      </div>
    </div>

    <!-- Snippet Editor Modal -->
    <SnippetEditorModal
      v-if="showEditor"
      :snippet="editingSnippet"
      @close="closeEditor"
      @save="saveSnippet"
    />

    <!-- Context Menu -->
    <div
      v-if="contextMenu.visible"
      :style="{ top: contextMenu.y + 'px', left: contextMenu.x + 'px' }"
      class="fixed z-50 bg-gray-800 border border-gray-600 rounded shadow-lg py-1 min-w-48"
      @click.stop
    >
      <button
        @click="copySnippet(contextMenu.snippet)"
        class="w-full px-3 py-2 text-left text-sm text-gray-200 hover:bg-gray-700 flex items-center"
      >
        <Copy class="w-4 h-4 mr-2" />
        Copy Code
      </button>
      <button
        @click="editSnippet(contextMenu.snippet)"
        class="w-full px-3 py-2 text-left text-sm text-gray-200 hover:bg-gray-700 flex items-center"
      >
        <Edit class="w-4 h-4 mr-2" />
        Edit
      </button>
      <button
        @click="duplicateSnippet(contextMenu.snippet)"
        class="w-full px-3 py-2 text-left text-sm text-gray-200 hover:bg-gray-700 flex items-center"
      >
        <Copy class="w-4 h-4 mr-2" />
        Duplicate
      </button>
      <hr class="border-gray-600 my-1" />
      <button
        @click="deleteSnippet(contextMenu.snippet)"
        class="w-full px-3 py-2 text-left text-sm text-red-400 hover:bg-gray-700 flex items-center"
      >
        <Trash class="w-4 h-4 mr-2" />
        Delete
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { Search, Plus, Star, Code, Copy, Edit, Trash } from 'lucide-vue-next'
import { useSnippetsStore } from '../stores/snippetsStore'
import { useEditorStore } from '../stores/editorStore'
import SnippetEditorModal from './SnippetEditorModal.vue'
import type { Snippet, SnippetCategory } from '../types'

// Stores
const snippetsStore = useSnippetsStore()
const editorStore = useEditorStore()

// Component state
const searchVisible = ref(false)
const searchQuery = ref('')
const selectedCategory = ref('all')
const showEditor = ref(false)
const editingSnippet = ref<Snippet | null>(null)
const contextMenu = ref({
  visible: false,
  x: 0,
  y: 0,
  snippet: null as Snippet | null
})

// Categories
const categories = ref<SnippetCategory[]>([
  { id: 'all', name: 'All', icon: 'Code' },
  { id: 'functions', name: 'Functions', icon: 'Zap' },
  { id: 'loops', name: 'Loops', icon: 'RotateCw' },
  { id: 'conditionals', name: 'Conditionals', icon: 'GitBranch' },
  { id: 'classes', name: 'Classes', icon: 'Package' },
  { id: 'async', name: 'Async', icon: 'Clock' },
  { id: 'testing', name: 'Testing', icon: 'TestTube' },
  { id: 'debugging', name: 'Debugging', icon: 'Bug' },
  { id: 'database', name: 'Database', icon: 'Database' },
  { id: 'api', name: 'API', icon: 'Globe' },
  { id: 'utils', name: 'Utilities', icon: 'Tool' },
  { id: 'favorites', name: 'Favorites', icon: 'Star' }
])

// Computed
const filteredSnippets = computed(() => {
  let snippets = snippetsStore.getSnippets()

  // Filter by category
  if (selectedCategory.value !== 'all') {
    if (selectedCategory.value === 'favorites') {
      snippets = snippets.filter(s => s.favorite)
    } else {
      snippets = snippets.filter(s => s.category === selectedCategory.value)
    }
  }

  // Filter by search query
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    snippets = snippets.filter(s =>
      s.name.toLowerCase().includes(query) ||
      s.description.toLowerCase().includes(query) ||
      s.code.toLowerCase().includes(query) ||
      s.tags.some(tag => tag.toLowerCase().includes(query))
    )
  }

  return snippets.sort((a, b) => {
    // Sort by favorite first, then by usage count
    if (a.favorite && !b.favorite) return -1
    if (!a.favorite && b.favorite) return 1
    return b.usageCount - a.usageCount
  })
})

// Methods
const toggleSearch = () => {
  searchVisible.value = !searchVisible.value
  if (searchVisible.value) {
    nextTick(() => {
      searchInput.value?.focus()
    })
  }
}

const addSnippet = () => {
  editingSnippet.value = null
  showEditor.value = true
}

const insertSnippet = async (snippet: Snippet) => {
  try {
    // Track usage
    snippetsStore.incrementUsage(snippet.id)
    
    // Insert into editor
    await editorStore.insertText(snippet.code)
    
    // Update last used
    snippetsStore.updateLastUsed(snippet.id)
    
    // If it's a template with variables, open variable input
    if (snippet.variables && snippet.variables.length > 0) {
      openVariableEditor(snippet)
    }
  } catch (error) {
    console.error('Failed to insert snippet:', error)
  }
}

const favoriteSnippet = (snippetId: string) => {
  snippetsStore.toggleFavorite(snippetId)
}

const editSnippet = (snippet: Snippet) => {
  editingSnippet.value = snippet
  showEditor.value = true
  hideContextMenu()
}

const duplicateSnippet = (snippet: Snippet) => {
  const duplicated = { ...snippet }
  duplicated.id = Date.now().toString()
  duplicated.name = `${snippet.name} (Copy)`
  duplicated.usageCount = 0
  duplicated.lastUsed = ''
  snippetsStore.addSnippet(duplicated)
  hideContextMenu()
}

const copySnippet = (snippet: Snippet) => {
  navigator.clipboard.writeText(snippet.code)
  hideContextMenu()
}

const deleteSnippet = (snippet: Snippet) => {
  if (confirm(`Are you sure you want to delete "${snippet.name}"?`)) {
    snippetsStore.removeSnippet(snippet.id)
    hideContextMenu()
  }
}

const closeEditor = () => {
  showEditor.value = false
  editingSnippet.value = null
}

const saveSnippet = (snippet: Snippet) => {
  if (editingSnippet.value) {
    snippetsStore.updateSnippet(snippet)
  } else {
    snippetsStore.addSnippet(snippet)
  }
  closeEditor()
}

const openVariableEditor = (snippet: Snippet) => {
  // Implementation for variable replacement
  // This would open a modal to input variable values
  console.log('Opening variable editor for snippet:', snippet.name)
}

const showContextMenu = (event: MouseEvent, snippet: Snippet) => {
  event.preventDefault()
  contextMenu.value = {
    visible: true,
    x: event.clientX,
    y: event.clientY,
    snippet
  }
}

const hideContextMenu = () => {
  contextMenu.value.visible = false
}

// Event handlers
const handleClickOutside = (event: MouseEvent) => {
  if (!event.target?.closest('.smart-snippets-panel')) {
    hideContextMenu()
  }
}

const handleKeyDown = (event: KeyboardEvent) => {
  if (event.key === 'Escape') {
    hideContextMenu()
    if (searchVisible.value) {
      searchVisible.value = false
    }
  }
}

// Lifecycle
onMounted(() => {
  document.addEventListener('click', handleClickOutside)
  document.addEventListener('keydown', handleKeyDown)
  
  // Load snippets
  snippetsStore.loadSnippets()
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
  document.removeEventListener('keydown', handleKeyDown)
})

// Template refs
const searchInput = ref<HTMLInputElement>()

// Context menu event listeners
const handleContextMenu = (event: MouseEvent, snippet: Snippet) => {
  showContextMenu(event, snippet)
}
</script>

<style scoped>
.smart-snippets-panel {
  @apply flex flex-col h-full bg-gray-800 text-white;
}

.smart-snippets-panel ::-webkit-scrollbar {
  width: 6px;
}

.smart-snippets-panel ::-webkit-scrollbar-track {
  background: #374151;
}

.smart-snippets-panel ::-webkit-scrollbar-thumb {
  background: #6B7280;
  border-radius: 3px;
}

.smart-snippets-panel ::-webkit-scrollbar-thumb:hover {
  background: #9CA3AF;
}
</style>