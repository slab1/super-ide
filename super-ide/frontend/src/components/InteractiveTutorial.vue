<template>
  <div class="interactive-tutorial fixed inset-0 bg-black bg-opacity-75 z-50 flex items-center justify-center">
    <div class="bg-gray-900 border border-gray-700 rounded-lg shadow-2xl max-w-6xl w-full mx-4 max-h-[95vh] overflow-hidden">
      <!-- Tutorial Header -->
      <div class="tutorial-header p-6 border-b border-gray-700">
        <div class="flex items-center justify-between">
          <div>
            <h2 class="text-2xl font-bold text-white flex items-center">
              <GraduationCap class="w-6 h-6 mr-3 text-blue-400" />
              {{ tutorial.title }}
            </h2>
            <p class="text-gray-400 mt-1">{{ tutorial.description }}</p>
          </div>
          <button
            @click="$emit('tutorial-completed')"
            class="p-2 hover:bg-gray-700 rounded text-gray-400 hover:text-white transition-colors"
          >
            <X class="w-6 h-6" />
          </button>
        </div>
        
        <!-- Progress -->
        <div class="mt-4">
          <div class="flex justify-between text-sm text-gray-400 mb-2">
            <span>Progress</span>
            <span>{{ currentExerciseIndex + 1 }} of {{ tutorial.exercises?.length || 0 }}</span>
          </div>
          <div class="w-full bg-gray-700 rounded-full h-2">
            <div 
              class="bg-gradient-to-r from-blue-500 to-purple-500 h-2 rounded-full transition-all duration-300"
              :style="{ width: `${((currentExerciseIndex + 1) / (tutorial.exercises?.length || 1)) * 100}%` }"
            ></div>
          </div>
        </div>
      </div>

      <!-- Tutorial Content -->
      <div class="tutorial-content flex">
        <!-- Exercise Panel -->
        <div class="flex-1 p-6 border-r border-gray-700">
          <!-- Current Exercise -->
          <div v-if="currentExercise" class="exercise-panel">
            <div class="flex items-center justify-between mb-4">
              <h3 class="text-xl font-semibold text-white">{{ currentExercise.title }}</h3>
              <div class="flex items-center space-x-2">
                <span 
                  :class="[
                    'px-3 py-1 rounded-full text-sm font-medium',
                    exerciseStatus === 'completed' ? 'bg-green-900 text-green-200' :
                    exerciseStatus === 'in-progress' ? 'bg-yellow-900 text-yellow-200' :
                    'bg-gray-700 text-gray-300'
                  ]"
                >
                  {{ getExerciseStatusText() }}
                </span>
                <button
                  @click="requestExerciseHint"
                  class="p-2 bg-yellow-600 hover:bg-yellow-700 rounded-lg transition-colors"
                  title="Get hint"
                >
                  <Lightbulb class="w-4 h-4" />
                </button>
              </div>
            </div>

            <p class="text-gray-300 mb-6">{{ currentExercise.description }}</p>
            
            <!-- Instructions -->
            <div class="instructions mb-6">
              <h4 class="text-lg font-medium text-white mb-3">Instructions</h4>
              <div class="bg-gray-800 rounded-lg p-4">
                <p class="text-gray-300 whitespace-pre-line">{{ currentExercise.instructions }}</p>
              </div>
            </div>

            <!-- Code Editor -->
            <div class="code-editor-section mb-6">
              <div class="flex items-center justify-between mb-3">
                <h4 class="text-lg font-medium text-white">Your Code</h4>
                <div class="flex space-x-2">
                  <button
                    @click="resetCode"
                    class="px-3 py-1 bg-gray-600 hover:bg-gray-700 text-white text-sm rounded transition-colors"
                  >
                    Reset
                  </button>
                  <button
                    @click="runCode"
                    :disabled="isRunning"
                    class="px-3 py-1 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 text-white text-sm rounded transition-colors"
                  >
                    {{ isRunning ? 'Running...' : 'Run Code' }}
                  </button>
                </div>
              </div>
              <div class="bg-gray-800 rounded-lg overflow-hidden">
                <div class="px-4 py-2 bg-gray-700 border-b border-gray-600">
                  <span class="text-sm text-gray-300">starter_code.rs</span>
                </div>
                <textarea
                  v-model="userCode"
                  @input="onCodeChange"
                  class="w-full h-64 p-4 bg-gray-900 text-green-400 font-mono text-sm resize-none focus:outline-none"
                  placeholder="Write your code here..."
                ></textarea>
              </div>
            </div>

            <!-- Test Results -->
            <div v-if="testResults" class="test-results mb-6">
              <h4 class="text-lg font-medium text-white mb-3">Test Results</h4>
              <div class="space-y-2">
                <div
                  v-for="(result, index) in testResults"
                  :key="index"
                  :class="[
                    'p-3 rounded-lg border',
                    result.passed ? 'bg-green-900 border-green-700' : 'bg-red-900 border-red-700'
                  ]"
                >
                  <div class="flex items-center justify-between">
                    <span class="text-white font-medium">{{ result.description }}</span>
                    <div class="flex items-center space-x-2">
                      <span 
                        :class="[
                          'w-4 h-4 rounded-full flex items-center justify-center text-xs',
                          result.passed ? 'bg-green-500' : 'bg-red-500'
                        ]"
                      >
                        {{ result.passed ? '✓' : '✗' }}
                      </span>
                      <span class="text-gray-300 text-sm">{{ result.message }}</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- Action Buttons -->
            <div class="flex justify-between">
              <button
                @click="previousExercise"
                :disabled="currentExerciseIndex === 0"
                class="px-6 py-3 bg-gray-600 hover:bg-gray-700 disabled:bg-gray-800 disabled:text-gray-500 text-white font-medium rounded-lg transition-colors"
              >
                Previous
              </button>
              
              <div class="flex space-x-3">
                <button
                  v-if="exerciseStatus === 'completed'"
                  @click="nextExercise"
                  :disabled="currentExerciseIndex === (tutorial.exercises?.length || 1) - 1"
                  class="px-6 py-3 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-800 disabled:text-gray-500 text-white font-medium rounded-lg transition-colors"
                >
                  Next Exercise
                </button>
                
                <button
                  @click="submitSolution"
                  :disabled="!userCode.trim() || isSubmitting"
                  class="px-6 py-3 bg-green-600 hover:bg-green-700 disabled:bg-gray-800 disabled:text-gray-500 text-white font-medium rounded-lg transition-colors"
                >
                  {{ isSubmitting ? 'Submitting...' : 'Submit Solution' }}
                </button>
              </div>
            </div>
          </div>

          <!-- No Exercise Selected -->
          <div v-else class="text-center py-12">
            <GraduationCap class="w-16 h-16 mx-auto mb-4 text-gray-500" />
            <h3 class="text-lg font-medium text-white mb-2">Tutorial Complete!</h3>
            <p class="text-gray-400 mb-6">You've completed all exercises in this tutorial.</p>
            <button
              @click="$emit('tutorial-completed')"
              class="px-6 py-3 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-lg transition-colors"
            >
              Finish Tutorial
            </button>
          </div>
        </div>

        <!-- Hint & Progress Panel -->
        <div class="w-80 p-6 bg-gray-800">
          <div class="sticky top-0">
            <!-- Current Hint -->
            <div v-if="currentHint" class="hint-panel mb-6">
              <h4 class="text-lg font-medium text-white mb-3 flex items-center">
                <Lightbulb class="w-5 h-5 mr-2 text-yellow-400" />
                Hint
              </h4>
              <div class="bg-yellow-900 border border-yellow-700 rounded-lg p-4">
                <p class="text-yellow-100">{{ currentHint }}</p>
              </div>
            </div>

            <!-- Exercise List -->
            <div class="exercise-list">
              <h4 class="text-lg font-medium text-white mb-3">Exercises</h4>
              <div class="space-y-2">
                <div
                  v-for="(exercise, index) in tutorial.exercises"
                  :key="index"
                  @click="goToExercise(index)"
                  :class="[
                    'p-3 rounded-lg cursor-pointer transition-all border',
                    index === currentExerciseIndex
                      ? 'bg-blue-900 border-blue-600 text-blue-200'
                      : getExerciseStatus(index) === 'completed'
                      ? 'bg-green-900 border-green-700 text-green-200'
                      : 'bg-gray-700 border-gray-600 text-gray-300 hover:bg-gray-600'
                  ]"
                >
                  <div class="flex items-center justify-between">
                    <span class="font-medium text-sm">{{ exercise.title }}</span>
                    <div class="flex items-center space-x-1">
                      <div
                        v-if="getExerciseStatus(index) === 'completed'"
                        class="text-green-400"
                      >
                        ✓
                      </div>
                      <div
                        v-if="getExerciseStatus(index) === 'in-progress'"
                        class="text-yellow-400 animate-pulse"
                      >
                        ⚡
                      </div>
                    </div>
                  </div>
                  <p class="text-xs mt-1 opacity-80">{{ exercise.description.substring(0, 60) }}...</p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { 
  GraduationCap, 
  X, 
  Lightbulb 
} from 'lucide-vue-next'

// Props
interface Props {
  tutorial: {
    id: string
    title: string
    description: string
    exercises?: any[]
  }
}

const props = defineProps<Props>()

// Emits
const emit = defineEmits<{
  'tutorial-completed': []
  'exercise-completed': [exerciseId: string]
}>()

// Reactive state
const currentExerciseIndex = ref(0)
const userCode = ref('')
const currentHint = ref('')
const testResults = ref<any[]>([])
const isRunning = ref(false)
const isSubmitting = ref(false)
const exerciseProgress = ref<Record<number, string>>({})

// Computed properties
const currentExercise = computed(() => {
  return props.tutorial.exercises?.[currentExerciseIndex.value] || null
})

const exerciseStatus = computed(() => {
  return exerciseProgress.value[currentExerciseIndex.value] || 'not-started'
})

// Methods
const getExerciseStatus = (index: number) => {
  return exerciseProgress.value[index] || 'not-started'
}

const getExerciseStatusText = () => {
  switch (exerciseStatus.value) {
    case 'completed': return 'Completed'
    case 'in-progress': return 'In Progress'
    default: return 'Not Started'
  }
}

const onCodeChange = () => {
  // Mark as in-progress
  exerciseProgress.value[currentExerciseIndex.value] = 'in-progress'
}

const resetCode = () => {
  if (currentExercise.value) {
    userCode.value = currentExercise.value.starter_code || ''
  }
}

const runCode = async () => {
  isRunning.value = true
  
  try {
    // Simulate code execution
    await new Promise(resolve => setTimeout(resolve, 2000))
    
    // Mock test results
    testResults.value = [
      {
        passed: true,
        description: 'Function compiles successfully',
        message: 'No compilation errors'
      },
      {
        passed: userCode.value.includes('return'),
        description: 'Returns a value',
        message: userCode.value.includes('return') ? 'Returns a value' : 'Missing return statement'
      },
      {
        passed: userCode.value.includes('fn'),
        description: 'Defines a function',
        message: 'Function definition found'
      }
    ]
    
  } catch (error) {
    console.error('Code execution error:', error)
    testResults.value = [{
      passed: false,
      description: 'Execution failed',
      message: 'An error occurred during execution'
    }]
  } finally {
    isRunning.value = false
  }
}

const submitSolution = async () => {
  isSubmitting.value = true
  
  try {
    // Simulate solution validation
    await new Promise(resolve => setTimeout(resolve, 1500))
    
    // Mark as completed
    exerciseProgress.value[currentExerciseIndex.value] = 'completed'
    
    // Emit completion event
    emit('exercise-completed', currentExercise.value?.id || '')
    
  } catch (error) {
    console.error('Solution submission error:', error)
  } finally {
    isSubmitting.value = false
  }
}

const requestExerciseHint = () => {
  if (currentExercise.value?.hints?.length) {
    const hintIndex = Math.min(
      exerciseProgress.value[currentExerciseIndex.value] === 'in-progress' ? 1 : 0,
      currentExercise.value.hints.length - 1
    )
    currentHint.value = currentExercise.value.hints[hintIndex]
  }
}

const previousExercise = () => {
  if (currentExerciseIndex.value > 0) {
    currentExerciseIndex.value--
    loadExercise()
  }
}

const nextExercise = () => {
  if (currentExerciseIndex.value < (props.tutorial.exercises?.length || 1) - 1) {
    currentExerciseIndex.value++
    loadExercise()
  }
}

const goToExercise = (index: number) => {
  currentExerciseIndex.value = index
  loadExercise()
}

const loadExercise = () => {
  if (currentExercise.value) {
    userCode.value = currentExercise.value.starter_code || ''
    currentHint.value = ''
    testResults.value = []
  }
}

// Lifecycle
onMounted(() => {
  loadExercise()
})
</script>

<style scoped>
.interactive-tutorial {
  backdrop-filter: blur(8px);
}

.tutorial-content {
  height: 600px;
}

/* Textarea styling */
textarea {
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  line-height: 1.5;
}

/* Exercise list animations */
.exercise-item {
  transition: all 0.2s ease;
}

.exercise-item:hover {
  transform: translateY(-1px);
}

/* Progress bar animations */
.progress-bar {
  transition: width 0.3s ease;
}

/* Test result animations */
.test-result {
  animation: slideInRight 0.3s ease-out;
}

@keyframes slideInRight {
  0% {
    transform: translateX(20px);
    opacity: 0;
  }
  100% {
    transform: translateX(0);
    opacity: 1;
  }
}

/* Hint panel styling */
.hint-panel {
  animation: slideInDown 0.3s ease-out;
}

@keyframes slideInDown {
  0% {
    transform: translateY(-10px);
    opacity: 0;
  }
  100% {
    transform: translateY(0);
    opacity: 1;
  }
}

/* Button hover effects */
button:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

/* Status indicators */
.status-completed {
  animation: successGlow 2s ease-in-out infinite;
}

@keyframes successGlow {
  0%, 100% {
    box-shadow: 0 0 0 0 rgba(34, 197, 94, 0.4);
  }
  50% {
    box-shadow: 0 0 0 8px rgba(34, 197, 94, 0);
  }
}
</style>