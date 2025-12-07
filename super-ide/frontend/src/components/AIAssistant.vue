<template>
  <div class="h-full flex flex-col bg-gray-800">
    <!-- Header -->
    <div class="p-3 border-b border-gray-700">
      <div class="flex items-center justify-between">
        <div class="flex items-center space-x-2">
          <Bot class="h-4 w-4 text-blue-400" />
          <h2 class="text-sm font-semibold text-gray-200">AI Assistant</h2>
        </div>
        <div class="flex space-x-1">
          <button 
            @click="clearChat"
            class="p-1 hover:bg-gray-700 rounded"
            title="Clear Chat"
          >
            <Trash2 class="h-4 w-4 text-gray-400" />
          </button>
          <button 
            @click="toggleSettings"
            class="p-1 hover:bg-gray-700 rounded"
            title="Settings"
          >
            <Settings class="h-4 w-4 text-gray-400" />
          </button>
        </div>
      </div>
    </div>

    <!-- Chat Messages -->
    <div 
      ref="chatContainer"
      class="flex-1 overflow-auto p-3 space-y-3"
    >
      <div v-if="messages.length === 0" class="text-center text-gray-500 mt-8">
        <Bot class="h-12 w-12 mx-auto mb-3 text-gray-600" />
        <p class="text-sm">Hello! I'm your AI coding assistant.</p>
        <p class="text-xs mt-1">Ask me to explain code, write functions, or debug issues.</p>
      </div>

      <div
        v-for="message in messages"
        :key="message.id"
        class="flex"
        :class="{ 'justify-end': message.type === 'user' }"
      >
        <div
          class="max-w-[80%] p-3 rounded-lg text-sm"
          :class="messageClasses(message.type)"
        >
                   <div v <!-- User Message -->
 === 'user'"-if="message.type class="text-gray-800">
            {{ message.content }}
          </div>

          <!-- AI Message -->
          <div v-else class="text-gray-200">
            <div class="whitespace-pre-wrap">{{ message.content }}</div>
            
            <!-- Code Suggestions -->
            <div v-if="message.suggestions && message.suggestions.length > 0" class="mt-3 space-y-2">
              <h4 class="text-xs font-semibold text-blue-300">Code Suggestions:</h4>
              <div
                v-for="suggestion in message.suggestions"
                :key="suggestion.label"
                class="bg-gray-700 p-2 rounded border-l-2 border-blue-400 cursor-pointer hover:bg-gray-600"
                @click="applySuggestion(suggestion)"
              >
                <div class="font-mono text-xs text-blue-300">{{ suggestion.label }}</div>
                <div v-if="suggestion.detail" class="text-xs text-gray-400 mt-1">
                  {{ suggestion.detail }}
                </div>
                <div v-if="suggestion.documentation" class="text-xs text-gray-500 mt-1">
                  {{ suggestion.documentation }}
                </div>
              </div>
            </div>
          </div>

          <!-- Timestamp -->
          <div class="text-xs text-gray-500 mt-1">
            {{ formatTime(message.timestamp) }}
          </div>
        </div>
      </div>

      <!-- Typing Indicator -->
      <div v-if="isTyping" class="flex">
        <div class="bg-gray-700 p-3 rounded-lg">
          <div class="flex space-x-1">
            <div class="w-2 h-2 bg-gray-400 rounded-full animate-bounce"></div>
            <div class="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style="animation-delay: 0.1s"></div>
            <div class="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style="animation-delay: 0.2s"></div>
          </div>
        </div>
      </div>
    </div>

    <!-- Input Area -->
    <div class="border-t border-gray-700 p-3">
      <div class="flex space-x-2">
        <div class="flex-1 relative">
          <textarea
            v-model="inputText"
            @keydown="handleKeyDown"
            placeholder="Ask me anything about your code..."
            class="w-full bg-gray-700 text-white rounded-lg px-3 py-2 text-sm resize-none focus:outline-none focus:ring-1 focus:ring-blue-500"
            rows="2"
          ></textarea>
          <div class="absolute bottom-2 right-2 text-xs text-gray-500">
            {{ inputText.length }}/1000
          </div>
        </div>
        <button
          @click="sendMessage"
          :disabled="!canSend"
          class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed text-sm font-medium"
        >
          <Send class="h-4 w-4" />
        </button>
      </div>

      <!-- Quick Actions -->
      <div class="flex flex-wrap gap-1 mt-2">
        <button
          v-for="action in quickActions"
          :key="action.label"
          @click="quickAction(action.prompt)"
          class="px-2 py-1 bg-gray-700 hover:bg-gray-600 rounded text-xs text-gray-300"
        >
          {{ action.label }}
        </button>
      </div>
    </div>

    <!-- Settings Panel -->
    <div v-if="showSettings" class="border-t border-gray-700 p-3 bg-gray-750">
      <h3 class="text-sm font-semibold text-gray-200 mb-2">AI Settings</h3>
      <div class="space-y-2">
        <div class="flex items-center justify-between">
          <label class="text-xs text-gray-400">Temperature</label>
          <input
            v-model.number="aiSettings.temperature"
            type="range"
            min="0"
            max="1"
            step="0.1"
            class="w-20"
          />
          <span class="text-xs text-gray-400 w-8">{{ aiSettings.temperature }}</span>
        </div>
        <div class="flex items-center justify-between">
          <label class="text-xs text-gray-400">Max Tokens</label>
          <input
            v-model.number="aiSettings.maxTokens"
            type="number"
            min="100"
            max="4000"
            class="w-16 bg-gray-700 text-white text-xs rounded px-2 py-1"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick, watch } from 'vue'
import { Bot, Send, Trash2, Settings } from 'lucide-vue-next'
import { useAIStore } from '../stores/aiStore'
import { useFileStore } from '../stores/fileStore'
import type { AIMessage, CodeCompletion } from '../types'

const aiStore = useAIStore()
const fileStore = useFileStore()

const messages = ref<AIMessage[]>([])
const inputText = ref('')
const isTyping = ref(false)
const showSettings = ref(false)
const chatContainer = ref<HTMLDivElement>()

const aiSettings = ref({
  temperature: 0.7,
  maxTokens: 2048
})

const quickActions = [
  { label: 'Explain Code', prompt: 'Explain this code for me' },
  { label: 'Write Tests', prompt: 'Write unit tests for this' },
  { label: 'Optimize', prompt: 'How can I optimize this code?' },
  { label: 'Debug', prompt: 'Help me debug this issue' },
  { label: 'Document', prompt: 'Add documentation to this code' }
]

const canSend = computed(() => {
  return inputText.value.trim().length > 0 && inputText.value.length <= 1000 && !isTyping.value
})

const messageClasses = (type: 'user' | 'assistant') => {
  return type === 'user' 
    ? 'bg-blue-600 text-white ml-8' 
    : 'bg-gray-700 text-gray-200 mr-8'
}

onMounted(() => {
  // Add welcome message
  addMessage('assistant', 
    "Hello! I'm your AI coding assistant. I can help you with:\n\n" +
    "• Explaining code and concepts\n" +
    "• Writing and optimizing functions\n" +
    "• Debugging issues\n" +
    "• Generating tests and documentation\n" +
    "• Code reviews and suggestions\n\n" +
    "What would you like to work on?"
  )
})

function addMessage(type: 'user' | 'assistant', content: string, suggestions?: CodeCompletion[]) {
  const message: AIMessage = {
    id: Date.now().toString(),
    type,
    content,
    timestamp: new Date(),
    suggestions
  }
  messages.value.push(message)
  scrollToBottom()
}

function scrollToBottom() {
  nextTick(() => {
    if (chatContainer.value) {
      chatContainer.value.scrollTop = chatContainer.value.scrollHeight
    }
  })
}

async function sendMessage() {
  if (!canSend.value) return

  const content = inputText.value.trim()
  inputText.value = ''
  addMessage('user', content)
  
  isTyping.value = true

  try {
    // Get current file context for AI
    const currentFile = await fileStore.getCurrentFile()
    const context = {
      filePath: currentFile?.path,
      fileContent: currentFile?.content,
      language: getLanguageFromFileName(currentFile?.name || '')
    }

    // Send to AI
    const response = await aiStore.sendMessage(content, context)
    
    addMessage('assistant', response.content, response.suggestions)
  } catch (error) {
    addMessage('assistant', 
      `Sorry, I encountered an error: ${error instanceof Error ? error.message : 'Unknown error'}`
    )
  } finally {
    isTyping.value = false
  }
}

function quickAction(prompt: string) {
  inputText.value = prompt
}

function handleKeyDown(event: KeyboardEvent) {
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault()
    sendMessage()
  }
}

function applySuggestion(suggestion: CodeCompletion) {
  // TODO: Apply suggestion to the current editor
  console.log('Applying suggestion:', suggestion)
}

function clearChat() {
  messages.value = []
  onMounted(() => {
    // Re-add welcome message
    addMessage('assistant', 
      "Hello! I'm your AI coding assistant. How can I help you today?"
    )
  })
}

function toggleSettings() {
  showSettings.value = !showSettings.value
}

function formatTime(timestamp: Date): string {
  return timestamp.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}

function getLanguageFromFileName(filename: string): string {
  const ext = filename.split('.').pop()?.toLowerCase()
  const languageMap: Record<string, string> = {
    'rs': 'rust',
    'py': 'python',
    'js': 'javascript',
    'ts': 'typescript',
    'go': 'go',
    'java': 'java',
    'cpp': 'cpp',
    'c': 'c'
  }
  return languageMap[ext || ''] || 'plaintext'
}
</script>

<style scoped>
/* Custom scrollbar */
:deep(::-webkit-scrollbar) {
  width: 6px;
}

:deep(::-webkit-scrollbar-track) {
  background: #374151;
}

:deep(::-webkit-scrollbar-thumb) {
  background: #6b7280;
  border-radius: 3px;
}

:deep(::-webkit-scrollbar-thumb:hover) {
  background: #9ca3af;
}

/* Animation for typing indicator */
@keyframes bounce {
  0%, 80%, 100% { transform: translateY(0); }
  40% { transform: translateY(-4px); }
}

.animate-bounce {
  animation: bounce 1.4s infinite ease-in-out;
}
</style>