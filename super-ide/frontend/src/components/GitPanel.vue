<template>
  <div class="h-full flex flex-col bg-gray-800">
    <!-- Header -->
    <div class="p-3 border-b border-gray-700">
      <div class="flex items-center justify-between">
        <h2 class="text-sm font-semibold text-gray-200">Git</h2>
        <div class="flex space-x-1">
          <button 
            @click="refreshGit"
            class="p-1 hover:bg-gray-700 rounded"
            title="Refresh"
          >
            <RotateCcw class="h-4 w-4 text-gray-400" />
          </button>
        </div>
      </div>
      
      <!-- Repository Status -->
      <div v-if="isRepository === false" class="mt-2 text-xs text-gray-500">
        <div class="flex items-center space-x-2">
          <AlertCircle class="h-3 w-3" />
          <span>Not a git repository</span>
        </div>
      </div>
      
      <div v-else-if="gitStatus" class="mt-2">
        <div class="flex items-center space-x-2 text-xs">
          <GitBranch class="h-3 w-3 text-gray-400" />
          <span class="text-gray-300">{{ currentBranch }}</span>
        </div>
        
        <!-- Status Summary -->
        <div class="mt-1 flex items-center space-x-4 text-xs">
          <span v-if="gitStatus.ahead_count > 0" class="text-blue-400">
            ↑{{ gitStatus.ahead_count }}
          </span>
          <span v-if="gitStatus.behind_count > 0" class="text-purple-400">
            ↓{{ gitStatus.behind_count }}
          </span>
          <span 
            class="w-2 h-2 rounded-full"
            :class="getOverallStatusColor()"
          ></span>
        </div>
      </div>
    </div>

    <!-- Git Changes -->
    <div class="flex-1 overflow-auto p-2">
      <div v-if="loading" class="flex items-center justify-center py-8">
        <div class="animate-spin rounded-full h-6 w-6 border-b-2 border-blue-400"></div>
      </div>
      
      <div v-else-if="error" class="text-red-400 text-sm p-2">
        {{ error }}
      </div>

      <div v-else-if="!isRepository" class="text-gray-500 text-sm p-4 text-center">
        <GitBranch class="h-8 w-8 mx-auto mb-2 opacity-50" />
        <p>Initialize a git repository to start version control</p>
        <button 
          @click="initializeRepository"
          class="mt-2 px-3 py-1 bg-blue-600 hover:bg-blue-700 rounded text-xs"
        >
          Initialize Repository
        </button>
      </div>

      <div v-else>
        <!-- Staged Changes -->
        <div v-if="gitStatus?.staged_files?.length" class="mb-4">
          <h3 class="text-xs font-semibold text-green-400 mb-2 flex items-center">
            <CheckCircle class="h-3 w-3 mr-1" />
            Staged ({{ gitStatus.staged_files.length }})
          </h3>
          <div class="space-y-1">
            <div
              v-for="file in gitStatus.staged_files"
              :key="file.path"
              class="flex items-center py-1 px-2 bg-gray-700 rounded text-xs"
            >
              <div class="w-2 h-2 bg-green-500 rounded-full mr-2"></div>
              <span class="flex-1 truncate">{{ file.path }}</span>
              <span v-if="file.added_lines" class="text-green-400 text-xs">
                +{{ file.added_lines }}
              </span>
              <span v-if="file.removed_lines" class="text-red-400 text-xs">
                -{{ file.removed_lines }}
              </span>
            </div>
          </div>
        </div>

        <!-- Unstaged Changes -->
        <div v-if="gitStatus?.unstaged_files?.length" class="mb-4">
          <h3 class="text-xs font-semibold text-yellow-400 mb-2 flex items-center">
            <Edit class="h-3 w-3 mr-1" />
            Modified ({{ gitStatus.unstaged_files.length }})
          </h3>
          <div class="space-y-1">
            <div
              v-for="file in gitStatus.unstaged_files"
              :key="file.path"
              class="flex items-center py-1 px-2 bg-gray-700 rounded text-xs"
            >
              <div class="w-2 h-2 bg-yellow-500 rounded-full mr-2"></div>
              <span class="flex-1 truncate">{{ file.path }}</span>
              <span v-if="file.added_lines" class="text-green-400 text-xs">
                +{{ file.added_lines }}
              </span>
              <span v-if="file.removed_lines" class="text-red-400 text-xs">
                -{{ file.removed_lines }}
              </span>
            </div>
          </div>
        </div>

        <!-- Untracked Files -->
        <div v-if="gitStatus?.untracked_files?.length" class="mb-4">
          <h3 class="text-xs font-semibold text-blue-400 mb-2 flex items-center">
            <PlusCircle class="h-3 w-3 mr-1" />
            Untracked ({{ gitStatus.untracked_files.length }})
          </h3>
          <div class="space-y-1">
            <div
              v-for="file in gitStatus.untracked_files"
              :key="file.path"
              class="flex items-center py-1 px-2 bg-gray-700 rounded text-xs"
            >
              <div class="w-2 h-2 bg-blue-500 rounded-full mr-2"></div>
              <span class="flex-1 truncate">{{ file.path }}</span>
            </div>
          </div>
        </div>

        <!-- Clean State -->
        <div v-if="!hasChanges" class="text-gray-500 text-sm p-4 text-center">
          <CheckCircle class="h-8 w-8 mx-auto mb-2 text-green-500" />
          <p>Working directory clean</p>
        </div>
      </div>
    </div>

    <!-- Actions -->
    <div v-if="isRepository && hasChanges" class="p-3 border-t border-gray-700">
      <div class="space-y-2">
        <button 
          @click="showCommitDialog = true"
          :disabled="gitStatus?.staged_files?.length === 0"
          class="w-full px-3 py-2 bg-green-600 hover:bg-green-700 disabled:bg-gray-600 disabled:cursor-not-allowed rounded text-sm"
        >
          Commit Changes
        </button>
        
        <div class="flex space-x-2">
          <button 
            @click="stageAllChanges"
            class="flex-1 px-2 py-1 bg-blue-600 hover:bg-blue-700 rounded text-xs"
          >
            Stage All
          </button>
          <button 
            @click="discardAllChanges"
            class="flex-1 px-2 py-1 bg-red-600 hover:bg-red-700 rounded text-xs"
          >
            Discard
          </button>
        </div>
      </div>
    </div>

    <!-- Commit Dialog -->
    <div v-if="showCommitDialog" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div class="bg-gray-800 rounded-lg p-6 w-96 max-w-full mx-4">
        <h3 class="text-lg font-semibold text-white mb-4">Commit Changes</h3>
        
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-300 mb-2">
              Commit Message
            </label>
            <textarea
              v-model="commitMessage"
              class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-white text-sm resize-none"
              rows="3"
              placeholder="Enter commit message..."
            ></textarea>
          </div>
          
          <div class="flex justify-end space-x-3">
            <button 
              @click="showCommitDialog = false"
              class="px-4 py-2 text-gray-400 hover:text-white"
            >
              Cancel
            </button>
            <button 
              @click="commitChanges"
              :disabled="!commitMessage.trim() || isCommitting"
              class="px-4 py-2 bg-green-600 hover:bg-green-700 disabled:bg-gray-600 disabled:cursor-not-allowed rounded text-sm"
            >
              <span v-if="isCommitting">Committing...</span>
              <span v-else>Commit</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { 
  RotateCcw, 
  GitBranch, 
  AlertCircle, 
  CheckCircle, 
  Edit, 
  PlusCircle 
} from 'lucide-vue-next'
import { useGitStore } from '../stores/gitStore'
import type { GitStatus } from '../types'

const gitStore = useGitStore()

const loading = ref(false)
const error = ref<string | null>(null)
const showCommitDialog = ref(false)
const commitMessage = ref('')
const isCommitting = ref(false)

const gitStatus = computed(() => gitStore.status)
const currentBranch = computed(() => gitStore.currentBranch)
const isRepository = computed(() => gitStore.isRepository)
const hasChanges = computed(() => gitStore.getTotalChanges() > 0)

onMounted(async () => {
  await refreshGit()
})

async function refreshGit() {
  loading.value = true
  error.value = null
  
  try {
    await gitStore.refresh()
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load git status'
  } finally {
    loading.value = false
  }
}

function getOverallStatusColor(): string {
  if (!gitStatus.value) return 'bg-gray-500'
  if (hasChanges.value) return 'bg-yellow-500'
  return 'bg-green-500'
}

async function initializeRepository() {
  // This would require a new API endpoint
  error.value = 'Repository initialization not yet implemented'
}

async function stageAllChanges() {
  // This would require a new API endpoint
  error.value = 'Staging all changes not yet implemented'
}

async function discardAllChanges() {
  // This would require a new API endpoint
  error.value = 'Discarding changes not yet implemented'
}

async function commitChanges() {
  if (!commitMessage.value.trim()) return
  
  isCommitting.value = true
  error.value = null
  
  try {
    await gitStore.commit(commitMessage.value)
    commitMessage.value = ''
    showCommitDialog.value = false
    await refreshGit()
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to commit changes'
  } finally {
    isCommitting.value = false
  }
}
</script>