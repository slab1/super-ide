<template>
  <div class="ai-intelligence-panel bg-gray-900 text-white p-4 h-full overflow-y-auto">
    <!-- Header -->
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-xl font-bold text-blue-400">AI Code Intelligence</h2>
      <div class="flex items-center space-x-2">
        <button
          @click="toggleAutoAnalysis"
          :class="[
            'px-3 py-1 rounded text-sm font-medium transition-colors',
            aiStore.autoAnalysis 
              ? 'bg-green-600 hover:bg-green-700 text-white' 
              : 'bg-gray-600 hover:bg-gray-700 text-gray-300'
          ]"
        >
          Auto Analysis
        </button>
        <button
          @click="runAnalysis"
          :disabled="aiStore.isAnalyzing"
          class="px-3 py-1 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 rounded text-sm font-medium transition-colors"
        >
          <span v-if="aiStore.isAnalyzing">Analyzing...</span>
          <span v-else>Analyze Code</span>
        </button>
      </div>
    </div>

    <!-- Analysis Settings -->
    <div class="mb-4 p-3 bg-gray-800 rounded-lg">
      <h3 class="text-sm font-medium text-gray-300 mb-2">Analysis Options</h3>
      <div class="grid grid-cols-2 gap-2">
        <label class="flex items-center">
          <input
            v-model="aiStore.showBugPredictions"
            type="checkbox"
            class="mr-2 rounded"
          />
          <span class="text-sm">Bug Prediction</span>
        </label>
        <label class="flex items-center">
          <input
            v-model="aiStore.showSecurityAnalysis"
            type="checkbox"
            class="mr-2 rounded"
          />
          <span class="text-sm">Security Analysis</span>
        </label>
        <label class="flex items-center">
          <input
            v-model="aiStore.showPerformanceAnalysis"
            type="checkbox"
            class="mr-2 rounded"
          />
          <span class="text-sm">Performance Analysis</span>
        </label>
      </div>
    </div>

    <!-- Overall Score -->
    <div v-if="aiStore.currentAnalysis" class="mb-4 p-4 bg-gray-800 rounded-lg">
      <div class="flex items-center justify-between mb-2">
        <h3 class="text-lg font-medium">Code Quality Score</h3>
        <div class="text-2xl font-bold" :class="scoreColor">
          {{ aiStore.overallScore.toFixed(0) }}/100
        </div>
      </div>
      <div class="w-full bg-gray-700 rounded-full h-2">
        <div 
          class="h-2 rounded-full transition-all duration-300"
          :class="scoreBarColor"
          :style="{ width: `${aiStore.overallScore}%` }"
        ></div>
      </div>
    </div>

    <!-- Error Display -->
    <div v-if="aiStore.error" class="mb-4 p-3 bg-red-900 border border-red-700 rounded-lg">
      <div class="flex items-center">
        <svg class="w-5 h-5 text-red-400 mr-2" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd"></path>
        </svg>
        <span class="text-red-200">{{ aiStore.error }}</span>
      </div>
      <button
        @click="aiStore.clearError()"
        class="mt-2 text-sm text-red-300 hover:text-red-100 underline"
      >
        Dismiss
      </button>
    </div>

    <!-- Code Issues -->
    <div v-if="aiStore.hasIssues" class="mb-4">
      <h3 class="text-lg font-medium mb-2 flex items-center">
        <svg class="w-5 h-5 mr-2 text-red-400" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd"></path>
        </svg>
        Issues ({{ aiStore.currentAnalysis?.issues.length || 0 }})
      </h3>
      <div class="space-y-2">
        <div
          v-for="issue in aiStore.currentAnalysis?.issues || []"
          :key="issue.id"
          class="p-3 rounded-lg border-l-4"
          :class="getIssueSeverityClass(issue.severity)"
        >
          <div class="flex items-start justify-between">
            <div class="flex-1">
              <div class="font-medium">{{ issue.message }}</div>
              <div class="text-sm text-gray-400 mt-1">
                Line {{ issue.line }}: Column {{ issue.column }}
                <span v-if="issue.rule_id" class="ml-2 text-xs bg-gray-700 px-1 rounded">
                  {{ issue.rule_id }}
                </span>
              </div>
            </div>
            <span 
              class="text-xs px-2 py-1 rounded font-medium"
              :class="getSeverityBadgeClass(issue.severity)"
            >
              {{ issue.severity }}
            </span>
          </div>
          <div v-if="issue.fix_suggestion" class="mt-2 text-sm text-green-300">
            💡 {{ issue.fix_suggestion }}
          </div>
        </div>
      </div>
    </div>

    <!-- Bug Predictions -->
    <div v-if="aiStore.showBugPredictions && aiStore.bugPredictions.length > 0" class="mb-4">
      <h3 class="text-lg font-medium mb-2 flex items-center">
        <svg class="w-5 h-5 mr-2 text-yellow-400" fill="currentColor" viewBox="0 0 20 20">
          <path d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"></path>
        </svg>
        Bug Predictions ({{ aiStore.bugPredictions.length }})
      </h3>
      <div class="space-y-2">
        <div
          v-for="(bug, index) in aiStore.bugPredictions"
          :key="index"
          class="p-3 bg-yellow-900 border border-yellow-700 rounded-lg"
        >
          <div class="flex items-start justify-between">
            <div class="flex-1">
              <div class="font-medium">{{ bug.description }}</div>
              <div class="text-sm text-yellow-300 mt-1">
                Line {{ bug.line }}: {{ bug.bug_type }}
              </div>
              <div class="text-sm text-gray-400 mt-1">
                Confidence: {{ (bug.confidence * 100).toFixed(0) }}%
              </div>
            </div>
            <span 
              class="text-xs px-2 py-1 rounded font-medium"
              :class="getSeverityBadgeClass(bug.severity)"
            >
              {{ bug.severity }}
            </span>
          </div>
          <div class="mt-2 text-sm text-green-300">
            💡 {{ bug.fix_suggestion }}
          </div>
        </div>
      </div>
    </div>

    <!-- Security Vulnerabilities -->
    <div v-if="aiStore.showSecurityAnalysis && aiStore.securityVulnerabilities.length > 0" class="mb-4">
      <h3 class="text-lg font-medium mb-2 flex items-center">
        <svg class="w-5 h-5 mr-2 text-red-400" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M5 9V7a5 5 0 0110 0v2a2 2 0 012 2v5a2 2 0 01-2 2H5a2 2 0 01-2-2v-5a2 2 0 012-2zm8-2v2H7V7a3 3 0 016 0z" clip-rule="evenodd"></path>
        </svg>
        Security Vulnerabilities ({{ aiStore.securityVulnerabilities.length }})
      </h3>
      <div class="space-y-2">
        <div
          v-for="vuln in aiStore.securityVulnerabilities"
          :key="vuln.id"
          class="p-3 bg-red-900 border border-red-700 rounded-lg"
        >
          <div class="flex items-start justify-between">
            <div class="flex-1">
              <div class="font-medium">{{ vuln.title }}</div>
              <div class="text-sm text-red-300 mt-1">{{ vuln.description }}</div>
              <div class="text-sm text-gray-400 mt-1">
                Line {{ vuln.line }}
                <span v-if="vuln.cwe_id" class="ml-2 text-xs bg-red-800 px-1 rounded">
                  {{ vuln.cwe_id }}
                </span>
              </div>
            </div>
            <span 
              class="text-xs px-2 py-1 rounded font-medium"
              :class="getSeverityBadgeClass(vuln.severity)"
            >
              {{ vuln.severity }}
            </span>
          </div>
          <div class="mt-2 text-sm text-green-300">
            🛡️ {{ vuln.recommendation }}
          </div>
        </div>
      </div>
    </div>

    <!-- Code Explanation -->
    <div v-if="aiStore.codeExplanation" class="mb-4">
      <h3 class="text-lg font-medium mb-2 flex items-center">
        <svg class="w-5 h-5 mr-2 text-blue-400" fill="currentColor" viewBox="0 0 20 20">
          <path d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"></path>
        </svg>
        Code Explanation
      </h3>
      <div class="p-3 bg-blue-900 border border-blue-700 rounded-lg">
        <div class="font-medium mb-2">{{ aiStore.codeExplanation.summary }}</div>
        <div class="text-sm text-blue-200 mb-3">{{ aiStore.codeExplanation.explanation }}</div>
        <div v-if="aiStore.codeExplanation.key_concepts.length > 0" class="mb-3">
          <div class="text-sm font-medium text-blue-300 mb-1">Key Concepts:</div>
          <div class="flex flex-wrap gap-1">
            <span
              v-for="concept in aiStore.codeExplanation.key_concepts"
              :key="concept"
              class="text-xs bg-blue-800 text-blue-200 px-2 py-1 rounded"
            >
              {{ concept }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- Generated Tests -->
    <div v-if="aiStore.generatedTests" class="mb-4">
      <h3 class="text-lg font-medium mb-2 flex items-center">
        <svg class="w-5 h-5 mr-2 text-green-400" fill="currentColor" viewBox="0 0 20 20">
          <path d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"></path>
        </svg>
        Generated Tests
      </h3>
      <div class="relative">
        <pre class="p-3 bg-gray-800 border border-gray-700 rounded-lg text-sm overflow-x-auto"><code>{{ aiStore.generatedTests }}</code></pre>
        <button
          @click="copyToClipboard(aiStore.generatedTests)"
          class="absolute top-2 right-2 p-1 bg-gray-700 hover:bg-gray-600 rounded text-xs"
        >
          Copy
        </button>
      </div>
    </div>

    <!-- Improvement Suggestions -->
    <div v-if="aiStore.improvementSuggestions.length > 0" class="mb-4">
      <h3 class="text-lg font-medium mb-2 flex items-center">
        <svg class="w-5 h-5 mr-2 text-purple-400" fill="currentColor" viewBox="0 0 20 20">
          <path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z"></path>
        </svg>
        Improvement Suggestions ({{ aiStore.improvementSuggestions.length }})
      </h3>
      <div class="space-y-2">
        <div
          v-for="(suggestion, index) in aiStore.improvementSuggestions"
          :key="index"
          class="p-3 bg-purple-900 border border-purple-700 rounded-lg"
        >
          {{ suggestion }}
        </div>
      </div>
    </div>

    <!-- Debug Session Controls -->
    <div v-if="aiStore.debugSession" class="mb-4">
      <h3 class="text-lg font-medium mb-2 flex items-center">
        <svg class="w-5 h-5 mr-2 text-orange-400" fill="currentColor" viewBox="0 0 20 20">
          <path d="M10 2L3 7v11h4v-6h6v6h4V7l-7-5z"></path>
        </svg>
        Debug Session
      </h3>
      <div class="p-3 bg-orange-900 border border-orange-700 rounded-lg">
        <div class="text-sm mb-2">
          <strong>Session:</strong> {{ aiStore.debugSession.session_id }}
        </div>
        <div class="text-sm mb-2">
          <strong>Current Line:</strong> {{ aiStore.debugSession.current_line || 'Not started' }}
        </div>
        <div class="flex space-x-2">
          <button
            @click="debugStep('step_over')"
            class="px-3 py-1 bg-orange-600 hover:bg-orange-700 rounded text-sm"
          >
            Step Over
          </button>
          <button
            @click="debugStep('step_into')"
            class="px-3 py-1 bg-orange-600 hover:bg-orange-700 rounded text-sm"
          >
            Step Into
          </button>
          <button
            @click="debugStep('continue')"
            class="px-3 py-1 bg-green-600 hover:bg-green-700 rounded text-sm"
          >
            Continue
          </button>
          <button
            @click="stopDebug()"
            class="px-3 py-1 bg-red-600 hover:bg-red-700 rounded text-sm"
          >
            Stop
          </button>
        </div>
      </div>
    </div>

    <!-- Debug Variables -->
    <div v-if="aiStore.debugVariables.length > 0" class="mb-4">
      <h3 class="text-lg font-medium mb-2">Variables</h3>
      <div class="space-y-1">
        <div
          v-for="variable in aiStore.debugVariables"
          :key="variable.name"
          class="p-2 bg-gray-800 border border-gray-700 rounded text-sm"
        >
          <div class="flex justify-between">
            <span class="font-medium">{{ variable.name }}</span>
            <span class="text-gray-400">{{ variable.type_name }}</span>
          </div>
          <div class="text-blue-300">{{ variable.value }}</div>
        </div>
      </div>
    </div>

    <!-- Action Buttons -->
    <div class="flex flex-wrap gap-2 mt-4">
      <button
        @click="explainCode"
        class="px-3 py-2 bg-blue-600 hover:bg-blue-700 rounded text-sm"
      >
        Explain Code
      </button>
      <button
        @click="generateTests"
        class="px-3 py-2 bg-green-600 hover:bg-green-700 rounded text-sm"
      >
        Generate Tests
      </button>
      <button
        @click="getImprovements"
        class="px-3 py-2 bg-purple-600 hover:bg-purple-700 rounded text-sm"
      >
        Get Improvements
      </button>
      <button
        v-if="!aiStore.debugSession"
        @click="startDebug"
        class="px-3 py-2 bg-orange-600 hover:bg-orange-700 rounded text-sm"
      >
        Start Debug
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useAIStore } from '../stores/aiStore'

// Store
const aiStore = useAIStore()

// Computed properties
const scoreColor = computed(() => {
  const score = aiStore.overallScore
  if (score >= 80) return 'text-green-400'
  if (score >= 60) return 'text-yellow-400'
  return 'text-red-400'
})

const scoreBarColor = computed(() => {
  const score = aiStore.overallScore
  if (score >= 80) return 'bg-green-500'
  if (score >= 60) return 'bg-yellow-500'
  return 'bg-red-500'
})

// Methods
const runAnalysis = async () => {
  try {
    // This would be called with actual code from the editor
    const sampleCode = `fn main() {
    let mut x = 42;
    x.unwrap(); // This will cause issues
    println!("Hello, world!");
}`
    await aiStore.analyzeCodeAdvanced(sampleCode, 'rust', 'main.rs')
  } catch (error) {
    console.error('Analysis failed:', error)
  }
}

const toggleAutoAnalysis = () => {
  aiStore.autoAnalysis = !aiStore.autoAnalysis
}

const getIssueSeverityClass = (severity: string) => {
  switch (severity) {
    case 'Critical': return 'bg-red-900 border-red-700'
    case 'Error': return 'bg-red-800 border-red-600'
    case 'Warning': return 'bg-yellow-900 border-yellow-700'
    default: return 'bg-blue-900 border-blue-700'
  }
}

const getSeverityBadgeClass = (severity: string) => {
  switch (severity) {
    case 'Critical': return 'bg-red-600 text-white'
    case 'Error': return 'bg-red-500 text-white'
    case 'Warning': return 'bg-yellow-500 text-black'
    default: return 'bg-blue-500 text-white'
  }
}

const copyToClipboard = async (text: string) => {
  try {
    await navigator.clipboard.writeText(text)
    // Could show a toast notification here
  } catch (error) {
    console.error('Failed to copy to clipboard:', error)
  }
}

const explainCode = async () => {
  try {
    const sampleCode = `fn main() {
    let x = 42;
    println!("Value: {}", x);
}`
    await aiStore.explainCodeEnhanced(sampleCode, 'rust')
  } catch (error) {
    console.error('Explanation failed:', error)
  }
}

const generateTests = async () => {
  try {
    const sampleCode = `fn add(a: i32, b: i32) -> i32 {
    a + b
}`
    await aiStore.generateTestsEnhanced(sampleCode, 'rust')
  } catch (error) {
    console.error('Test generation failed:', error)
  }
}

const getImprovements = async () => {
  try {
    const sampleCode = `fn main() {
    let mut data = vec![1, 2, 3];
    data.clone(); // Inefficient
    println!("{:?}", data);
}`
    await aiStore.getImprovements(sampleCode, 'rust')
  } catch (error) {
    console.error('Improvements failed:', error)
  }
}

const startDebug = async () => {
  try {
    await aiStore.startDebugSession('main.rs', 'rust', [10])
  } catch (error) {
    console.error('Debug start failed:', error)
  }
}

const debugStep = async (action: 'step_over' | 'step_into' | 'step_out' | 'continue') => {
  if (!aiStore.debugSession) return
  
  try {
    await aiStore.debugStep(aiStore.debugSession.session_id, action)
  } catch (error) {
    console.error('Debug step failed:', error)
  }
}

const stopDebug = async () => {
  if (!aiStore.debugSession) return
  
  try {
    await aiStore.stopDebugSession(aiStore.debugSession.session_id)
  } catch (error) {
    console.error('Debug stop failed:', error)
  }
}

// Initialize
onMounted(() => {
  // Could trigger initial analysis here
})
</script>

<style scoped>
/* Custom scrollbar for dark theme */
.ai-intelligence-panel::-webkit-scrollbar {
  width: 8px;
}

.ai-intelligence-panel::-webkit-scrollbar-track {
  background: #374151;
}

.ai-intelligence-panel::-webkit-scrollbar-thumb {
  background: #6b7280;
  border-radius: 4px;
}

.ai-intelligence-panel::-webkit-scrollbar-thumb:hover {
  background: #9ca3af;
}
</style>
