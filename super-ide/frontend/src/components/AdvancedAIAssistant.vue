<template>
  <div class="h-full flex flex-col bg-gray-900 border border-gray-700 rounded-lg">
    <!-- Header with AI Status -->
    <div class="p-4 border-b border-gray-700 bg-gray-800">
      <div class="flex items-center justify-between">
        <div class="flex items-center space-x-3">
          <div class="relative">
            <Brain class="h-6 w-6 text-purple-400" />
            <div 
              v-if="aiStatus.isActive"
              class="absolute -top-1 -right-1 w-3 h-3 bg-green-500 rounded-full animate-pulse"
            ></div>
          </div>
          <div>
            <h3 class="text-sm font-semibold text-white">Advanced AI Assistant</h3>
            <div class="flex items-center space-x-2 text-xs text-gray-400">
              <span>{{ aiStatus.provider }}</span>
              <span>•</span>
              <span>{{ aiStatus.model }}</span>
              <span>•</span>
              <span :class="aiStatus.isActive ? 'text-green-400' : 'text-red-400'">
                {{ aiStatus.isActive ? 'Online' : 'Offline' }}
              </span>
            </div>
          </div>
        </div>
        
        <div class="flex items-center space-x-2">
          <button 
            @click="toggleAdvancedFeatures"
            class="p-2 hover:bg-gray-700 rounded transition-colors"
            title="Toggle Advanced Features"
          >
            <Settings class="h-4 w-4 text-gray-400" />
          </button>
          <button 
            @click="clearContext"
            class="p-2 hover:bg-gray-700 rounded transition-colors"
            title="Clear Context"
          >
            <RefreshCw class="h-4 w-4 text-gray-400" />
          </button>
        </div>
      </div>
    </div>

    <!-- Feature Tabs -->
    <div class="flex border-b border-gray-700 bg-gray-800">
      <button
        v-for="tab in featureTabs"
        :key="tab.id"
        @click="activeTab = tab.id"
        :class="[
          'px-4 py-2 text-sm font-medium transition-colors',
          activeTab === tab.id 
            ? 'text-purple-400 border-b-2 border-purple-400 bg-gray-700' 
            : 'text-gray-400 hover:text-gray-200'
        ]"
      >
        <component :is="tab.icon" class="h-4 w-4 inline mr-2" />
        {{ tab.label }}
      </button>
    </div>

    <!-- Smart Code Completion Panel -->
    <div v-if="activeTab === 'completion'" class="flex-1 overflow-auto p-4">
      <div class="mb-4">
        <h4 class="text-sm font-semibold text-white mb-2">Smart Code Completion</h4>
        <p class="text-xs text-gray-400 mb-3">Context-aware suggestions based on your code and project structure</p>
      </div>

      <!-- Current Context -->
      <div v-if="currentContext" class="bg-gray-800 rounded-lg p-3 mb-4">
        <div class="text-xs text-gray-400 mb-2">Current Context:</div>
        <div class="text-sm text-white">{{ currentContext.fileName }} ({{ currentContext.language }})</div>
        <div class="text-xs text-gray-500 mt-1">
          Line {{ currentContext.line }}, Column {{ currentContext.column }}
        </div>
      </div>

      <!-- Completion Suggestions -->
      <div v-if="completions.length > 0" class="space-y-3">
        <div 
          v-for="completion in completions"
          :key="completion.id"
          class="bg-gray-800 rounded-lg p-3 border border-gray-600 hover:border-purple-500 transition-colors cursor-pointer"
          @click="applyCompletion(completion)"
        >
          <div class="flex items-start justify-between">
            <div class="flex-1">
              <div class="flex items-center space-x-2 mb-2">
                <span class="text-sm font-medium text-white">{{ completion.label }}</span>
                <span 
                  :class="[
                    'px-2 py-0.5 rounded text-xs font-medium',
                    completion.confidence > 0.8 ? 'bg-green-900 text-green-300' :
                    completion.confidence > 0.6 ? 'bg-yellow-900 text-yellow-300' :
                    'bg-red-900 text-red-300'
                  ]"
                >
                  {{ Math.round(completion.confidence * 100) }}%
                </span>
              </div>
              
              <div class="bg-gray-900 rounded p-2 mb-2">
                <code class="text-sm text-green-400">{{ completion.insertText }}</code>
              </div>
              
              <div v-if="completion.detail" class="text-xs text-gray-400 mb-2">
                {{ completion.detail }}
              </div>
              
              <div class="flex items-center space-x-4 text-xs text-gray-500">
                <span>{{ completion.kind }}</span>
                <span>{{ completion.fileName || 'Current file' }}</span>
              </div>
            </div>
            
            <div class="flex flex-col space-y-1">
              <button 
                @click.stop="acceptSuggestion(completion)"
                class="p-1 bg-green-600 hover:bg-green-700 rounded text-white"
                title="Accept"
              >
                <Check class="h-3 w-3" />
              </button>
              <button 
                @click.stop="rejectSuggestion(completion)"
                class="p-1 bg-red-600 hover:bg-red-700 rounded text-white"
                title="Reject"
              >
                <X class="h-3 w-3" />
              </button>
            </div>
          </div>
        </div>
      </div>

      <div v-else class="text-center py-8 text-gray-500">
        <Code class="h-12 w-12 mx-auto mb-3 text-gray-600" />
        <p class="text-sm">Start typing to get smart code suggestions</p>
        <p class="text-xs mt-1">The AI will analyze your code context and provide relevant completions</p>
      </div>
    </div>

    <!-- Code Review Panel -->
    <div v-if="activeTab === 'review'" class="flex-1 overflow-auto p-4">
      <div class="mb-4">
        <h4 class="text-sm font-semibold text-white mb-2">AI-Powered Code Review</h4>
        <p class="text-xs text-gray-400 mb-3">Get intelligent suggestions for code quality, security, and performance</p>
      </div>

      <!-- Review Actions -->
      <div class="flex space-x-2 mb-4">
        <button 
          @click="analyzeCode"
          :disabled="isAnalyzing"
          class="px-4 py-2 bg-purple-600 hover:bg-purple-700 disabled:bg-gray-600 rounded text-sm font-medium text-white transition-colors"
        >
          <Zap v-if="!isAnalyzing" class="h-4 w-4 inline mr-2" />
          <Loader v-else class="h-4 w-4 inline mr-2 animate-spin" />
          {{ isAnalyzing ? 'Analyzing...' : 'Analyze Current File' }}
        </button>
        <button 
          @click="autoFixIssues"
          :disabled="!hasFixableIssues"
          class="px-4 py-2 bg-green-600 hover:bg-green-700 disabled:bg-gray-600 rounded text-sm font-medium text-white transition-colors"
        >
          <Wrench class="h-4 w-4 inline mr-2" />
          Auto-fix Issues
        </button>
      </div>

      <!-- Review Results -->
      <div v-if="reviewResults.length > 0" class="space-y-3">
        <div 
          v-for="result in reviewResults"
          :key="result.id"
          :class="[
            'rounded-lg p-3 border',
            result.severity === 'error' ? 'bg-red-900 border-red-700' :
            result.severity === 'warning' ? 'bg-yellow-900 border-yellow-700' :
            'bg-blue-900 border-blue-700'
          ]"
        >
          <div class="flex items-start justify-between mb-2">
            <div class="flex items-center space-x-2">
              <component 
                :is="getSeverityIcon(result.severity)" 
                :class="getSeverityIconClass(result.severity)"
                class="h-4 w-4"
              />
              <span class="font-medium text-white">{{ result.title }}</span>
              <span class="text-xs text-gray-400">{{ result.category }}</span>
            </div>
            <span class="text-xs text-gray-400">Line {{ result.line }}</span>
          </div>
          
          <p class="text-sm text-gray-300 mb-2">{{ result.description }}</p>
          
          <div v-if="result.suggestion" class="bg-gray-800 rounded p-2 mb-2">
            <div class="text-xs text-gray-400 mb-1">Suggested Fix:</div>
            <code class="text-sm text-green-400">{{ result.suggestion }}</code>
          </div>
          
          <div class="flex items-center space-x-2">
            <button 
              v-if="result.canAutoFix"
              @click="applyFix(result)"
              class="px-2 py-1 bg-green-600 hover:bg-green-700 rounded text-xs text-white transition-colors"
            >
              Apply Fix
            </button>
            <button 
              @click="explainIssue(result)"
              class="px-2 py-1 bg-blue-600 hover:bg-blue-700 rounded text-xs text-white transition-colors"
            >
              Explain More
            </button>
          </div>
        </div>
      </div>

      <div v-else-if="!isAnalyzing" class="text-center py-8 text-gray-500">
        <Eye class="h-12 w-12 mx-auto mb-3 text-gray-600" />
        <p class="text-sm">Click "Analyze Current File" to review your code</p>
      </div>
    </div>

    <!-- Intelligent Debugging Panel -->
    <div v-if="activeTab === 'debugging'" class="flex-1 overflow-auto p-4">
      <div class="mb-4">
        <h4 class="text-sm font-semibold text-white mb-2">Intelligent Debugging Assistant</h4>
        <p class="text-xs text-gray-400 mb-3">Get AI-powered debugging insights and step-by-step solutions</p>
      </div>

      <!-- Debug Input -->
      <div class="bg-gray-800 rounded-lg p-4 mb-4">
        <label class="block text-sm font-medium text-gray-300 mb-2">Describe the issue you're facing:</label>
        <textarea
          v-model="debugQuery"
          placeholder="e.g., 'My function is returning None when it should return a value' or 'Getting a segmentation fault in my Rust code'"
          class="w-full bg-gray-700 text-white rounded px-3 py-2 text-sm resize-none focus:outline-none focus:ring-2 focus:ring-purple-500"
          rows="3"
        ></textarea>
        <div class="flex items-center justify-between mt-2">
          <span class="text-xs text-gray-400">{{ debugQuery.length }}/500</span>
          <button 
            @click="debugIssue"
            :disabled="!debugQuery.trim() || isDebugging"
            class="px-4 py-2 bg-purple-600 hover:bg-purple-700 disabled:bg-gray-600 rounded text-sm font-medium text-white transition-colors"
          >
            <Search v-if="!isDebugging" class="h-4 w-4 inline mr-2" />
            <Loader v-else class="h-4 w-4 inline mr-2 animate-spin" />
            {{ isDebugging ? 'Debugging...' : 'Debug Issue' }}
          </button>
        </div>
      </div>

      <!-- Debug Results -->
      <div v-if="debugResults" class="space-y-4">
        <div class="bg-gray-800 rounded-lg p-4">
          <h5 class="font-semibold text-white mb-2">Root Cause Analysis</h5>
          <p class="text-sm text-gray-300">{{ debugResults.rootCause }}</p>
        </div>

        <div v-if="debugResults.steps && debugResults.steps.length > 0">
          <h5 class="font-semibold text-white mb-2">Step-by-step Solution</h5>
          <div class="space-y-2">
            <div 
              v-for="(step, index) in debugResults.steps"
              :key="index"
              class="flex items-start space-x-3 bg-gray-800 rounded-lg p-3"
            >
              <div class="w-6 h-6 bg-purple-600 rounded-full flex items-center justify-center text-xs font-medium text-white">
                {{ index + 1 }}
              </div>
              <div class="flex-1">
                <p class="text-sm text-white">{{ step.description }}</p>
                <div v-if="step.code" class="bg-gray-900 rounded p-2 mt-2">
                  <code class="text-sm text-green-400">{{ step.code }}</code>
                </div>
                <div v-if="step.explanation" class="text-xs text-gray-400 mt-1">
                  {{ step.explanation }}
                </div>
              </div>
            </div>
          </div>
        </div>

        <div v-if="debugResults.prevention" class="bg-green-900 rounded-lg p-4">
          <h5 class="font-semibold text-white mb-2">Prevention Tips</h5>
          <p class="text-sm text-gray-300">{{ debugResults.prevention }}</p>
        </div>
      </div>
    </div>

    <!-- Project Scaffolding Panel -->
    <div v-if="activeTab === 'scaffolding'" class="flex-1 overflow-auto p-4">
      <div class="mb-4">
        <h4 class="text-sm font-semibold text-white mb-2">AI-Driven Project Scaffolding</h4>
        <p class="text-xs text-gray-400 mb-3">Generate project structures and boilerplate code intelligently</p>
      </div>

      <!-- Scaffolding Options -->
      <div class="grid grid-cols-1 gap-4">
        <div class="bg-gray-800 rounded-lg p-4">
          <h5 class="font-semibold text-white mb-3">Project Templates</h5>
          <div class="space-y-2">
            <button 
              v-for="template in projectTemplates"
              :key="template.id"
              @click="selectTemplate(template)"
              class="w-full text-left p-3 bg-gray-700 hover:bg-gray-600 rounded-lg transition-colors"
            >
              <div class="flex items-center justify-between">
                <div>
                  <div class="font-medium text-white">{{ template.name }}</div>
                  <div class="text-sm text-gray-400">{{ template.description }}</div>
                </div>
                <div class="flex items-center space-x-2">
                  <span class="px-2 py-1 bg-purple-600 rounded text-xs text-white">{{ template.language }}</span>
                  <ChevronRight class="h-4 w-4 text-gray-400" />
                </div>
              </div>
            </button>
          </div>
        </div>

        <div v-if="selectedTemplate" class="bg-gray-800 rounded-lg p-4">
          <h5 class="font-semibold text-white mb-3">Customize {{ selectedTemplate.name }}</h5>
          
          <div class="space-y-3">
            <div>
              <label class="block text-sm font-medium text-gray-300 mb-1">Project Name</label>
              <input 
                v-model="projectConfig.name"
                type="text"
                class="w-full bg-gray-700 text-white rounded px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-purple-500"
              />
            </div>
            
            <div>
              <label class="block text-sm font-medium text-gray-300 mb-1">Description</label>
              <textarea 
                v-model="projectConfig.description"
                class="w-full bg-gray-700 text-white rounded px-3 py-2 text-sm resize-none focus:outline-none focus:ring-2 focus:ring-purple-500"
                rows="2"
              ></textarea>
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-300 mb-1">Features to Include</label>
              <div class="space-y-2">
                <label 
                  v-for="feature in selectedTemplate.features"
                  :key="feature.id"
                  class="flex items-center space-x-2"
                >
                  <input 
                    v-model="projectConfig.features"
                    :value="feature.id"
                    type="checkbox"
                    class="rounded border-gray-600 text-purple-600 focus:ring-purple-500"
                  />
                  <span class="text-sm text-gray-300">{{ feature.name }}</span>
                  <span class="text-xs text-gray-500">{{ feature.description }}</span>
                </label>
              </div>
            </div>
          </div>

          <button 
            @click="generateProject"
            :disabled="isGenerating"
            class="w-full mt-4 px-4 py-2 bg-purple-600 hover:bg-purple-700 disabled:bg-gray-600 rounded text-sm font-medium text-white transition-colors"
          >
            <Zap v-if="!isGenerating" class="h-4 w-4 inline mr-2" />
            <Loader v-else class="h-4 w-4 inline mr-2 animate-spin" />
            {{ isGenerating ? 'Generating...' : 'Generate Project' }}
          </button>
        </div>
      </div>
    </div>

    <!-- Context-Aware Help Panel -->
    <div v-if="activeTab === 'help'" class="flex-1 overflow-auto p-4">
      <div class="mb-4">
        <h4 class="text-sm font-semibold text-white mb-2">Context-Aware Help System</h4>
        <p class="text-xs text-gray-400 mb-3">Get relevant help based on your current code and context</p>
      </div>

      <!-- Help Query -->
      <div class="bg-gray-800 rounded-lg p-4 mb-4">
        <label class="block text-sm font-medium text-gray-300 mb-2">What would you like help with?</label>
        <div class="flex space-x-2">
          <input 
            v-model="helpQuery"
            type="text"
            placeholder="e.g., 'How do I use async/await in Rust?' or 'Explain this error message'"
            class="flex-1 bg-gray-700 text-white rounded px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-purple-500"
          />
          <button 
            @click="searchHelp"
            :disabled="!helpQuery.trim() || isSearching"
            class="px-4 py-2 bg-purple-600 hover:bg-purple-700 disabled:bg-gray-600 rounded text-sm font-medium text-white transition-colors"
          >
            <Search v-if="!isSearching" class="h-4 w-4" />
            <Loader v-else class="h-4 w-4 animate-spin" />
          </button>
        </div>
      </div>

      <!-- Quick Help Buttons -->
      <div class="mb-4">
        <div class="text-sm font-medium text-gray-300 mb-2">Quick Help Topics</div>
        <div class="flex flex-wrap gap-2">
          <button 
            v-for="topic in quickHelpTopics"
            :key="topic.id"
            @click="getQuickHelp(topic)"
            class="px-3 py-1 bg-gray-700 hover:bg-gray-600 rounded text-xs text-gray-300 transition-colors"
          >
            {{ topic.name }}
          </button>
        </div>
      </div>

      <!-- Help Results -->
      <div v-if="helpResults.length > 0" class="space-y-3">
        <div 
          v-for="result in helpResults"
          :key="result.id"
          class="bg-gray-800 rounded-lg p-4"
        >
          <div class="flex items-start justify-between mb-2">
            <h5 class="font-semibold text-white">{{ result.title }}</h5>
            <span class="px-2 py-1 bg-blue-600 rounded text-xs text-white">{{ result.category }}</span>
          </div>
          
          <p class="text-sm text-gray-300 mb-3">{{ result.content }}</p>
          
          <div v-if="result.codeExample" class="bg-gray-900 rounded p-3 mb-3">
            <div class="text-xs text-gray-400 mb-1">Example:</div>
            <code class="text-sm text-green-400">{{ result.codeExample }}</code>
          </div>
          
          <div v-if="result.relatedTopics && result.relatedTopics.length > 0">
            <div class="text-xs text-gray-400 mb-1">Related Topics:</div>
            <div class="flex flex-wrap gap-1">
              <button 
                v-for="topic in result.relatedTopics"
                :key="topic"
                @click="helpQuery = topic"
                class="px-2 py-1 bg-gray-700 hover:bg-gray-600 rounded text-xs text-gray-300 transition-colors"
              >
                {{ topic }}
              </button>
            </div>
          </div>
        </div>
      </div>

      <div v-else-if="!isSearching" class="text-center py-8 text-gray-500">
        <HelpCircle class="h-12 w-12 mx-auto mb-3 text-gray-600" />
        <p class="text-sm">Ask me anything about programming!</p>
        <p class="text-xs mt-1">I'll provide context-aware help based on your current code</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { 
  Brain, 
  Settings, 
  RefreshCw, 
  Code, 
  Eye, 
  Search, 
  HelpCircle, 
  Zap, 
  Wrench, 
  Check, 
  X, 
  Loader, 
  ChevronRight,
  Lightbulb,
  Bug,
  FileText,
  Layers,
  AlertTriangle,
  AlertCircle,
  Info
} from 'lucide-vue-next'
import { useAIStore } from '../stores/aiStore'
import { useFileStore } from '../stores/fileStore'

const aiStore = useAIStore()
const fileStore = useFileStore()

// State
const activeTab = ref('completion')
const aiStatus = ref({
  isActive: true,
  provider: 'local',
  model: 'demo-model'
})

const completions = ref([])
const reviewResults = ref([])
const debugResults = ref(null)
const projectTemplates = ref([
  {
    id: 'rust-web-api',
    name: 'Rust Web API',
    description: 'A RESTful web API built with Actix-web and SQLx',
    language: 'Rust',
    features: [
      { id: 'database', name: 'Database Integration', description: 'SQLx with PostgreSQL' },
      { id: 'auth', name: 'Authentication', description: 'JWT-based auth system' },
      { id: 'validation', name: 'Input Validation', description: 'Request validation middleware' },
      { id: 'testing', name: 'Testing Suite', description: 'Unit and integration tests' }
    ]
  },
  {
    id: 'python-web-app',
    name: 'Python Web App',
    description: 'FastAPI web application with async support',
    language: 'Python',
    features: [
      { id: 'database', name: 'Database Integration', description: 'SQLAlchemy ORM' },
      { id: 'auth', name: 'Authentication', description: 'OAuth2 and JWT' },
      { id: 'docs', name: 'Auto Documentation', description: 'OpenAPI/Swagger docs' },
      { id: 'testing', name: 'Testing Suite', description: 'Pytest integration' }
    ]
  },
  {
    id: 'react-frontend',
    name: 'React Frontend',
    description: 'Modern React app with TypeScript and Tailwind',
    language: 'JavaScript',
    features: [
      { id: 'routing', name: 'React Router', description: 'Client-side routing' },
      { id: 'state', name: 'State Management', description: 'Redux Toolkit' },
      { id: 'ui', name: 'UI Components', description: 'Custom component library' },
      { id: 'testing', name: 'Testing Suite', description: 'Jest and React Testing Library' }
    ]
  }
])

const selectedTemplate = ref(null)
const projectConfig = ref({
  name: '',
  description: '',
  features: []
})

const debugQuery = ref('')
const helpQuery = ref('')
const helpResults = ref([])

const isAnalyzing = ref(false)
const isDebugging = ref(false)
const isGenerating = ref(false)
const isSearching = ref(false)

// Computed
const featureTabs = computed(() => [
  { id: 'completion', label: 'Smart Completion', icon: Code },
  { id: 'review', label: 'Code Review', icon: Eye },
  { id: 'debugging', label: 'Debugging', icon: Bug },
  { id: 'scaffolding', label: 'Scaffolding', icon: Layers },
  { id: 'help', label: 'Context Help', icon: HelpCircle }
])

const currentContext = computed(() => {
  const currentFile = fileStore.getCurrentFile()
  return currentFile ? {
    fileName: currentFile.name,
    language: getLanguageFromFileName(currentFile.name),
    line: 1, // Would get from editor
    column: 1 // Would get from editor
  } : null
})

const hasFixableIssues = computed(() => 
  reviewResults.value.some(result => result.canAutoFix)
)

const quickHelpTopics = computed(() => [
  { id: 'syntax', name: 'Syntax Help' },
  { id: 'errors', name: 'Common Errors' },
  { id: 'best-practices', name: 'Best Practices' },
  { id: 'performance', name: 'Performance Tips' },
  { id: 'debugging', name: 'Debugging Techniques' }
])

// Methods
function toggleAdvancedFeatures() {
  // Toggle advanced features panel
  // console.log('Toggle advanced features')
}

function clearContext() {
  completions.value = []
  reviewResults.value = []
  debugResults.value = null
  helpResults.value = []
  // console.log('Context cleared')
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

async function applyCompletion(completion: any) {
  // Apply completion to editor
  // console.log('Applying completion:', completion)
  // This would integrate with the editor to insert the completion
}

async function acceptSuggestion(completion: any) {
  await applyCompletion(completion)
  // Learn from user acceptance
  aiStore.learnFromFeedback('completion_accepted', true)
}

async function rejectSuggestion(completion: any) {
  // Remove from completions list
  completions.value = completions.value.filter(c => c.id !== completion.id)
  // Learn from user rejection
  aiStore.learnFromFeedback('completion_rejected', false)
}

async function analyzeCode() {
  if (!currentContext.value) return
  
  isAnalyzing.value = true
  try {
    const currentFile = fileStore.getCurrentFile()
    if (!currentFile) return
    
    const result = await aiStore.analyzeCode(currentFile.content, currentContext.value.language)
    reviewResults.value = result.issues || []
  } catch (error) {
    // Handle code analysis error gracefully
  } finally {
    isAnalyzing.value = false
  }
}

async function autoFixIssues() {
  // Auto-fix all fixable issues
  for (const result of reviewResults.value) {
    if (result.canAutoFix) {
      await applyFix(result)
    }
  }
}

async function applyFix(result: any) {
  // Apply the suggested fix
  // console.log('Applying fix:', result)
  // This would modify the current file content
}

function explainIssue(result: any) {
  // Show detailed explanation
  // console.log('Explaining issue:', result)
}

function getSeverityIcon(severity: string) {
  switch (severity) {
    case 'error': return AlertCircle
    case 'warning': return AlertTriangle
    case 'info': return Info
    default: return Info
  }
}

function getSeverityIconClass(severity: string) {
  switch (severity) {
    case 'error': return 'text-red-500'
    case 'warning': return 'text-yellow-500'
    case 'info': return 'text-blue-500'
    default: return 'text-gray-500'
  }
}

async function debugIssue() {
  if (!debugQuery.value.trim()) return
  
  isDebugging.value = true
  try {
    const currentFile = fileStore.getCurrentFile()
    const context = currentFile ? currentFile.content : ''
    
    const result = await aiStore.debugCode(context, currentContext.value?.language || 'rust', debugQuery.value)
    debugResults.value = {
      rootCause: result.rootCause || 'Analysis in progress...',
      steps: result.steps || [],
      prevention: result.prevention || 'Consider adding error handling and validation.'
    }
  } catch (error) {
    // Handle debugging error gracefully
  } finally {
    isDebugging.value = false
  }
}

function selectTemplate(template: any) {
  selectedTemplate.value = template
  projectConfig.value = {
    name: `${template.name} Project`,
    description: '',
    features: []
  }
}

async function generateProject() {
  if (!selectedTemplate.value || !projectConfig.value.name) return
  
  isGenerating.value = true
  try {
    // Generate project structure and files
    // console.log('Generating project:', projectConfig.value)
    
    // This would call AI to generate the project
    const result = await aiStore.generateProject({
      template: selectedTemplate.value.id,
      config: projectConfig.value
    })
    
    // console.log('Project generated:', result)
  } catch (error) {
    // Handle project generation error gracefully
  } finally {
    isGenerating.value = false
  }
}

async function searchHelp() {
  if (!helpQuery.value.trim()) return
  
  isSearching.value = true
  try {
    // Search for help based on context
    const currentFile = fileStore.getCurrentFile()
    const context = currentFile ? currentFile.content : ''
    
    const results = await aiStore.explainCode(context, currentContext.value?.language || 'rust')
    helpResults.value = [{
      id: 'help-1',
      title: 'Contextual Help',
      content: results,
      category: 'General',
      codeExample: null,
      relatedTopics: []
    }]
  } catch (error) {
    // Handle help search error gracefully
  } finally {
    isSearching.value = false
  }
}

function getQuickHelp(topic: any) {
  helpQuery.value = `Help with ${topic.name.toLowerCase()}`
  searchHelp()
}

// Watch for context changes to update completions
watch(currentContext, (newContext) => {
  if (newContext) {
    // Trigger completion update based on new context
    // console.log('Context changed:', newContext)
  }
})

// Lifecycle
onMounted(() => {
  // Initialize AI assistant
  // console.log('Advanced AI Assistant mounted')
})
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
</style>