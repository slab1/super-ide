<template>
  <div class="h-full flex flex-col bg-gray-800">
    <!-- Header -->
    <div class="p-3 border-b border-gray-700">
      <div class="flex items-center justify-between">
        <div class="flex items-center space-x-2">
          <h2 class="text-sm font-semibold text-gray-200">Explorer</h2>
          <div v-if="gitStatus" class="flex items-center space-x-1">
            <!-- Git Status Indicator -->
            <div 
              class="w-2 h-2 rounded-full"
              :class="getGitStatusColor()"
              :title="getGitStatusTooltip()"
            ></div>
            <span class="text-xs text-gray-400">{{ getBranchName() }}</span>
          </div>
          <div v-else-if="isRepository === false" class="text-xs text-gray-500">
            No Git Repo
          </div>
        </div>
        <div class="flex space-x-1">
          <button 
            @click="refreshAll"
            class="p-1 hover:bg-gray-700 rounded"
            title="Refresh"
          >
            <RotateCcw class="h-4 w-4 text-gray-400" />
          </button>
          <button 
            @click="createNewFile"
            class="p-1 hover:bg-gray-700 rounded"
            title="New File"
          >
            <FilePlus class="h-4 w-4 text-gray-400" />
          </button>
          <button 
            @click="createNewFolder"
            class="p-1 hover:bg-gray-700 rounded"
            title="New Folder"
          >
            <FolderPlus class="h-4 w-4 text-gray-400" />
          </button>
        </div>
      </div>
      
      <!-- Git Changes Summary -->
      <div v-if="gitStatus && hasChanges" class="mt-2 text-xs text-gray-400">
        <div class="flex space-x-4">
          <span v-if="gitStatus.staged_files.length > 0" class="text-green-400">
            {{ gitStatus.staged_files.length }} staged
          </span>
          <span v-if="gitStatus.unstaged_files.length > 0" class="text-yellow-400">
            {{ gitStatus.unstaged_files.length }} modified
          </span>
          <span v-if="gitStatus.untracked_files.length > 0" class="text-blue-400">
            {{ gitStatus.untracked_files.length }} untracked
          </span>
        </div>
      </div>
    </div>

    <!-- File Tree -->
    <div class="flex-1 overflow-auto p-2">
      <div v-if="loading" class="flex items-center justify-center py-8">
        <div class="animate-spin rounded-full h-6 w-6 border-b-2 border-blue-400"></div>
      </div>
      
      <div v-else-if="error" class="text-red-400 text-sm p-2">
        {{ error }}
      </div>

      <div v-else>
        <FileTreeNode
          v-for="item in fileTree"
          :key="item.path"
          :item="item"
          :selected-file="selectedFile"
          :git-status="gitStatus"
          @file-selected="$emit('file-selected', $event)"
          @file-created="onFileCreated"
          @folder-created="onFolderCreated"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { FilePlus, FolderPlus, RotateCcw } from 'lucide-vue-next'
import { useFileStore } from '../stores/fileStore'
import { useGitStore } from '../stores/gitStore'
import FileTreeNode from './FileTreeNode.vue'
import type { FileInfo, FileTreeNode as FileTreeNodeType, GitStatus } from '../types'

interface Props {
  selectedFile?: FileInfo | null
}

defineProps<Props>()

const emit = defineEmits<{
  fileSelected: [file: FileInfo]
  fileCreated: [file: FileInfo]
  folderCreated: [file: FileInfo]
}>()

const fileStore = useFileStore()
const gitStore = useGitStore()
const loading = ref(false)
const error = ref<string | null>(null)
const fileTree = ref<FileTreeNodeType[]>([])
const gitStatus = ref<GitStatus | null>(null)
const isRepository = ref<boolean | null>(null)

const hasChanges = computed(() => {
  if (!gitStatus.value) return false
  return gitStatus.value.staged_files.length > 0 || 
         gitStatus.value.unstaged_files.length > 0 || 
         gitStatus.value.untracked_files.length > 0
})

onMounted(async () => {
  await refreshAll()
})

async function refreshAll() {
  loading.value = true
  error.value = null
  
  try {
    // Load file tree and git status in parallel
    const [tree, status] = await Promise.allSettled([
      fileStore.getFileTree(),
      gitStore.getStatus()
    ])
    
    fileTree.value = tree.status === 'fulfilled' ? tree.value : []
    gitStatus.value = status.status === 'fulfilled' ? status.value : null
    isRepository.value = status.status === 'fulfilled' ? true : false
    
    if (tree.status === 'rejected') {
      throw new Error('Failed to load file tree')
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load files'
  } finally {
    loading.value = false
  }
}

async function refreshFiles() {
  await fileStore.getFileTree()
  fileTree.value = fileStore.fileTree
}

function getGitStatusColor(): string {
  if (!gitStatus.value) return 'bg-gray-500'
  if (hasChanges.value) return 'bg-yellow-500'
  return 'bg-green-500'
}

function getGitStatusTooltip(): string {
  if (!gitStatus.value) return 'Not a git repository'
  if (hasChanges.value) return 'Changes not committed'
  return 'Working directory clean'
}

function getBranchName(): string {
  return gitStore.currentBranch || 'main'
}

async function createNewFile() {
  const name = prompt('Enter file name:')
  if (!name) return
  
  try {
    const newFile = await fileStore.createFile(name)
    emit('fileCreated', newFile)
    await refreshFiles()
  } catch (err) {
    alert('Failed to create file: ' + (err instanceof Error ? err.message : 'Unknown error'))
  }
}

async function createNewFolder() {
  const name = prompt('Enter folder name:')
  if (!name) return
  
  try {
    const newFolder = await fileStore.createDirectory(name)
    emit('folderCreated', newFolder)
    await refreshFiles()
  } catch (err) {
    alert('Failed to create folder: ' + (err instanceof Error ? err.message : 'Unknown error'))
  }
}

function onFileCreated(file: FileInfo) {
  emit('fileCreated', file)
}

function onFolderCreated(folder: FileInfo) {
  emit('folderCreated', folder)
}
</script>