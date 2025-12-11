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
    
    <!-- Right Panel: AI Assistant or Learning Mode -->
    <div class="w-80 border-l border-gray-700 flex flex-col">
      <!-- Panel Toggle -->
      <div class="p-2 border-b border-gray-700 bg-gray-800">
        <div class="flex space-x-1">
          <button
            @click="activePanel = 'ai'"
            :class="[
              'flex-1 px-3 py-2 rounded text-sm font-medium transition-colors',
              activePanel === 'ai'
                ? 'bg-blue-600 text-white'
                : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
            ]"
          >
            <MessageCircle class="w-4 h-4 inline mr-2" />
            AI Assistant
          </button>
          <button
            @click="activePanel = 'learning'"
            :class="[
              'flex-1 px-3 py-2 rounded text-sm font-medium transition-colors',
              activePanel === 'learning'
                ? 'bg-green-600 text-white'
                : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
            ]"
          >
            <GraduationCap class="w-4 h-4 inline mr-2" />
            Learning
          </button>
        </div>
      </div>
      
      <!-- Panel Content -->
      <div class="flex-1 overflow-hidden">
        <AIAssistant v-if="activePanel === 'ai'" class="h-full" />
        <LearningPanel 
          v-if="activePanel === 'learning'" 
          class="h-full"
          :current-code="currentFile?.content || ''"
          @learning-mode-toggled="onLearningModeToggled"
          @help-requested="onHelpRequested"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { FileCode, MessageCircle, GraduationCap } from 'lucide-vue-next'
import FileExplorer from './components/FileExplorer.vue'
import CodeEditor from './components/CodeEditor.vue'
import TerminalPanel from './components/TerminalPanel.vue'
import AIAssistant from './components/AIAssistant.vue'
import LearningPanel from './components/LearningPanel.vue'
import Toolbar from './components/Toolbar.vue'
import type { FileInfo } from './types'

const currentFile = ref<FileInfo | null>(null)
const activePanel = ref<'ai' | 'learning'>('ai')

function onFileSelected(file: FileInfo) {
  currentFile.value = file
}

function onContentChanged(content: string) {
  // Handle content changes - could save to backend
  console.log('Content changed:', content)
}

function onLearningModeToggled(active: boolean) {
  console.log('Learning mode toggled:', active)
}

function onHelpRequested(context: any) {
  console.log('Help requested:', context)
  // Switch to AI panel when help is requested
  activePanel.value = 'ai'
}
</script>

<style scoped>
#app {
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
}
</style>