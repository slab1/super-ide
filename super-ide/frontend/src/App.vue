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
    
    <!-- Right Panel: AI Assistant, Learning Mode, Smart Snippets, or Collaboration -->
    <div class="w-80 border-l border-gray-700 flex flex-col">
      <!-- Panel Toggle -->
      <div class="p-2 border-b border-gray-700 bg-gray-800">
        <div class="grid grid-cols-3 gap-1">
          <button
            @click="activePanel = 'ai'"
            :class="[
              'px-2 py-2 rounded text-xs font-medium transition-colors',
              activePanel === 'ai'
                ? 'bg-purple-600 text-white'
                : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
            ]"
          >
            <Brain class="w-3 h-3 inline mr-1" />
            AI
          </button>
          <button
            @click="activePanel = 'git'"
            :class="[
              'px-2 py-2 rounded text-xs font-medium transition-colors',
              activePanel === 'git'
                ? 'bg-orange-600 text-white'
                : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
            ]"
          >
            <GitBranch class="w-3 h-3 inline mr-1" />
            Git
          </button>
          <button
            @click="activePanel = 'snippets'"
            :class="[
              'px-2 py-2 rounded text-xs font-medium transition-colors',
              activePanel === 'snippets'
                ? 'bg-blue-600 text-white'
                : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
            ]"
          >
            <Code class="w-3 h-3 inline mr-1" />
            Snippets
          </button>
          <button
            @click="activePanel = 'learning'"
            :class="[
              'px-2 py-2 rounded text-xs font-medium transition-colors',
              activePanel === 'learning'
                ? 'bg-green-600 text-white'
                : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
            ]"
          >
            <GraduationCap class="w-3 h-3 inline mr-1" />
            Learning
          </button>
          <button
            @click="activePanel = 'collaboration'"
            :class="[
              'px-2 py-2 rounded text-xs font-medium transition-colors',
              activePanel === 'collaboration'
                ? 'bg-teal-600 text-white'
                : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
            ]"
          >
            <Users class="w-3 h-3 inline mr-1" />
            Collab
          </button>
        </div>
      </div>
      
      <!-- Panel Content -->
      <div class="flex-1 overflow-hidden">
        <AICodeIntelligencePanel v-if="activePanel === 'ai'" class="h-full" />
        <GitPanel v-if="activePanel === 'git'" class="h-full" />
        <SmartSnippets v-if="activePanel === 'snippets'" class="h-full" />
        <LearningPanel 
          v-if="activePanel === 'learning'" 
          class="h-full"
          :current-code="currentFile?.content || ''"
          @learning-mode-toggled="onLearningModeToggled"
          @help-requested="onHelpRequested"
        />
        <CollaborationPanel v-if="activePanel === 'collaboration'" class="h-full" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { FileCode, Brain, GraduationCap, Code, GitBranch, Users } from 'lucide-vue-next'
import FileExplorer from './components/FileExplorer.vue'
import CodeEditor from './components/CodeEditor.vue'
import TerminalPanel from './components/TerminalPanel.vue'
import AICodeIntelligencePanel from './components/AICodeIntelligencePanel.vue'
import GitPanel from './components/GitPanel.vue'
import SmartSnippets from './components/SmartSnippets.vue'
import LearningPanel from './components/LearningPanel.vue'
import CollaborationPanel from './components/CollaborationPanel.vue'
import Toolbar from './components/Toolbar.vue'
import type { FileTreeNode } from './types'

const currentFile = ref<FileTreeNode | null>(null)
const activePanel = ref<'ai' | 'git' | 'snippets' | 'learning' | 'collaboration'>('ai')

function onFileSelected(file: FileTreeNode) {
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