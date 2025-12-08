<template>
  <div class="h-full flex flex-col bg-gray-900">
    <!-- Terminal Header -->
    <div class="flex items-center justify-between px-3 py-2 border-b border-gray-700 bg-gray-800">
      <div class="flex items-center space-x-2">
        <Terminal class="h-4 w-4 text-green-400" />
        <span class="text-sm font-medium text-gray-200">Terminal</span>
        <select 
          v-model="activeSessionId"
          @change="switchSession"
          class="bg-gray-700 text-white text-sm rounded px-2 py-1 border-none focus:ring-1 focus:ring-blue-500"
        >
          <option 
            v-for="session in sessions" 
            :key="session.id" 
            :value="session.id"
          >
            {{ session.name }}
          </option>
        </select>
      </div>
      
      <div class="flex space-x-1">
        <button 
          @click="createNewSession"
          class="p-1 hover:bg-gray-700 rounded"
          title="New Terminal"
        >
          <Plus class="h-4 w-4 text-gray-400" />
        </button>
        <button 
          @click="clearTerminal"
          class="p-1 hover:bg-gray-700 rounded"
          title="Clear Terminal"
        >
          <Trash2 class="h-4 w-4 text-gray-400" />
        </button>
        <button 
          @click="killSession"
          class="p-1 hover:bg-gray-700 rounded"
          title="Kill Terminal"
        >
          <X class="h-4 w-4 text-gray-400" />
        </button>
      </div>
    </div>

    <!-- Terminal Output -->
    <div 
      ref="terminalOutput"
      class="flex-1 overflow-auto p-3 font-mono text-sm bg-gray-900 text-green-400"
      @click="focusInput"
    >
      <div v-if="output.length === 0" class="text-gray-500">
        Terminal ready. Type commands below.
      </div>
      
      <div
        v-for="(line, index) in output"
        :key="index"
        class="whitespace-pre-wrap break-words"
        :class="getLineClass(line)"
      >
        {{ line.content }}
      </div>
    </div>

    <!-- Terminal Input -->
    <div class="border-t border-gray-700 bg-gray-800 p-2">
      <div class="flex items-center">
        <span class="text-green-400 mr-2">{{ getPrompt() }}</span>
        <input
          ref="terminalInput"
          v-model="currentCommand"
          @keydown="handleKeyDown"
          @keyup="handleKeyUp"
          class="flex-1 bg-transparent text-green-400 font-mono outline-none"
          placeholder="Type a command..."
          autocomplete="off"
          spellcheck="false"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import { Terminal, Plus, Trash2, X } from 'lucide-vue-next'
import { useTerminalStore } from '../stores/terminalStore'
import type { TerminalSession, TerminalOutput } from '../types'

const terminalStore = useTerminalStore()

const sessions = ref<TerminalSession[]>([])
const activeSessionId = ref<string>('')
const currentCommand = ref('')
const terminalOutput = ref<HTMLDivElement>()
const terminalInput = ref<HTMLInputElement>()
const output = ref<TerminalOutput[]>([])
const commandHistory = ref<string[]>([])
const historyIndex = ref(-1)

onMounted(async () => {
  await initializeTerminal()
})

onUnmounted(() => {
  cleanup()
})

async function initializeTerminal() {
  try {
    // Create initial terminal session
    await createNewSession()
    
    // Connect to WebSocket for real-time communication
    terminalStore.connect()
    
    // Listen for terminal output
    terminalStore.onOutput((data) => {
      addOutput(data)
    })
  } catch (error) {
    console.error('Failed to initialize terminal:', error)
    addOutput({
      type: 'error',
      content: 'Failed to initialize terminal',
      timestamp: new Date()
    })
  }
}

async function createNewSession() {
  try {
    const session = await terminalStore.createSession()
    sessions.value.push(session)
    activeSessionId.value = session.id
    await switchToSession(session.id)
  } catch (error) {
    console.error('Failed to create terminal session:', error)
  }
}

async function switchSession() {
  if (activeSessionId.value) {
    await switchToSession(activeSessionId.value)
  }
}

async function switchToSession(sessionId: string) {
  try {
    await terminalStore.switchToSession(sessionId)
    const session = sessions.value.find(s => s.id === sessionId)
    if (session) {
      output.value = await terminalStore.getSessionOutput(sessionId)
    }
  } catch (error) {
    console.error('Failed to switch to session:', error)
  }
}

function addOutput(data: TerminalOutput) {
  output.value.push(data)
  nextTick(() => {
    if (terminalOutput.value) {
      terminalOutput.value.scrollTop = terminalOutput.value.scrollHeight
    }
  })
}

function getPrompt(): string {
  const session = sessions.value.find(s => s.id === activeSessionId.value)
  return session ? `${session.name}@super-ide:${session.currentDirectory}$` : '$'
}

function getLineClass(line: TerminalOutput) {
  const classes = []
  
  if (line.type === 'error') {
    classes.push('text-red-400')
  } else if (line.type === 'input') {
    classes.push('text-blue-400')
  } else if (line.type === 'system') {
    classes.push('text-yellow-400')
  }
  
  return classes.join(' ')
}

function focusInput() {
  if (terminalInput.value) {
    terminalInput.value.focus()
  }
}

async function executeCommand() {
  const command = currentCommand.value.trim()
  if (!command) return

  // Add to history
  commandHistory.value.push(command)
  historyIndex.value = -1

  // Add command to output
  addOutput({
    type: 'input',
    content: `${getPrompt()} ${command}`,
    timestamp: new Date()
  })

  // Clear input
  currentCommand.value = ''

  try {
    await terminalStore.executeCommand(activeSessionId.value, command)
  } catch (error) {
    addOutput({
      type: 'error',
      content: `Command failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
      timestamp: new Date()
    })
  }
}

function handleKeyDown(event: KeyboardEvent) {
  if (event.key === 'Enter') {
    event.preventDefault()
    executeCommand()
  } else if (event.key === 'ArrowUp') {
    event.preventDefault()
    navigateHistory(-1)
  } else if (event.key === 'ArrowDown') {
    event.preventDefault()
    navigateHistory(1)
  } else if (event.key === 'Tab') {
    event.preventDefault()
    // TODO: Implement command completion
  }
}

function handleKeyUp(event: KeyboardEvent) {
  // Handle other key events if needed
}

function navigateHistory(direction: number) {
  if (commandHistory.value.length === 0) return
  
  historyIndex.value += direction
  
  if (historyIndex.value >= 0) {
    historyIndex.value = commandHistory.value.length - 1
    currentCommand.value = commandHistory.value[historyIndex.value]
  } else if (historyIndex.value < -commandHistory.value.length) {
    historyIndex.value = -1
    currentCommand.value = ''
  } else {
    currentCommand.value = commandHistory.value[commandHistory.value.length + historyIndex.value]
  }
}

async function clearTerminal() {
  output.value = []
}

async function killSession() {
  if (!activeSessionId.value) return
  
  try {
    await terminalStore.killSession(activeSessionId.value)
    const sessionIndex = sessions.value.findIndex(s => s.id === activeSessionId.value)
    if (sessionIndex !== -1) {
      sessions.value.splice(sessionIndex, 1)
    }
    
    if (sessions.value.length > 0) {
      activeSessionId.value = sessions.value[0].id
      await switchToSession(activeSessionId.value)
    } else {
      await createNewSession()
    }
  } catch (error) {
    console.error('Failed to kill session:', error)
  }
}

function cleanup() {
  terminalStore.disconnect()
}
</script>

<style scoped>
/* Terminal specific styles */
.font-mono {
  font-family: 'Fira Code', 'Monaco', 'Consolas', 'Courier New', monospace;
}

/* Scrollbar styling for terminal output */
:deep(::-webkit-scrollbar) {
  width: 8px;
}

:deep(::-webkit-scrollbar-track) {
  background: #1f2937;
}

:deep(::-webkit-scrollbar-thumb) {
  background: #4b5563;
  border-radius: 4px;
}

:deep(::-webkit-scrollbar-thumb:hover) {
  background: #6b7280;
}
</style>