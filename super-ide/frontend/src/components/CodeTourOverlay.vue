<template>
  <div v-if="activeTour" class="code-tour-overlay fixed inset-0 bg-black bg-opacity-50 z-50 flex items-center justify-center">
    <div class="bg-gray-900 border border-gray-700 rounded-lg shadow-2xl max-w-4xl w-full mx-4 max-h-[90vh] overflow-hidden">
      <!-- Tour Header -->
      <div class="tour-header p-6 border-b border-gray-700">
        <div class="flex items-center justify-between">
          <div>
            <h2 class="text-xl font-semibold text-white flex items-center space-x-2">
              <PlayCircle class="w-6 h-6 text-blue-400" />
              <span>{{ activeTour.title }}</span>
            </h2>
            <p class="text-gray-400 text-sm mt-1">{{ activeTour.description }}</p>
          </div>
          <div class="flex items-center space-x-4">
            <!-- Progress Indicator -->
            <div class="text-sm text-gray-400">
              Step {{ currentStepIndex + 1 }} of {{ activeTour.steps.length }}
            </div>
            <button
              @click="$emit('tour-completed')"
              class="p-2 hover:bg-gray-700 rounded text-gray-400 hover:text-white transition-colors"
            >
              <X class="w-5 h-5" />
            </button>
          </div>
        </div>
        
        <!-- Progress Bar -->
        <div class="mt-4">
          <div class="w-full bg-gray-700 rounded-full h-2">
            <div 
              class="bg-gradient-to-r from-blue-500 to-green-500 h-2 rounded-full transition-all duration-300"
              :style="{ width: `${((currentStepIndex + 1) / activeTour.steps.length) * 100}%` }"
            ></div>
          </div>
        </div>
      </div>

      <!-- Tour Content -->
      <div class="tour-content flex">
        <!-- Code Display Area -->
        <div class="code-area flex-1 p-6 border-r border-gray-700">
          <!-- Current Code with Highlighting -->
          <div class="code-display relative">
            <!-- Step 1: File Overview -->
            <div v-if="currentStep.title === 'File Overview'" class="space-y-4">
              <h3 class="text-lg font-semibold text-white mb-4">📄 File Structure</h3>
              
              <!-- File Tree Visualization -->
              <div class="bg-gray-800 rounded-lg p-4">
                <div class="font-mono text-sm">
                  <div class="flex items-center space-x-2 text-green-400">
                    <Folder class="w-4 h-4" />
                    <span>my-project/</span>
                  </div>
                  <div class="ml-6 space-y-1">
                    <div class="flex items-center space-x-2 text-blue-400">
                      <FileText class="w-4 h-4" />
                      <span>main.rs</span>
                    </div>
                    <div class="flex items-center space-x-2 text-blue-400">
                      <FileText class="w-4 h-4" />
                      <span>lib.rs</span>
                    </div>
                    <div class="flex items-center space-x-2 text-yellow-400">
                      <FileText class="w-4 h-4" />
                      <span>Cargo.toml</span>
                    </div>
                  </div>
                </div>
              </div>

              <div class="bg-blue-900 border border-blue-700 rounded-lg p-4">
                <div class="flex items-start space-x-3">
                  <Lightbulb class="w-5 h-5 text-blue-400 mt-0.5" />
                  <div>
                    <h4 class="font-semibold text-blue-200">File Organization</h4>
                    <p class="text-blue-100 text-sm mt-1">
                      This project follows Rust's standard structure. The <code class="bg-blue-800 px-1 rounded">main.rs</code> 
                      file contains the program entry point, while <code class="bg-blue-800 px-1 rounded">lib.rs</code> 
                      defines library functions.
                    </p>
                  </div>
                </div>
              </div>
            </div>

            <!-- Step 2: Function Analysis -->
            <div v-if="currentStep.title === 'Function Analysis'" class="space-y-4">
              <h3 class="text-lg font-semibold text-white mb-4">🔍 Understanding Functions</h3>
              
              <!-- Function Declaration Highlighting -->
              <div class="bg-gray-800 rounded-lg p-4">
                <div class="font-mono text-sm">
                  <div class="text-gray-400">// Function declaration</div>
                  <div class="text-blue-400">
                    fn <span class="text-yellow-400">calculate_factorial</span>
                    <span class="text-white">(n: u64) -> u64</span> {
                  </div>
                  <div class="ml-4 text-green-400">
                    // Base case: factorial of 0 or 1 is 1
                  </div>
                  <div class="ml-4 text-white">
                    if n &lt;= <span class="text-purple-400">1</span> {
                  </div>
                  <div class="ml-8 text-green-400">
                    <span class="text-yellow-400">1</span>
                  </div>
                  <div class="ml-4 text-white">
                    } <span class="text-blue-400">else</span> {
                  </div>
                  <div class="ml-8 text-white">
                    n * <span class="text-yellow-400">calculate_factorial</span>
                    <span class="text-white">(n - 1)</span>
                  </div>
                  <div class="ml-4 text-white">
                    }
                  </div>
                  <div class="text-white">}</div>
                </div>
              </div>

              <!-- Interactive Elements -->
              <div class="space-y-2">
                <div 
                  v-for="element in currentStep.interactive_elements"
                  :key="element.id"
                  :class="[
                    'p-3 rounded-lg border cursor-pointer transition-all',
                    selectedElement === element.id 
                      ? 'bg-blue-900 border-blue-600 text-blue-200' 
                      : 'bg-gray-800 border-gray-700 text-gray-300 hover:bg-gray-750'
                  ]"
                  @click="selectInteractiveElement(element.id)"
                >
                  <div class="flex items-center space-x-2">
                    <component :is="getElementIcon(element.element_type)" class="w-4 h-4" />
                    <span class="font-medium">{{ element.title || 'Interactive Element' }}</span>
                  </div>
                  <p v-if="selectedElement === element.id" class="text-sm mt-2 text-blue-100">
                    {{ element.content }}
                  </p>
                </div>
              </div>
            </div>

            <!-- Step 3: Memory & Execution Flow -->
            <div v-if="currentStep.title === 'Execution Flow'" class="space-y-4">
              <h3 class="text-lg font-semibold text-white mb-4">⚡ Execution Flow Visualization</h3>
              
              <!-- Call Stack Visualization -->
              <div class="bg-gray-800 rounded-lg p-4">
                <h4 class="font-semibold text-white mb-3">Call Stack</h4>
                <div class="space-y-2">
                  <div 
                    v-for="(frame, index) in callStack"
                    :key="index"
                    :class="[
                      'p-2 rounded border text-sm',
                      index === callStack.length - 1 
                        ? 'bg-yellow-900 border-yellow-600 text-yellow-200' 
                        : 'bg-gray-700 border-gray-600 text-gray-300'
                    ]"
                  >
                    <div class="font-mono">
                      calculate_factorial({{ frame.n }})
                      <span v-if="index === callStack.length - 1" class="text-yellow-400 animate-pulse">
                        ← Current
                      </span>
                    </div>
                  </div>
                </div>
              </div>

              <!-- Memory State -->
              <div class="bg-gray-800 rounded-lg p-4">
                <h4 class="font-semibold text-white mb-3">Memory State</h4>
                <div class="grid grid-cols-2 gap-4">
                  <div>
                    <h5 class="text-sm font-medium text-gray-300 mb-2">Local Variables</h5>
                    <div class="space-y-1 text-sm">
                      <div class="flex justify-between">
                        <span class="text-gray-400">n:</span>
                        <span class="text-white">{{ currentMemoryState.n }}</span>
                      </div>
                      <div class="flex justify-between">
                        <span class="text-gray-400">result:</span>
                        <span class="text-white">{{ currentMemoryState.result }}</span>
                      </div>
                    </div>
                  </div>
                  <div>
                    <h5 class="text-sm font-medium text-gray-300 mb-2">Return Value</h5>
                    <div class="text-lg font-mono text-green-400">
                      {{ currentMemoryState.returnValue }}
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- Step 4: Algorithm Complexity -->
            <div v-if="currentStep.title === 'Algorithm Analysis'" class="space-y-4">
              <h3 class="text-lg font-semibold text-white mb-4">📊 Algorithm Analysis</h3>
              
              <!-- Complexity Chart -->
              <div class="bg-gray-800 rounded-lg p-4">
                <h4 class="font-semibold text-white mb-3">Time Complexity</h4>
                <div class="text-center">
                  <div class="text-3xl font-bold text-red-400 mb-2">O(n)</div>
                  <p class="text-gray-400 text-sm">Linear time complexity</p>
                </div>
                
                <!-- Visual complexity representation -->
                <div class="mt-4">
                  <div class="flex justify-between items-end space-x-1 h-16">
                    <div 
                      v-for="(height, index) in complexityBars"
                      :key="index"
                      :class="[
                        'bg-blue-500 transition-all duration-300',
                        index <= currentComplexityIndex ? 'opacity-100' : 'opacity-30'
                      ]"
                      :style="{ height: `${height}%` }"
                    ></div>
                  </div>
                </div>
              </div>

              <!-- Trade-offs -->
              <div class="bg-yellow-900 border border-yellow-700 rounded-lg p-4">
                <div class="flex items-start space-x-3">
                  <AlertTriangle class="w-5 h-5 text-yellow-400 mt-0.5" />
                  <div>
                    <h4 class="font-semibold text-yellow-200">Trade-offs</h4>
                    <ul class="text-yellow-100 text-sm mt-2 space-y-1">
                      <li>• Simple to understand and implement</li>
                      <li>• Inefficient for large numbers (repeated calculations)</li>
                      <li>• Consider dynamic programming for optimization</li>
                    </ul>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Explanation Sidebar -->
        <div class="explanation-sidebar w-80 p-6">
          <div class="sticky top-0">
            <!-- Step Explanation -->
            <div class="explanation-content mb-6">
              <h3 class="text-lg font-semibold text-white mb-3">{{ currentStep.title }}</h3>
              <p class="text-gray-300 text-sm leading-relaxed">{{ currentStep.explanation }}</p>
              
              <!-- Key Concepts -->
              <div v-if="currentStep.highlighted_concepts?.length" class="mt-4">
                <h4 class="text-sm font-medium text-gray-400 mb-2">Key Concepts:</h4>
                <div class="space-y-1">
                  <span 
                    v-for="concept in currentStep.highlighted_concepts"
                    :key="concept"
                    class="inline-block bg-blue-900 text-blue-200 text-xs px-2 py-1 rounded mr-2 mb-1"
                  >
                    {{ concept }}
                  </span>
                </div>
              </div>
            </div>

            <!-- Navigation Controls -->
            <div class="navigation-controls space-y-3">
              <button
                @click="previousStep"
                :disabled="currentStepIndex === 0"
                class="w-full flex items-center justify-center space-x-2 px-4 py-2 bg-gray-700 hover:bg-gray-600 disabled:bg-gray-800 disabled:text-gray-500 disabled:cursor-not-allowed text-white rounded-lg transition-colors"
              >
                <ChevronLeft class="w-4 h-4" />
                <span>Previous</span>
              </button>
              
              <button
                @click="nextStep"
                :disabled="currentStepIndex === activeTour.steps.length - 1"
                class="w-full flex items-center justify-center space-x-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-800 disabled:text-gray-500 disabled:cursor-not-allowed text-white rounded-lg transition-colors"
              >
                <span>Next</span>
                <ChevronRight class="w-4 h-4" />
              </button>
              
              <button
                @click="$emit('tour-completed')"
                class="w-full px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg transition-colors"
              >
                Complete Tour
              </button>
            </div>

            <!-- Quick Actions -->
            <div class="quick-actions mt-6">
              <h4 class="text-sm font-medium text-gray-400 mb-3">Quick Actions</h4>
              <div class="space-y-2">
                <button
                  @click="showCodeExample"
                  class="w-full flex items-center space-x-2 px-3 py-2 bg-gray-700 hover:bg-gray-600 text-gray-300 rounded text-sm transition-colors"
                >
                  <Code class="w-4 h-4" />
                  <span>Show Example</span>
                </button>
                <button
                  @click="runInteractiveDemo"
                  class="w-full flex items-center space-x-2 px-3 py-2 bg-gray-700 hover:bg-gray-600 text-gray-300 rounded text-sm transition-colors"
                >
                  <Play class="w-4 h-4" />
                  <span>Try It Yourself</span>
                </button>
                <button
                  @click="askAITutor"
                  class="w-full flex items-center space-x-2 px-3 py-2 bg-gray-700 hover:bg-gray-600 text-gray-300 rounded text-sm transition-colors"
                >
                  <MessageCircle class="w-4 h-4" />
                  <span>Ask AI Tutor</span>
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { 
  PlayCircle, 
  X, 
  ChevronLeft, 
  ChevronRight, 
  Folder, 
  FileText, 
  Lightbulb, 
  AlertTriangle, 
  Code, 
  Play, 
  MessageCircle,
  HelpCircle,
  Zap,
  TrendingUp,
  Database
} from 'lucide-vue-next'

// Props
interface Props {
  tour: {
    id: string
    title: string
    description: string
    steps: TourStep[]
  }
  currentStep: number
}

interface TourStep {
  step_number: number
  title: string
  explanation: string
  highlighted_concepts?: string[]
  visual_aids?: any[]
  interactive_elements?: InteractiveElement[]
}

interface InteractiveElement {
  id: string
  element_type: string
  content: string
}

const props = defineProps<Props>()

// Emits
const emit = defineEmits<{
  'tour-completed': []
  'next-step': []
  'previous-step': []
}>()

// Reactive state
const currentStepIndex = computed({
  get: () => props.currentStep,
  set: (value) => emit('update:currentStep', value)
})

const selectedElement = ref<string | null>(null)

// Computed current step
const currentStep = computed(() => {
  return props.tour.steps[props.currentStep] || props.tour.steps[0]
})

// Mock data for visualizations
const callStack = ref([
  { n: 5, level: 5 },
  { n: 4, level: 4 },
  { n: 3, level: 3 },
  { n: 2, level: 2 },
  { n: 1, level: 1 }
])

const currentMemoryState = ref({
  n: 3,
  result: 6,
  returnValue: 6
})

const complexityBars = ref([20, 40, 60, 80, 100, 80, 60, 40, 20])
const currentComplexityIndex = ref(4)

// Methods
const nextStep = () => {
  if (currentStepIndex.value < props.tour.steps.length - 1) {
    currentStepIndex.value++
    updateVisualization()
    emit('next-step')
  }
}

const previousStep = () => {
  if (currentStepIndex.value > 0) {
    currentStepIndex.value--
    updateVisualization()
    emit('previous-step')
  }
}

const updateVisualization = () => {
  // Update visualizations based on current step
  switch (currentStep.value?.title) {
    case 'Execution Flow':
      // Update call stack for current step
      break
    case 'Algorithm Analysis':
      // Animate complexity bars
      animateComplexityBars()
      break
  }
}

const animateComplexityBars = () => {
  // Animate complexity visualization
  currentComplexityIndex.value = 0
  const interval = setInterval(() => {
    if (currentComplexityIndex.value < complexityBars.value.length - 1) {
      currentComplexityIndex.value++
    } else {
      clearInterval(interval)
    }
  }, 200)
}

const selectInteractiveElement = (elementId: string) => {
  selectedElement.value = selectedElement.value === elementId ? null : elementId
}

const getElementIcon = (elementType: string) => {
  const icons = {
    'VariableInspector': Database,
    'FunctionCallTrace': Zap,
    'CodeHighlight': Code,
    'Tooltip': HelpCircle,
    'ExpandableExplanation': MessageCircle
  }
  return icons[elementType] || HelpCircle
}

const showCodeExample = () => {
  // Show modal with code example
  console.log('Show code example')
}

const runInteractiveDemo = () => {
  // Open interactive demo in new panel
  console.log('Run interactive demo')
}

const askAITutor = () => {
  // Send context to AI tutor
  console.log('Ask AI tutor about current step')
}

// Watch for step changes
watch(() => props.currentStep, (newStep) => {
  selectedElement.value = null
  updateVisualization()
})
</script>

<style scoped>
.code-tour-overlay {
  backdrop-filter: blur(8px);
}

.tour-content {
  height: 500px;
}

.code-display {
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
}

.explanation-sidebar {
  background: linear-gradient(180deg, #1f2937 0%, #111827 100%);
}

/* Code highlighting animations */
.code-highlight {
  animation: highlightFade 2s ease-in-out;
}

@keyframes highlightFade {
  0% {
    background-color: rgba(59, 130, 246, 0.3);
  }
  50% {
    background-color: rgba(59, 130, 246, 0.1);
  }
  100% {
    background-color: transparent;
  }
}

/* Interactive element hover effects */
.interactive-element {
  transition: all 0.2s ease;
}

.interactive-element:hover {
  transform: translateX(4px);
}

/* Complexity bar animations */
.complexity-bar {
  transition: height 0.5s ease-out, opacity 0.3s ease;
}

/* Navigation button hover effects */
.navigation-controls button {
  transition: all 0.2s ease;
}

.navigation-controls button:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

/* Scrollbar styling */
.explanation-content {
  scrollbar-width: thin;
  scrollbar-color: #4b5563 #1f2937;
}

.explanation-content::-webkit-scrollbar {
  width: 4px;
}

.explanation-content::-webkit-scrollbar-track {
  background: #1f2937;
}

.explanation-content::-webkit-scrollbar-thumb {
  background: #4b5563;
  border-radius: 2px;
}

.explanation-content::-webkit-scrollbar-thumb:hover {
  background: #6b7280;
}

/* Concept badge animations */
.concept-badge {
  animation: slideIn 0.3s ease-out;
}

@keyframes slideIn {
  0% {
    transform: translateX(-10px);
    opacity: 0;
  }
  100% {
    transform: translateX(0);
    opacity: 1;
  }
}

/* Memory state visualization */
.memory-frame {
  animation: memoryPulse 2s ease-in-out infinite;
}

@keyframes memoryPulse {
  0%, 100% {
    border-color: #6b7280;
  }
  50% {
    border-color: #f59e0b;
  }
}
</style>