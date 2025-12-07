<template>
  <div id="app" class="h-screen bg-gray-900 text-white flex">
    <!-- File Explorer Sidebar -->
    <FileExplorer 
      class="w-64 border-r border-gray-700"
      @file-selected="onFileSelected"
    />
    
    <!-- Main Editor Area -->
    <div class="flex-1 flex flex-col">
      <!-- Toolbar -->
      <Toolbar class="border-b border-gray-700" />
      
      <!-- Editor and Terminal Split View -->
      <div class="flex-1 flex flex-col">
        <!-- Code Editor -->
        <CodeEditor 
          v-if="currentFile"
          :file="currentFile"
          class="flex-1"
          @content-changed="onContentChanged"
        />
        <div v-else class="flex-1 flex items-center justify-center text-gray-400">
          <div class="text-center">
            <FileCode class="mx-auto h-16 w-16 mb-4 text-gray-500" />
            <h3 class="text-lg font-medium mb-2">Welcome to Super IDE</h3>
            <p class="text-sm">Select a file from the explorer to start coding</p>
          </div>
        </div>
        
        <!-- Terminal -->
        <TerminalPanel class="h-48 border-t border-gray-700" />
      </div>
    </div>
    
    <!-- AI Assistant Panel -->
    <AIAssistant class="w-80 border-l border-gray-700" />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { FileCode } from 'lucide-vue-next'
import FileExplorer from './components/FileExplorer.vue'
import CodeEditor from './components/CodeEditor.vue'
import TerminalPanel from './components/TerminalPanel.vue'
import AIAssistant from './components/AIAssistant.vue'
import Toolbar from './components/Toolbar.vue'
import type { FileInfo } from './types'

const currentFile = ref<FileInfo | null>(null)

function onFileSelected(file: FileInfo) {
  currentFile.value = file
}

function onContentChanged(content: string) {
  // Handle content changes - could save to backend
  console.log('Content changed:', content)
}
</script>

<style scoped>
#app {
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
}
</style>