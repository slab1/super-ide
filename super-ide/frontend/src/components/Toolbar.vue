<template>
  <div class="flex items-center justify-between px-4 py-2 bg-gray-800 border-b border-gray-700">
    <!-- Left Side - Project Info -->
    <div class="flex items-center space-x-4">
      <div class="flex items-center space-x-2">
        <div class="w-6 h-6 bg-blue-600 rounded flex items-center justify-center">
          <span class="text-white text-xs font-bold">SI</span>
        </div>
        <span class="text-sm font-medium text-gray-200">{{ projectName }}</span>
      </div>
      
      <div class="text-xs text-gray-400">
        {{ projectLanguage }}
      </div>
      
      <div 
        class="flex items-center space-x-1 text-xs"
        :class="gitStatusClass"
      >
        <GitBranch v-if="gitStatus" class="h-3 w-3" />
        <span>{{ gitStatusText }}</span>
      </div>
    </div>

    <!-- Center - Navigation Breadcrumbs -->
    <div class="flex items-center space-x-2 text-sm text-gray-300">
      <div v-if="currentFilePath" class="flex items-center space-x-1">
        <span v-for="(segment, index) in pathSegments" :key="index">
          <span v-if="index > 0" class="text-gray-500">/</span>
          <span 
            class="hover:text-white cursor-pointer"
            @click="navigateToSegment(index)"
          >
            {{ segment }}
          </span>
        </span>
      </div>
    </div>

    <!-- Right Side - Actions -->
    <div class="flex items-center space-x-2">
      <!-- File Actions -->
      <div class="flex items-center space-x-1">
        <button
          @click="saveFile"
          :disabled="!canSave"
          class="p-2 hover:bg-gray-700 rounded"
          title="Save (Ctrl+S)"
        >
          <Save class="h-4 w-4 text-gray-400" :class="{ 'text-blue-400': hasUnsavedChanges }" />
        </button>
        
        <button
          @click="formatDocument"
          class="p-2 hover:bg-gray-700 rounded"
          title="Format Document"
        >
          <Wand2 class="h-4 w-4 text-gray-400" />
        </button>
      </div>

      <!-- View Actions -->
      <div class="flex items-center space-x-1">
        <button
          @click="toggleMinimap"
          class="p-2 hover:bg-gray-700 rounded"
          title="Toggle Minimap"
        >
          <Map class="h-4 w-4 text-gray-400" :class="{ 'text-blue-400': showMinimap }" />
        </button>
        
        <button
          @click="toggleWordWrap"
          class="p-2 hover:bg-gray-700 rounded"
          title="Toggle Word Wrap"
        >
          <WrapText class="h-4 w-4 text-gray-400" :class="{ 'text-blue-400': wordWrap }" />
        </button>
      </div>

      <!-- Git Actions -->
      <div class="flex items-center space-x-1">
        <button
          @click="showGitStatus"
          class="p-2 hover:bg-gray-700 rounded"
          title="Git Status"
        >
          <GitCommit class="h-4 w-4 text-gray-400" />
        </button>
        
        <button
          @click="showTerminal"
          class="p-2 hover:bg-gray-700 rounded"
          title="Toggle Terminal"
        >
          <Terminal class="h-4 w-4 text-gray-400" />
        </button>
      </div>

      <!-- Settings -->
      <button
        @click="showSettings"
        class="p-2 hover:bg-gray-700 rounded"
        title="Settings"
      >
        <Settings class="h-4 w-4 text-gray-400" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { 
  Save, 
  Wand2, 
  Map, 
  WrapText, 
  GitBranch, 
  GitCommit, 
  Terminal, 
  Settings 
} from 'lucide-vue-next'
import { useFileStore } from '../stores/fileStore'
import { useGitStore } from '../stores/gitStore'
// import { useSettingsStore } from '../stores/settingsStore'

const fileStore = useFileStore()
const gitStore = useGitStore()
// const settingsStore = useSettingsStore()

const projectName = ref('Super IDE Project')
const projectLanguage = ref('Mixed')
const currentFilePath = ref('')
const hasUnsavedChanges = ref(false)
const showMinimap = ref(true)
const wordWrap = ref(false)
const gitStatus = ref<string | null>(null)

const gitStatusClass = computed(() => {
  if (!gitStatus.value) return 'text-gray-400'
  if (gitStatus.value === 'clean') return 'text-green-400'
  return 'text-yellow-400'
})

const gitStatusText = computed(() => {
  if (!gitStatus.value) return 'No Git'
  if (gitStatus.value === 'clean') return 'Clean'
  return 'Modified'
})

const pathSegments = computed(() => {
  if (!currentFilePath.value) return []
  return currentFilePath.value.split('/').filter(segment => segment.length > 0)
})

const canSave = computed(() => {
  return hasUnsavedChanges.value && !!fileStore.getCurrentFile()
})

onMounted(async () => {
  await loadProjectInfo()
})

async function loadProjectInfo() {
  try {
    const project = await fileStore.getProjectInfo()
    projectName.value = project.name
    projectLanguage.value = project.language
    gitStatus.value = project.gitStatus || null
  } catch (error) {
    console.error('Failed to load project info:', error)
  }
}

async function saveFile() {
  if (!canSave.value) return
  
  try {
    await fileStore.saveCurrentFile()
    hasUnsavedChanges.value = false
  } catch (error) {
    console.error('Failed to save file:', error)
    // TODO: Show error notification
  }
}

async function formatDocument() {
  try {
    await fileStore.formatCurrentFile()
  } catch (error) {
    console.error('Failed to format document:', error)
  }
}

function toggleMinimap() {
  showMinimap.value = !showMinimap.value
  // TODO: Apply to editor
}

function toggleWordWrap() {
  wordWrap.value = !wordWrap.value
  // TODO: Apply to editor
}

function navigateToSegment(index: number) {
  // TODO: Navigate to parent directory
  // console.log('Navigate to segment:', index)
}

async function showGitStatus() {
  try {
    const status = await gitStore.getStatus()
    // TODO: Show git status panel or update UI
    // console.log('Git status:', status)
  } catch (error) {
    console.error('Failed to get git status:', error)
  }
}

function showTerminal() {
  // TODO: Focus terminal or toggle terminal panel
  console.log('Toggle terminal')
}

function showSettings() {
  // TODO: Open settings modal
  console.log('Show settings')
}

// Keyboard shortcuts
function setupKeyboardShortcuts() {
  document.addEventListener('keydown', (event) => {
    if (event.ctrlKey || event.metaKey) {
      switch (event.key) {
        case 's':
          event.preventDefault()
          saveFile()
          break
        case '`':
          event.preventDefault()
          showTerminal()
          break
      }
    }
  })
}

onMounted(() => {
  setupKeyboardShortcuts()
})

// Watch for file changes
watch(() => fileStore.getCurrentFile(), (newFile, oldFile) => {
  if (newFile) {
    currentFilePath.value = newFile.path
  }
}, { immediate: true })

// Watch for content changes
watch(() => fileStore.getCurrentFile()?.content, (newContent, oldContent) => {
  if (newContent !== oldContent) {
    hasUnsavedChanges.value = true
  }
})
</script>