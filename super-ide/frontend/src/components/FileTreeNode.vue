<template>
  <div class="select-none">
    <!-- File/Folder Item -->
    <div
      class="flex items-center py-1 px-2 hover:bg-gray-700 cursor-pointer rounded text-sm group"
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

      <!-- Git Status Indicator -->
      <div v-if="showGitStatus" class="w-2 h-2 mr-2 flex items-center justify-center">
        <div
          class="w-2 h-2 rounded-full"
          :class="getGitStatusColor()"
          :title="getGitStatusTooltip()"
        ></div>
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
        class="text-xs text-gray-500 ml-2 opacity-0 group-hover:opacity-100 transition-opacity"
      >
        {{ formatFileSize(item.size) }}
      </span>

      <!-- Modified Time (for files) -->
      <span
        v-if="item.type === 'file' && item.modified"
        class="text-xs text-gray-500 ml-2 opacity-0 group-hover:opacity-100 transition-opacity"
      >
        {{ formatRelativeTime(item.modified) }}
      </span>
    </div>

    <!-- Children (for directories) -->
    <div v-if="item.type === 'directory' && expanded && item.children">
      <FileTreeNode
        v-for="child in item.children"
        :key="child.path"
        :item="child"
        :selected-file="selectedFile"
        :git-status="gitStatus"
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
import type { FileTreeNode as FileTreeNodeType, GitStatus } from '../types'

interface Props {
  item: FileTreeNodeType
  selectedFile?: FileInfo | null
  expanded?: boolean
  gitStatus?: GitStatus | null
}

const props = withDefaults(defineProps<Props>(), {
  expanded: false
})

const emit = defineEmits<{
  fileSelected: [file: FileTreeNodeType]
  fileCreated: [file: FileTreeNodeType]
  folderCreated: [file: FileTreeNodeType]
}>()

const expanded = ref(props.expanded)

const isSelectedFile = computed(() => {
  return props.selectedFile?.path === props.item.path
})

const showGitStatus = computed(() => {
  return props.gitStatus && props.item.type === 'file'
})

const gitFileStatus = computed(() => {
  if (!props.gitStatus || props.item.type !== 'file') return null
  
  const relativePath = props.item.path
  
  // Check staged files
  const staged = props.gitStatus.staged_files.find(f => f.path === relativePath)
  if (staged) return { type: 'staged', status: staged }
  
  // Check unstaged files  
  const unstaged = props.gitStatus.unstaged_files.find(f => f.path === relativePath)
  if (unstaged) return { type: 'modified', status: unstaged }
  
  // Check untracked files
  const untracked = props.gitStatus.untracked_files.find(f => f.path === relativePath)
  if (untracked) return { type: 'untracked', status: untracked }
  
  return null
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
  // TODO: Implement context menu with git operations
  console.log('Context menu for:', props.item.path)
}

function getGitStatusColor(): string {
  if (!gitFileStatus.value) return 'bg-transparent'
  
  switch (gitFileStatus.value.type) {
    case 'staged':
      return 'bg-green-500'
    case 'modified':
      return 'bg-yellow-500'
    case 'untracked':
      return 'bg-blue-500'
    default:
      return 'bg-gray-500'
  }
}

function getGitStatusTooltip(): string {
  if (!gitFileStatus.value) return ''
  
  switch (gitFileStatus.value.type) {
    case 'staged':
      return 'Staged for commit'
    case 'modified':
      return 'Modified (unstaged)'
    case 'untracked':
      return 'Untracked file'
    default:
      return 'Git status unknown'
  }
}

function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B'
  
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
}

function formatRelativeTime(timestamp: string): string {
  const date = new Date(timestamp)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / 60000)
  const diffHours = Math.floor(diffMs / 3600000)
  const diffDays = Math.floor(diffMs / 86400000)
  
  if (diffMins < 1) return 'now'
  if (diffMins < 60) return `${diffMins}m`
  if (diffHours < 24) return `${diffHours}h`
  if (diffDays < 7) return `${diffDays}d`
  
  return date.toLocaleDateString()
}
</script>