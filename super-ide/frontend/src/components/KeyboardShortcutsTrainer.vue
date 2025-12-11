<template>
  <div class="keyboard-shortcuts-trainer">
    <!-- Header -->
    <div class="flex items-center justify-between p-3 border-b border-gray-700">
      <h3 class="text-sm font-medium text-gray-200">Keyboard Shortcuts</h3>
      <div class="flex items-center space-x-2">
        <button
          @click="toggleTrainerMode"
          :class="[
            'px-3 py-1 rounded text-xs font-medium transition-colors',
            isTrainingMode
              ? 'bg-red-600 text-white'
              : 'bg-blue-600 text-white hover:bg-blue-700'
          ]"
        >
          {{ isTrainingMode ? 'Stop Training' : 'Start Training' }}
        </button>
        <button
          @click="showPracticeMode = !showPracticeMode"
          class="px-3 py-1 bg-gray-700 text-gray-300 rounded text-xs hover:bg-gray-600 transition-colors"
        >
          Practice
        </button>
      </div>
    </div>

    <!-- Search and Filter -->
    <div class="p-3 border-b border-gray-700">
      <div class="flex items-center space-x-2">
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Search shortcuts..."
          class="flex-1 px-3 py-2 bg-gray-800 border border-gray-600 rounded text-sm text-gray-200 placeholder-gray-400 focus:outline-none focus:border-blue-500"
        />
        <select
          v-model="selectedCategory"
          class="px-3 py-2 bg-gray-800 border border-gray-600 rounded text-sm text-gray-200 focus:outline-none focus:border-blue-500"
        >
          <option value="">All Categories</option>
          <option v-for="cat in categories" :key="cat.id" :value="cat.id">
            {{ cat.name }}
          </option>
        </select>
      </div>
    </div>

    <!-- Statistics -->
    <div class="p-3 border-b border-gray-700 bg-gray-800">
      <div class="grid grid-cols-3 gap-4 text-center">
        <div>
          <div class="text-lg font-semibold text-green-400">{{ stats.learned }}</div>
          <div class="text-xs text-gray-400">Learned</div>
        </div>
        <div>
          <div class="text-lg font-semibold text-yellow-400">{{ stats.practicing }}</div>
          <div class="text-xs text-gray-400">Practicing</div>
        </div>
        <div>
          <div class="text-lg font-semibold text-blue-400">{{ stats.total }}</div>
          <div class="text-xs text-gray-400">Total</div>
        </div>
      </div>
    </div>

    <!-- Training Mode Indicator -->
    <div v-if="isTrainingMode" class="p-3 border-b border-gray-700 bg-red-900 bg-opacity-20">
      <div class="flex items-center justify-between">
        <div class="flex items-center space-x-2">
          <div class="w-2 h-2 bg-red-400 rounded-full animate-pulse"></div>
          <span class="text-sm font-medium text-red-400">Training Mode Active</span>
        </div>
        <button
          @click="nextChallenge"
          class="px-3 py-1 bg-red-600 text-white rounded text-xs hover:bg-red-700 transition-colors"
        >
          Next Challenge
        </button>
      </div>
      
      <!-- Current Challenge -->
      <div v-if="currentChallenge" class="mt-3 p-3 bg-gray-800 rounded">
        <div class="text-sm font-medium text-gray-200 mb-2">
          Challenge: {{ currentChallenge.name }}
        </div>
        <div class="text-xs text-gray-400 mb-2">
          {{ currentChallenge.description }}
        </div>
        <div class="flex items-center space-x-2">
          <kbd class="px-2 py-1 bg-gray-700 text-gray-300 rounded text-xs font-mono">
            {{ currentChallenge.keybinding }}
          </kbd>
          <button
            @click="showHint = !showHint"
            class="text-xs text-blue-400 hover:text-blue-300"
          >
            {{ showHint ? 'Hide Hint' : 'Show Hint' }}
          </button>
        </div>
        <div v-if="showHint" class="mt-2 p-2 bg-gray-900 rounded text-xs text-gray-400">
          {{ currentChallenge.hint }}
        </div>
        
        <!-- Input for testing -->
        <div class="mt-3">
          <label class="block text-xs text-gray-400 mb-1">Try it:</label>
          <input
            v-model="userInput"
            @keydown="handleKeydown"
            type="text"
            class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-sm text-gray-200 focus:outline-none focus:border-blue-500"
            placeholder="Press the key combination..."
            ref="keyInput"
          />
          <div v-if="feedback" class="mt-2 text-xs" :class="feedbackClass">
            {{ feedback }}
          </div>
        </div>
      </div>
    </div>

    <!-- Practice Mode -->
    <div v-if="showPracticeMode" class="p-3 border-b border-gray-700 bg-green-900 bg-opacity-20">
      <div class="flex items-center justify-between mb-3">
        <span class="text-sm font-medium text-green-400">Practice Mode</span>
        <button
          @click="startPractice"
          class="px-3 py-1 bg-green-600 text-white rounded text-xs hover:bg-green-700 transition-colors"
        >
          Start Practice
        </button>
      </div>
      
      <div v-if="practiceSession" class="space-y-2">
        <div class="text-sm text-gray-300">
          Press: <kbd class="px-2 py-1 bg-gray-700 text-gray-300 rounded text-xs font-mono">
            {{ practiceSession.currentShortcut.keybinding }}
          </kbd>
        </div>
        <div class="text-xs text-gray-400">
          {{ practiceSession.currentShortcut.description }}
        </div>
        <div class="text-xs text-gray-500">
          Progress: {{ practiceSession.correct }} / {{ practiceSession.total }}
        </div>
      </div>
    </div>

    <!-- Shortcuts List -->
    <div class="flex-1 overflow-y-auto">
      <div
        v-for="shortcut in filteredShortcuts"
        :key="shortcut.id"
        @click="selectShortcut(shortcut)"
        :class="[
          'p-3 border-b border-gray-700 hover:bg-gray-800 cursor-pointer transition-colors',
          selectedShortcut?.id === shortcut.id ? 'bg-gray-800 border-l-2 border-l-blue-500' : ''
        ]"
      >
        <div class="flex items-center justify-between">
          <div class="flex-1">
            <div class="flex items-center space-x-2 mb-1">
              <h4 class="text-sm font-medium text-gray-200">{{ shortcut.name }}</h4>
              <span
                :class="[
                  'px-2 py-1 rounded text-xs',
                  getStatusClass(shortcut.status)
                ]"
              >
                {{ shortcut.status }}
              </span>
            </div>
            <p class="text-xs text-gray-400 mb-2">{{ shortcut.description }}</p>
            <div class="flex items-center space-x-2">
              <kbd class="px-2 py-1 bg-gray-700 text-gray-300 rounded text-xs font-mono">
                {{ shortcut.keybinding }}
              </kbd>
              <span class="px-2 py-1 bg-gray-600 text-gray-400 text-xs rounded">
                {{ getCategoryName(shortcut.category) }}
              </span>
            </div>
          </div>
          <div class="flex items-center space-x-1">
            <button
              @click.stop="markAsLearned(shortcut)"
              :class="[
                'p-1 rounded',
                shortcut.status === 'learned' ? 'text-green-400' : 'text-gray-500 hover:text-gray-300'
              ]"
              title="Mark as learned"
            >
              <Check class="w-3 h-3" />
            </button>
            <button
              @click.stop="addToPractice(shortcut)"
              class="p-1 rounded text-gray-500 hover:text-gray-300"
              title="Add to practice"
            >
              <Plus class="w-3 h-3" />
            </button>
          </div>
        </div>
        
        <!-- Detailed View -->
        <div v-if="selectedShortcut?.id === shortcut.id" class="mt-3 pt-3 border-t border-gray-600">
          <div class="text-xs text-gray-400 space-y-1">
            <div><strong>Command:</strong> {{ shortcut.command }}</div>
            <div v-if="shortcut.contexts"><strong>Contexts:</strong> {{ shortcut.contexts.join(', ') }}</div>
            <div><strong>Usage count:</strong> {{ shortcut.usageCount }}</div>
            <div><strong>Last used:</strong> {{ shortcut.lastUsed || 'Never' }}</div>
          </div>
        </div>
      </div>
    </div>

    <!-- Bottom Actions -->
    <div class="p-3 border-t border-gray-700">
      <div class="flex items-center justify-between">
        <button
          @click="resetProgress"
          class="text-xs text-gray-400 hover:text-gray-300"
        >
          Reset Progress
        </button>
        <button
          @click="exportShortcuts"
          class="text-xs text-blue-400 hover:text-blue-300"
        >
          Export Progress
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'
import { Check, Plus } from 'lucide-vue-next'
import type { KeyboardShortcut, ShortcutCategory } from '../types'

// Component state
const searchQuery = ref('')
const selectedCategory = ref('')
const selectedShortcut = ref<KeyboardShortcut | null>(null)
const isTrainingMode = ref(false)
const showPracticeMode = ref(false)
const currentChallenge = ref<KeyboardShortcut | null>(null)
const showHint = ref(false)
const userInput = ref('')
const feedback = ref('')
const feedbackClass = ref('')
const practiceSession = ref<any>(null)
const keyInput = ref<HTMLInputElement>()

// Categories
const categories = ref<ShortcutCategory[]>([
  { id: 'editing', name: 'Editing', description: 'Text editing commands', shortcuts: [] },
  { id: 'navigation', name: 'Navigation', description: 'Cursor and file navigation', shortcuts: [] },
  { id: 'search', name: 'Search & Replace', description: 'Find and replace operations', shortcuts: [] },
  { id: 'selection', name: 'Selection', description: 'Text selection commands', shortcuts: [] },
  { id: 'view', name: 'View', description: 'Editor view and layout', shortcuts: [] },
  { id: 'file', name: 'File Operations', description: 'File management commands', shortcuts: [] },
  { id: 'debug', name: 'Debugging', description: 'Debug and run commands', shortcuts: [] },
  { id: 'git', name: 'Git', description: 'Version control commands', shortcuts: [] },
  { id: 'terminal', name: 'Terminal', description: 'Terminal operations', shortcuts: [] }
])

// Shortcuts data
const shortcuts = ref<KeyboardShortcut[]>([
  // Editing shortcuts
  { id: '1', name: 'Save', keybinding: 'Ctrl+S', description: 'Save current file', category: 'file', command: 'workbench.action.files.save', usageCount: 0, lastUsed: '', status: 'learned' },
  { id: '2', name: 'Undo', keybinding: 'Ctrl+Z', description: 'Undo last action', category: 'editing', command: 'undo', usageCount: 0, lastUsed: '', status: 'learned' },
  { id: '3', name: 'Redo', keybinding: 'Ctrl+Y', description: 'Redo last undone action', category: 'editing', command: 'redo', usageCount: 0, lastUsed: '', status: 'learned' },
  { id: '4', name: 'Cut Line', keybinding: 'Ctrl+X', description: 'Cut current line', category: 'editing', command: 'editor.action.clipboardCutAction', usageCount: 0, lastUsed: '', status: 'practicing' },
  { id: '5', name: 'Copy Line', keybinding: 'Ctrl+C', description: 'Copy current line', category: 'editing', command: 'editor.action.clipboardCopyAction', usageCount: 0, lastUsed: '', status: 'practicing' },
  { id: '6', name: 'Paste', keybinding: 'Ctrl+V', description: 'Paste from clipboard', category: 'editing', command: 'editor.action.clipboardPasteAction', usageCount: 0, lastUsed: '', status: 'practicing' },
  
  // Navigation shortcuts
  { id: '7', name: 'Go to Line', keybinding: 'Ctrl+G', description: 'Go to specific line', category: 'navigation', command: 'editor.action.gotoLine', usageCount: 0, lastUsed: '', status: 'learning' },
  { id: '8', name: 'Go to File', keybinding: 'Ctrl+P', description: 'Quick open file', category: 'navigation', command: 'workbench.action.quickOpen', usageCount: 0, lastUsed: '', status: 'learning' },
  { id: '9', name: 'Go to Symbol', keybinding: 'Ctrl+Shift+O', description: 'Go to symbol in file', category: 'navigation', command: 'workbench.action.gotoSymbol', usageCount: 0, lastUsed: '', status: 'learning' },
  { id: '10', name: 'Go to Definition', keybinding: 'F12', description: 'Go to definition', category: 'navigation', command: 'editor.action.revealDefinition', usageCount: 0, lastUsed: '', status: 'learning' },
  { id: '11', name: 'Go Back', keybinding: 'Alt+Left', description: 'Go back to previous location', category: 'navigation', command: 'workbench.action.navigateBack', usageCount: 0, lastUsed: '', status: 'learning' },
  
  // Search shortcuts
  { id: '12', name: 'Find', keybinding: 'Ctrl+F', description: 'Find in file', category: 'search', command: 'actions.find', usageCount: 0, lastUsed: '', status: 'learned' },
  { id: '13', name: 'Find Next', keybinding: 'F3', description: 'Find next occurrence', category: 'search', command: 'editor.action.nextMatchFindAction', usageCount: 0, lastUsed: '', status: 'practicing' },
  { id: '14', name: 'Find Previous', keybinding: 'Shift+F3', description: 'Find previous occurrence', category: 'search', command: 'editor.action.previousMatchFindAction', usageCount: 0, lastUsed: '', status: 'practicing' },
  { id: '15', name: 'Replace', keybinding: 'Ctrl+H', description: 'Find and replace', category: 'search', command: 'editor.action.startFindReplaceAction', usageCount: 0, lastUsed: '', status: 'learning' },
  
  // Selection shortcuts
  { id: '16', name: 'Select All', keybinding: 'Ctrl+A', description: 'Select all text', category: 'selection', command: 'editor.action.selectAll', usageCount: 0, lastUsed: '', status: 'learned' },
  { id: '17', name: 'Select Line', keybinding: 'Ctrl+L', description: 'Select current line', category: 'selection', command: 'editor.action.selectLine', usageCount: 0, lastUsed: '', status: 'learning' },
  { id: '18', name: 'Add Cursor Above', keybinding: 'Ctrl+Alt+Up', description: 'Add cursor above', category: 'selection', command: 'editor.action.addCursorAbove', usageCount: 0, lastUsed: '', status: 'learning' },
  { id: '19', name: 'Add Cursor Below', keybinding: 'Ctrl+Alt+Down', description: 'Add cursor below', category:: 'editor.action.addCursorBelow', usageCount: 0, lastUsed: '', status: 'learning' },
  
  // View shortcuts
  { id: '20', name: 'Toggle Terminal', keybinding: 'Ctrl+`', description: 'Toggle integrated terminal', category: 'view', command: 'workbench.action.terminal.toggleTerminal', usageCount: 0, lastUsed: '', status: 'learning' },
  { id: '21', name: 'Toggle Sidebar', keybinding: 'Ctrl+B', description: 'Toggle primary sidebar', category: 'view', command: 'workbench.action.toggleSidebarVisibility', usageCount: 0, lastUsed: '', status: 'learning' },
  { id: '22', name: 'Toggle Panel', keybinding: 'Ctrl+J', description: 'Toggle panel', category: 'view', command: 'workbench.action.togglePanel', usageCount: 0, lastUsed: '', status: 'learning' },
  { id: '23', name: 'Zoom In', keybinding: 'Ctrl+Plus', description: 'Zoom in', category: 'view', command: 'workbench.action.zoomIn', usageCount: 0, lastUsed: '', status: 'learning' },
  { id: '24', name: 'Zoom Out', keybinding: 'Ctrl+-', description: 'Zoom out', category: 'view', command: 'workbench.action.zoomOut', usageCount: 0, lastUsed: '', status: 'learning' },
  
  // File shortcuts
  { id: '25', name: 'New File', keybinding: 'Ctrl+N', description: 'Create new file', category: 'file', command: 'workbench.action.files.newUntitledFile', usageCount: 0, lastUsed: '', status: 'learning' },
  { id: '26', name: 'Open File', keybinding: 'Ctrl+O', description: 'Open file', category: 'file', command: 'workbench.action.files.openFileFolder', usageCount: 0, lastUsed: '', status: 'learning' },
  { id: '27', name: 'Close Editor', keybinding: 'Ctrl+W', description: 'Close active editor', category: 'file', command: 'workbench.action.closeActiveEditor', usageCount: 0, lastUsed: '', status: 'learning' },
  { id: '28', name: 'Close Folder', keybinding: 'Ctrl+K F', description: 'Close folder', category: 'file', command: 'workbench.action.closeFolder', usageCount: 0, lastUsed: '', status: 'learning' },
  
  // Debug shortcuts
  { id: '29', name: 'Start Debugging', keybinding: 'F5', description: 'Start debugging', category: 'debug', command: 'workbench.action.debug.start', usageCount: 0, lastUsed: '', status: 'learning' },
  { id: '30', name: 'Stop Debugging', keybinding: 'Shift+F5', description: 'Stop debugging', category: 'debug', command: 'workbench.action.debug.stop', usageCount: 0, lastUsed: '', status: 'learning' },
  { id: '31', name: 'Step Over', keybinding: 'F10', description: 'Step over', category: 'debug', command: 'workbench.action.debug.stepOver', usageCount: 0, lastUsed: '', status: 'learning' },
  { id: '32', name: 'Step Into', keybinding: 'F11', description: 'Step into', category: 'debug', command: 'workbench.action.debug.stepInto', usageCount: 0, lastUsed: '', status: 'learning' },
  { id: '33', name: 'Step Out', keybinding: 'Shift+F11', description: 'Step out', category: 'debug', command: 'workbench.action.debug.stepOut', usageCount: 0, lastUsed: '', status: 'learning' },
  
  // Git shortcuts
  { id: '34', name: 'View SCM', keybinding: 'Ctrl+Shift+G', description: 'View source control', category: 'git', command: 'workbench.view.scm', usageCount: 0, lastUsed: '', status: 'learning' },
  { id: '35', name: 'Commit', keybinding: 'Ctrl+Shift+Enter', description: 'Commit changes', category: 'git', command: 'workbench.scm.action.commit', usageCount: 0, lastUsed: '', status: 'learning' },
  
  // Terminal shortcuts
  { id: '36', name: 'New Terminal', keybinding: 'Ctrl+Shift+`', description: 'Create new terminal', category: 'terminal', command: 'workbench.action.terminal.new', usageCount: 0, lastUsed: '', status: 'learning' },
  { id: '37', name: 'Run Task', keybinding: 'Ctrl+Shift+P', description: 'Run task', category: 'terminal', command: 'workbench.action.tasks.runTask', usageCount: 0, lastUsed: '', status: 'learning' }
])

// Computed
const filteredShortcuts = computed(() => {
  let filtered = shortcuts.value

  // Filter by search query
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    filtered = filtered.filter(s =>
      s.name.toLowerCase().includes(query) ||
      s.description.toLowerCase().includes(query) ||
      s.keybinding.toLowerCase().includes(query)
    )
  }

  // Filter by category
  if (selectedCategory.value) {
    filtered = filtered.filter(s => s.category === selectedCategory.value)
  }

  return filtered
})

const stats = computed(() => {
  return {
    learned: shortcuts.value.filter(s => s.status === 'learned').length,
    practicing: shortcuts.value.filter(s => s.status === 'practicing').length,
    learning: shortcuts.value.filter(s => s.status === 'learning').length,
    total: shortcuts.value.length
  }
})

// Methods
const getCategoryName = (categoryId: string) => {
  const category = categories.value.find(c => c.id === categoryId)
  return category?.name || categoryId
}

const getStatusClass = (status: string) => {
  switch (status) {
    case 'learned': return 'bg-green-600 text-white'
    case 'practicing': return 'bg-yellow-600 text-white'
    case 'learning': return 'bg-blue-600 text-white'
    default: return 'bg-gray-600 text-gray-300'
  }
}

const selectShortcut = (shortcut: KeyboardShortcut) => {
  selectedShortcut.value = shortcut
}

const markAsLearned = (shortcut: KeyboardShortcut) => {
  shortcut.status = 'learned'
  shortcut.lastUsed = new Date().toISOString()
  shortcut.usageCount++
  
  // Save to localStorage
  saveProgress()
}

const addToPractice = (shortcut: KeyboardShortcut) => {
  if (!practiceSession.value) {
    startPractice()
  }
  
  if (practiceSession.value) {
    practiceSession.value.queue.push(shortcut)
  }
}

const toggleTrainerMode = () => {
  isTrainingMode.value = !isTrainingMode.value
  
  if (isTrainingMode.value) {
    nextChallenge()
  } else {
    currentChallenge.value = null
    showHint.value = false
    userInput.value = ''
    feedback.value = ''
  }
}

const nextChallenge = () => {
  // Get a random shortcut that's not learned
  const learningShortcuts = shortcuts.value.filter(s => s.status !== 'learned')
  if (learningShortcuts.length > 0) {
    const random = Math.floor(Math.random() * learningShortcuts.length)
    currentChallenge.value = learningShortcuts[random]
    showHint.value = false
    userInput.value = ''
    feedback.value = ''
    
    // Focus input
    nextTick(() => {
      keyInput.value?.focus()
    })
  }
}

const handleKeydown = (event: KeyboardEvent) => {
  event.preventDefault()
  
  if (!currentChallenge.value) return
  
  const keyCombo = getKeyCombo(event)
  
  if (keyCombo === currentChallenge.value.keybinding) {
    feedback.value = 'Correct! 🎉'
    feedbackClass.value = 'text-green-400'
    currentChallenge.value.status = 'practicing'
    
    setTimeout(() => {
      nextChallenge()
    }, 1500)
  } else {
    feedback.value = 'Try again...'
    feedbackClass.value = 'text-red-400'
  }
}

const getKeyCombo = (event: KeyboardEvent) => {
  const parts = []
  
  if (event.ctrlKey) parts.push('Ctrl')
  if (event.altKey) parts.push('Alt')
  if (event.shiftKey) parts.push('Shift')
  if (event.metaKey) parts.push('Cmd')
  
  // Add the actual key
  if (event.key === ' ') {
    parts.push('Space')
  } else if (event.key.length === 1) {
    parts.push(event.key.toUpperCase())
  } else {
    parts.push(event.key)
  }
  
  return parts.join('+')
}

const startPractice = () => {
  practiceSession.value = {
    currentShortcut: shortcuts.value[Math.floor(Math.random() * shortcuts.value.length)],
    queue: [],
    correct: 0,
    total: 0
  }
}

const resetProgress = () => {
  if (confirm('Are you sure you want to reset all progress?')) {
    shortcuts.value.forEach(shortcut => {
      shortcut.status = 'learning'
      shortcut.usageCount = 0
      shortcut.lastUsed = ''
    })
    saveProgress()
  }
}

const exportShortcuts = () => {
  const data = {
    shortcuts: shortcuts.value,
    exportedAt: new Date().toISOString()
  }
  
  const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  
  const a = document.createElement('a')
  a.href = url
  a.download = 'keyboard-shortcuts-progress.json'
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
}

const saveProgress = () => {
  localStorage.setItem('super-ide-shortcuts-progress', JSON.stringify(shortcuts.value))
}

const loadProgress = () => {
  const saved = localStorage.getItem('super-ide-shortcuts-progress')
  if (saved) {
    try {
      const savedShortcuts = JSON.parse(saved)
      // Merge saved progress with current shortcuts
      shortcuts.value.forEach(shortcut => {
        const saved = savedShortcuts.find((s: KeyboardShortcut) => s.id === shortcut.id)
        if (saved) {
          shortcut.status = saved.status
          shortcut.usageCount = saved.usageCount || 0
          shortcut.lastUsed = saved.lastUsed || ''
        }
      })
    } catch (error) {
      console.error('Failed to load shortcuts progress:', error)
    }
  }
}

// Lifecycle
onMounted(() => {
  loadProgress()
  
  // Add keyboard listener for global shortcuts
  document.addEventListener('keydown', (event) => {
    if (isTrainingMode.value && event.target === keyInput.value) {
      return // Let the input handle it
    }
    
    // Track shortcut usage
    const keyCombo = getKeyCombo(event)
    const shortcut = shortcuts.value.find(s => s.keybinding === keyCombo)
    
    if (shortcut) {
      shortcut.usageCount++
      shortcut.lastUsed = new Date().toISOString()
      saveProgress()
    }
  })
})

onUnmounted(() => {
  saveProgress()
})
</script>

<style scoped>
.keyboard-shortcuts-trainer {
  @apply flex flex-col h-full bg-gray-800 text-white;
}

.keyboard-shortcuts-trainer ::-webkit-scrollbar {
  width: 6px;
}

.keyboard-shortcuts-trainer ::-webkit-scrollbar-track {
  background: #374151;
}

.keyboard-shortcuts-trainer ::-webkit-scrollbar-thumb {
  background: #6B7280;
  border-radius: 3px;
}

.keyboard-shortcuts-trainer ::-webkit-scrollbar-thumb:hover {
  background: #9CA3AF;
}

/* Keyboard key styling */
kbd {
  @apply inline-block px-2 py-1 bg-gray-700 text-gray-300 rounded text-xs font-mono border border-gray-600;
}

/* Animation for correct/incorrect feedback */
@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.7; }
}

.animate-pulse {
  animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}
</style>