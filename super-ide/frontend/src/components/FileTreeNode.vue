<template>
  <div class="select-none">
    <!-- File/Folder Item -->
    <div
      class="flex items-center py-1 px-2 hover:bg-gray-700 cursor-pointer rounded text-sm"
      :class="{ 'bg-blue-600 text-white': selectedFile?.path === item.path }"
      @click="handleClick"
      @contextmenu="handleContextMenu"
    >
      <!-- Expand/Collapse Icon -->
      <div class="w-4 h-4 mr-1 flex items-center justify-center">
        <ChevronRight
          v-if="item.type === 'directory' && !expanded"
          class="h-3 w-3 text-gray-400 transition-transform"
        />
        <ChevronDown
          v-else-if="item.type === 'directory' && expanded"
          class="h-3 w-3 text-gray-400 transition-transform"
        />
        <div v-else class="w-3 h-3"></div>
      </div>

      <!-- File/Folder Icon -->
      <File
        v-if="item.type === 'file'"
        class="h-4 w-4 mr-2 text-gray-400"
        :class="{ 'text-blue-400': isSelectedFile(item) }"
      />
      <Folder
        v-else-if="item.type === 'directory' && !expanded"
        class="h-4 w-4 mr-2 text-yellow-500"
      />
      <FolderOpen
        v-else-if="item.type === 'directory' && expanded"
        class="h-4 w-4 mr-2 text-yellow-500"
      />

      <!-- File/Folder Name -->
      <span class="flex-1 truncate" :title="item.name">
        {{ item.name }}
      </span>

      <!-- File Size (for files) -->
      <span
        v-if="item.type === 'file' && item.size"
        class="text-xs text-gray-500 ml-2"
      >
        {{ formatFileSize(item.size) }}
      </span>
    </div>

    <!-- Children (for directories) -->
    <div v-if="item.type === 'directory' && expanded && item.children">
      <FileTreeNode
        v-for="child in item.children"
        :key="child.path"
        :item="child"
        :selected-file="selectedFile"
        @file-selected="$emit('file-selected', $event)"
        @file-created="$emit('file-created', $event)"
        @folder-created="$emit('folder-created', $event)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { ChevronRight, ChevronDown, File, Folder, FolderOpen } from 'lucide-vue-next'
import type { FileInfo } from '../types'

interface Props {
  item: FileInfo
  selectedFile?: FileInfo | null
  expanded?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  expanded: false
})

const emit = defineEmits<{
  fileSelected: [file: FileInfo]
  fileCreated: [file: FileInfo]
  folderCreated: [file: FileInfo]
}>()

const expanded = ref(props.expanded)

const isSelectedFile = computed(() => {
  return props.selectedFile?.path === props.item.path
})

function handleClick() {
  if (props.item.type === 'directory') {
    expanded.value = !expanded.value
  } else {
    emit('fileSelected', props.item)
  }
}

function handleContextMenu(event: MouseEvent) {
  event.preventDefault()
  // TODO: Implement context menu
  console.log('Context menu for:', props.item.path)
}

function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B'
  
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
}
</script>