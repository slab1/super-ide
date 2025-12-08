<template>
  <div class="h-full flex flex-col bg-gray-800">
    <!-- Header -->
    <div class="p-3 border-b border-gray-700">
      <div class="flex items-center justify-between">
        <h2 class="text-sm font-semibold text-gray-200">Explorer</h2>
        <div class="flex space-x-1">
          <button 
            @click="refreshFiles"
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
          @file-selected="$emit('file-selected', $event)"
          @file-created="onFileCreated"
          @folder-created="onFolderCreated"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { FilePlus, FolderPlus, RotateCcw } from 'lucide-vue-next'
import { useFileStore } from '../stores/fileStore'
import FileTreeNode from './FileTreeNode.vue'
import type { FileInfo } from '../types'

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
const loading = ref(false)
const error = ref<string | null>(null)
const fileTree = ref<FileInfo[]>([])

onMounted(async () => {
  await loadFileTree()
})

async function loadFileTree() {
  loading.value = true
  error.value = null
  
  try {
    fileTree.value = await fileStore.getFileTree()
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load files'
  } finally {
    loading.value = false
  }
}

async function refreshFiles() {
  await loadFileTree()
}

async function createNewFile() {
  const name = prompt('Enter file name:')
  if (!name) return
  
  try {
    const newFile = await fileStore.createFile(name)
    emit('fileCreated', newFile)
    await loadFileTree()
  } catch (err) {
    alert('Failed to create file: ' + (err instanceof Error ? err.message : 'Unknown error'))
  }
}

async function createNewFolder() {
  const name = prompt('Enter folder name:')
  if (!name) return
  
  try {
    const newFolder = await fileStore.createFolder(name)
    emit('folderCreated', newFolder)
    await loadFileTree()
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