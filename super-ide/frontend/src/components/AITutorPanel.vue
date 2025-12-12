<template>
  <div class="ai-tutor-panel bg-gray-800 border-l border-gray-700 h-full flex flex-col">
    <!-- Header -->
    <div class="tutor-header p-4 border-b border-gray-700">
      <div class="flex items-center justify-between">
        <div class="flex items-center space-x-3">
          <div class="w-10 h-10 bg-gradient-to-r from-blue-500 to-purple-600 rounded-full flex items-center justify-center">
            <Bot class="w-6 h-6 text-white" />
          </div>
          <div>
            <h3 class="font-semibold text-white">{{ tutor.name }}</h3>
            <p class="text-xs text-gray-400">{{ tutor.personality }} Tutor</p>
          </div>
        </div>
        <button
          @click="$emit('close')"
          class="p-2 hover:bg-gray-700 rounded text-gray-400 hover:text-white transition-colors"
        >
          <X class="w-4 h-4" />
        </button>
      </div>
      
      <!-- Current Context -->
      <div v-if="currentContext" class="mt-3 p-2 bg-gray-700 rounded-lg">
        <div class="flex items-center space-x-2 text-xs text-gray-300">
          <FileText class="w-3 h-3" />
          <span>{{ currentContext.file_name || 'No file selected' }}</span>
          <span v-if="currentContext.concept" class="text-blue-400">• {{ currentContext.concept }}</span>
        </div>
      </div>
    </div>

    <!-- Chat Messages -->
    <div class="chat-messages flex-1 overflow-y-auto p-4 space-y-4">
      <!-- Welcome Message -->
      <div v-if="messages.length === 0" class="text-center py-8">
        <div class="w-16 h-16 bg-gradient-to-r from-blue-500 to-purple-600 rounded-full flex items-center justify-center mx-auto mb-4">
          <MessageCircle class="w-8 h-8 text-white" />
        </div>
        <h3 class="text-lg font-semibold text-white mb-2">AI Tutor Ready!</h3>
        <p class="text-gray-400 text-sm mb-4">
          Hi! I'm your personal programming tutor. I can help you understand code, debug issues, and learn new concepts.
        </p>
        <div class="space-y-2">
          <button
            v-for="suggestion in quickSuggestions"
            :key="suggestion.text"
            @click="sendMessage(suggestion.text)"
            class="block w-full p-2 text-left text-sm bg-gray-700 hover:bg-gray-600 rounded-lg transition-colors text-gray-300"
          >
            {{ suggestion.text }}
          </button>
        </div>
      </div>

      <!-- Chat Messages -->
      <div
        v-for="message in messages"
        :key="message.id"
        :class="[
          'flex',
          message.is_user ? 'justify-end' : 'justify-start'
        ]"
      >
        <div
          :class="[
            'max-w-xs lg:max-w-md px-4 py-2 rounded-lg',
            message.is_user 
              ? 'bg-blue-600 text-white' 
              : 'bg-gray-700 text-gray-100'
          ]"
        >
          <!-- AI Message with Avatar -->
          <div v-if="!message.is_user" class="flex items-start space-x-2">
            <div class="w-6 h-6 bg-gradient-to-r from-blue-500 to-purple-600 rounded-full flex items-center justify-center flex-shrink-0 mt-1">
              <Bot class="w-3 h-3 text-white" />
            </div>
            <div class="flex-1">
              <div class="prose prose-sm max-w-none" v-html="formatMessage(message.content)"></div>
              
              <!-- Code Block -->
              <div v-if="message.code_example" class="mt-2">
                <div class="bg-gray-900 rounded p-2 text-xs">
                  <div class="flex items-center justify-between mb-1">
                    <span class="text-gray-400">{{ message.code_language || 'code' }}</span>
                    <button
                      @click="copyCode(message.code_example)"
                      class="text-gray-400 hover:text-white"
                    >
                      <Copy class="w-3 h-3" />
                    </button>
                  </div>
                  <pre class="text-green-400 overflow-x-auto"><code>{{ message.code_example }}</code></pre>
                </div>
              </div>

              <!-- Visual Aid -->
              <div v-if="message.visual_aid" class="mt-2">
                <div class="bg-gray-900 rounded p-2">
                  <div class="text-xs text-gray-400 mb-1">{{ message.visual_aid.title }}</div>
                  <div class="text-white">{{ message.visual_aid.content }}</div>
                </div>
              </div>

              <!-- Suggested Actions -->
              <div v-if="message.suggested_actions?.length" class="mt-2 space-y-1">
                <button
                  v-for="action in message.suggested_actions"
                  :key="action"
                  @click="sendMessage(action)"
                  class="block w-full p-1 text-left text-xs bg-blue-700 hover:bg-blue-600 rounded transition-colors"
                >
                  {{ action }}
                </button>
              </div>
            </div>
          </div>

          <!-- User Message -->
          <div v-else>
            {{ message.content }}
          </div>

          <!-- Timestamp -->
          <div class="text-xs opacity-70 mt-1">
            {{ formatTime(message.timestamp) }}
          </div>
        </div>
      </div>

      <!-- Typing Indicator -->
      <div v-if="isTyping" class="flex justify-start">
        <div class="bg-gray-700 rounded-lg px-4 py-2 max-w-xs">
          <div class="flex items-center space-x-2">
            <div class="w-6 h-6 bg-gradient-to-r from-blue-500 to-purple-600 rounded-full flex items-center justify-center">
              <Bot class="w-3 h-3 text-white" />
            </div>
            <div class="flex space-x-1">
              <div class="w-2 h-2 bg-gray-400 rounded-full animate-bounce"></div>
              <div class="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style="animation-delay: 0.1s"></div>
              <div class="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style="animation-delay: 0.2s"></div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Quick Actions -->
    <div class="quick-actions p-4 border-t border-gray-700">
      <div class="grid grid-cols-2 gap-2 mb-3">
        <button
          v-for="action in contextualActions"
          :key="action.text"
          @click="performQuickAction(action.action)"
          :class="[
            'p-2 rounded-lg text-sm font-medium transition-colors',
            action.primary 
              ? 'bg-blue-600 hover:bg-blue-700 text-white' 
              : 'bg-gray-700 hover:bg-gray-600 text-gray-300'
          ]"
        >
          <component :is="action.icon" class="w-4 h-4 mx-auto mb-1" />
          {{ action.text }}
        </button>
      </div>
    </div>

    <!-- Input Area -->
    <div class="input-area p-4 border-t border-gray-700">
      <div class="flex space-x-2">
        <input
          v-model="newMessage"
          @keyup.enter="sendMessage(newMessage)"
          placeholder="Ask me anything about programming..."
          class="flex-1 bg-gray-700 text-white px-3 py-2 rounded-lg border border-gray-600 focus:border-blue-500 focus:outline-none text-sm"
          :disabled="isTyping"
        />
        <button
          @click="sendMessage(newMessage)"
          :disabled="!newMessage.trim() || isTyping"
          class="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 disabled:cursor-not-allowed text-white rounded-lg transition-colors"
        >
          <Send class="w-4 h-4" />
        </button>
      </div>
      
      <!-- Suggested Questions -->
      <div v-if="suggestedQuestions.length > 0" class="mt-2">
        <div class="text-xs text-gray-400 mb-1">Try asking:</div>
        <div class="space-y-1">
          <button
            v-for="question in suggestedQuestions"
            :key="question"
            @click="sendMessage(question)"
            class="block w-full text-left p-1 text-xs text-gray-300 hover:text-white hover:bg-gray-700 rounded transition-colors"
          >
            {{ question }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { 
  Bot, 
  X, 
  MessageCircle, 
  Send, 
  Copy, 
  FileText, 
  HelpCircle, 
  Bug, 
  Code, 
  Lightbulb,
  Play,
  BookOpen
} from 'lucide-vue-next'

// Props
interface Props {
  tutor: {
    id: string
    name: string
    personality: string
    specializations: string[]
  }
  context: {
    current_file?: string
    current_concept?: string
    error_context?: string
    student_level?: string
    code_snippet?: string
  }
}

const props = defineProps<Props>()

// Emits
const emit = defineEmits<{
  close: []
}>()

// Reactive state
const messages = ref<any[]>([])
const newMessage = ref('')
const isTyping = ref(false)

// Quick suggestions for welcome screen
const quickSuggestions = ref([
  { text: "Explain this code to me" },
  { text: "Help me debug this error" },
  { text: "How can I improve this function?" },
  { text: "What does this concept mean?" }
])

// Contextual actions based on current state
const contextualActions = computed(() => {
  const actions = []
  
  if (props.context.error_context) {
    actions.push({
      text: "Debug Error",
      action: "debug_error",
      primary: true,
      icon: Bug
    })
  }
  
  if (props.context.code_snippet) {
    actions.push({
      text: "Explain Code",
      action: "explain_code",
      primary: true,
      icon: Code
    })
  }
  
  actions.push({
    text: "Get Hint",
    action: "get_hint",
    primary: false,
    icon: Lightbulb
  })
  
  actions.push({
    text: "Show Example",
    action: "show_example",
    primary: false,
    icon: Play
  })
  
  return actions
})

// Suggested questions based on context
const suggestedQuestions = computed(() => {
  const questions = []
  
  if (props.context.code_snippet) {
    questions.push(
      "What does this function do?",
      "How can I optimize this code?",
      "What are potential bugs here?"
    )
  }
  
  if (props.context.current_concept) {
    questions.push(
      `Can you explain ${props.context.current_concept} in simple terms?`,
      `What are common mistakes with ${props.context.current_concept}?`,
      `Show me a practical example of ${props.context.current_concept}`
    )
  }
  
  if (questions.length === 0) {
    questions.push(
      "What's the best way to learn programming?",
      "How do I write clean code?",
      "What should I learn next?"
    )
  }
  
  return questions.slice(0, 3) // Limit to 3 suggestions
})

// Current context display
const currentContext = computed(() => {
  return {
    file_name: props.context.current_file ? 
      props.context.current_file.split('/').pop() : null,
    concept: props.context.current_concept
  }
})

// Methods
const sendMessage = async (content: string) => {
  if (!content.trim()) return
  
  // Add user message
  const userMessage = {
    id: Date.now(),
    content: content.trim(),
    is_user: true,
    timestamp: new Date()
  }
  messages.value.push(userMessage)
  
  // Clear input
  const messageContent = content.trim()
  newMessage.value = ''
  
  // Show typing indicator
  isTyping.value = true
  
  try {
    // Simulate AI response (in real implementation, call API)
    const response = await generateAIResponse(messageContent, props.context)
    
    // Add AI response
    const aiMessage = {
      id: Date.now() + 1,
      content: response.content,
      code_example: response.code_example,
      code_language: response.code_language,
      visual_aid: response.visual_aid,
      suggested_actions: response.suggested_actions,
      is_user: false,
      timestamp: new Date()
    }
    
    messages.value.push(aiMessage)
    
  } catch (error) {
    console.error('Error getting AI response:', error)
    const errorMessage = {
      id: Date.now() + 1,
      content: "Sorry, I encountered an error. Please try again.",
      is_user: false,
      timestamp: new Date()
    }
    messages.value.push(errorMessage)
  } finally {
    isTyping.value = false
  }
}

const generateAIResponse = async (userMessage: string, context: any): Promise<any> => {
  // Simulate AI processing delay
  await new Promise(resolve => setTimeout(resolve, 1000 + Math.random() * 2000))
  
  // Simple rule-based responses (in real implementation, this would be AI)
  const lowerMessage = userMessage.toLowerCase()
  
  if (lowerMessage.includes('debug') || lowerMessage.includes('error')) {
    return {
      content: "I can help you debug this issue! Let me analyze the error and suggest some solutions.",
      suggested_actions: ["Show me the exact error", "Help me understand the error", "Suggest a fix"]
    }
  }
  
  if (lowerMessage.includes('explain')) {
    return {
      content: "I'd be happy to explain that! Based on your current context, here's what I can tell you:",
      code_example: "// Here's a simple example\nfunction example() {\n  console.log('This demonstrates the concept');\n  return true;\n}",
      code_language: "javascript",
      suggested_actions: ["Show me a more complex example", "When should I use this?", "What are best practices?"]
    }
  }
  
  if (lowerMessage.includes('optimize') || lowerMessage.includes('improve')) {
    return {
      content: "Great question! Here are some optimization strategies for your code:",
      visual_aid: {
        title: "Optimization Tips",
        content: "• Use appropriate data structures\n• Avoid unnecessary loops\n• Cache expensive calculations\n• Use built-in methods when possible"
      },
      suggested_actions: ["Show performance comparison", "What about memory usage?", "Give me a refactored example"]
    }
  }
  
  // Default response
  return {
    content: "That's an interesting question! Let me provide some guidance based on your current learning context.",
    suggested_actions: ["Tell me more", "Show me examples", "What should I practice next?"]
  }
}

const performQuickAction = async (action: string) => {
  switch (action) {
    case 'debug_error':
      if (props.context.error_context) {
        sendMessage(`Please help me debug this error: ${props.context.error_context}`)
      } else {
        sendMessage("I don't see an error in the current context. Can you point me to the problematic code?")
      }
      break
      
    case 'explain_code':
      if (props.context.code_snippet) {
        sendMessage("Please explain this code to me:")
      } else {
        sendMessage("I'd be happy to explain code! Please share the code you'd like me to analyze.")
      }
      break
      
    case 'get_hint':
      sendMessage("I need a hint to solve this problem")
      break
      
    case 'show_example':
      sendMessage("Can you show me a practical example of this concept?")
      break
  }
}

const formatMessage = (content: string) => {
  // Simple markdown-like formatting
  return content
    .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
    .replace(/\*(.*?)\*/g, '<em>$1</em>')
    .replace(/`(.*?)`/g, '<code class="bg-gray-800 px-1 rounded">$1</code>')
    .replace(/\n/g, '<br>')
}

const formatTime = (timestamp: Date) => {
  return timestamp.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}

const copyCode = async (code: string) => {
  try {
    await navigator.clipboard.writeText(code)
    // Could show a toast notification here
  } catch (error) {
    console.error('Failed to copy code:', error)
  }
}

// Watch for context changes to update suggestions
watch(() => props.context, (newContext) => {
  // Update suggested questions based on new context
  console.log('Context updated:', newContext)
}, { deep: true })

// Lifecycle
onMounted(() => {
  // Initialize tutor session
  // Load conversation history if available
})
</script>

<style scoped>
.ai-tutor-panel {
  width: 400px;
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
}

.chat-messages {
  scrollbar-width: thin;
  scrollbar-color: #4b5563 #1f2937;
}

.chat-messages::-webkit-scrollbar {
  width: 6px;
}

.chat-messages::-webkit-scrollbar-track {
  background: #1f2937;
}

.chat-messages::-webkit-scrollbar-thumb {
  background: #4b5563;
  border-radius: 3px;
}

.chat-messages::-webkit-scrollbar-thumb:hover {
  background: #6b7280;
}

/* Message animations */
.message-enter-active,
.message-leave-active {
  transition: all 0.3s ease;
}

.message-enter-from {
  opacity: 0;
  transform: translateY(20px);
}

.message-leave-to {
  opacity: 0;
  transform: translateY(-20px);
}

/* Typing indicator animation */
.animate-bounce {
  animation: bounce 1.4s infinite;
}

@keyframes bounce {
  0%, 80%, 100% {
    transform: scale(0);
  }
  40% {
    transform: scale(1);
  }
}

/* Code block styling */
.prose code {
  background-color: #1f2937;
  padding: 0.125rem 0.25rem;
  border-radius: 0.25rem;
  font-size: 0.875rem;
}

/* Button hover effects */
.quick-actions button {
  transition: all 0.2s ease;
}

.quick-actions button:hover {
  transform: translateY(-1px);
}

/* Input focus styles */
.input-area input:focus {
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}
</style>