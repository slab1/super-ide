<template>
  <div class="learning-panel bg-gray-900 text-white h-full flex flex-col">
    <!-- Header -->
    <div class="learning-header p-4 border-b border-gray-700">
      <div class="flex items-center justify-between">
        <div>
          <h2 class="text-xl font-semibold text-blue-400">🎓 Learning Mode</h2>
          <p class="text-sm text-gray-400" v-if="currentPath">
            {{ currentPath.title }} • {{ completedConcepts }}/{{ totalConcepts }} concepts
          </p>
        </div>
        <div class="flex items-center space-x-2">
          <button
            @click="toggleLearningMode"
            :class="[
              'px-3 py-1 rounded text-sm font-medium transition-colors',
              learningModeActive 
                ? 'bg-green-600 hover:bg-green-700 text-white' 
                : 'bg-gray-700 hover:bg-gray-600 text-gray-300'
            ]"
          >
            {{ learningModeActive ? '🟢 Active' : '⚪ Inactive' }}
          </button>
          <button
            @click="showSettings = !showSettings"
            class="p-2 hover:bg-gray-700 rounded"
          >
            <Settings class="w-4 h-4" />
          </button>
        </div>
      </div>
      
      <!-- Progress Bar -->
      <div v-if="currentPath" class="mt-3">
        <div class="flex justify-between text-xs text-gray-400 mb-1">
          <span>Progress</span>
          <span>{{ Math.round((completedConcepts / totalConcepts) * 100) }}%</span>
        </div>
        <div class="w-full bg-gray-700 rounded-full h-2">
          <div 
            class="bg-gradient-to-r from-blue-500 to-green-500 h-2 rounded-full transition-all duration-500"
            :style="{ width: `${(completedConcepts / totalConcepts) * 100}%` }"
          ></div>
        </div>
      </div>
    </div>

    <!-- Learning Content -->
    <div class="flex-1 flex overflow-hidden">
      <!-- Sidebar: Learning Paths & Concepts -->
      <div class="learning-sidebar w-80 border-r border-gray-700 overflow-y-auto">
        <!-- Learning Path Selector -->
        <div class="p-4 border-b border-gray-700">
          <h3 class="text-sm font-medium text-gray-300 mb-3">Learning Paths</h3>
          <div class="space-y-2">
            <div
              v-for="path in availablePaths"
              :key="path.id"
              @click="selectLearningPath(path)"
              :class="[
                'p-3 rounded-lg cursor-pointer transition-all border',
                currentPath?.id === path.id
                  ? 'bg-blue-600 border-blue-500 text-white'
                  : 'bg-gray-800 border-gray-700 hover:bg-gray-750 text-gray-300'
              ]"
            >
              <div class="flex items-center justify-between">
                <div>
                  <h4 class="font-medium">{{ path.title }}</h4>
                  <p class="text-xs text-gray-400 mt-1">{{ path.description }}</p>
                </div>
                <div class="text-xs text-gray-400">
                  {{ formatDuration(path.estimated_total_duration) }}
                </div>
              </div>
              <div class="mt-2">
                <div class="text-xs text-gray-400">
                  {{ path.modules.length }} modules • 
                  {{ getPathProgress(path.id) }}% complete
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Current Module Concepts -->
        <div v-if="currentModule" class="p-4">
          <h3 class="text-sm font-medium text-gray-300 mb-3">Current Module</h3>
          <div class="space-y-2">
            <div
              v-for="(concept, index) in currentModule.concepts"
              :key="concept.id"
              @click="selectConcept(concept)"
              :class="[
                'p-3 rounded-lg cursor-pointer transition-all border',
                currentConcept?.id === concept.id
                  ? 'bg-green-600 border-green-500 text-white'
                  : getConceptStatus(concept.id) === 'mastered'
                  ? 'bg-green-800 border-green-700 text-green-200'
                  : getConceptStatus(concept.id) === 'in-progress'
                  ? 'bg-yellow-800 border-yellow-700 text-yellow-200'
                  : 'bg-gray-800 border-gray-700 text-gray-300 hover:bg-gray-750'
              ]"
            >
              <div class="flex items-center justify-between">
                <div class="flex items-center space-x-2">
                  <div 
                    :class="[
                      'w-6 h-6 rounded-full flex items-center justify-center text-xs font-bold',
                      getConceptStatus(concept.id) === 'mastered' ? 'bg-green-500' :
                      getConceptStatus(concept.id) === 'in-progress' ? 'bg-yellow-500' : 'bg-gray-600'
                    ]"
                  >
                    {{ index + 1 }}
                  </div>
                  <div>
                    <h4 class="font-medium text-sm">{{ concept.name }}</h4>
                    <p class="text-xs text-gray-400 mt-1">{{ concept.explanation.substring(0, 80) }}...</p>
                  </div>
                </div>
                <div class="flex items-center space-x-1">
                  <div
                    v-if="getConceptStatus(concept.id) === 'mastered'"
                    class="text-green-400"
                  >
                    ✓
                  </div>
                  <div
                    v-if="getConceptStatus(concept.id) === 'in-progress'"
                    class="text-yellow-400 animate-pulse"
                  >
                    ⚡
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Achievement Showcase -->
        <div class="p-4 border-t border-gray-700">
          <h3 class="text-sm font-medium text-gray-300 mb-3">Recent Achievements</h3>
          <div class="space-y-2">
            <div
              v-for="achievement in recentAchievements"
              :key="achievement.id"
              class="flex items-center space-x-3 p-2 bg-gray-800 rounded-lg"
            >
              <div class="text-2xl">{{ achievement.icon }}</div>
              <div>
                <h4 class="font-medium text-sm">{{ achievement.title }}</h4>
                <p class="text-xs text-gray-400">{{ achievement.description }}</p>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Main Content Area -->
      <div class="flex-1 flex flex-col">
        <!-- Current Concept Display -->
        <div v-if="currentConcept" class="flex-1 overflow-y-auto">
          <ConceptViewer 
            :concept="currentConcept"
            :code-context="currentCode"
            :student-progress="studentProgress"
            @concept-completed="onConceptCompleted"
            @request-help="onRequestHelp"
            @next-concept="onNextConcept"
          />
        </div>

        <!-- Code Tour Overlay -->
        <CodeTourOverlay
          v-if="activeTour"
          :tour="activeTour"
          :current-step="tourStep"
          @tour-completed="onTourCompleted"
          @next-step="onNextTourStep"
        />

        <!-- Interactive Tutorial -->
        <InteractiveTutorial
          v-if="currentTutorial"
          :tutorial="currentTutorial"
          @tutorial-completed="onTutorialCompleted"
          @exercise-completed="onExerciseCompleted"
        />

        <!-- Default Welcome Screen -->
        <div v-if="!currentConcept && !activeTour && !currentTutorial" class="flex-1 flex items-center justify-center">
          <div class="text-center max-w-md">
            <GraduationCap class="mx-auto h-16 w-16 text-blue-400 mb-4" />
            <h3 class="text-xl font-semibold mb-2">Welcome to Super IDE Learning</h3>
            <p class="text-gray-400 mb-6">
              Select a learning path to begin your programming journey with AI-powered guidance.
            </p>
            <button
              @click="startQuickStart"
              class="px-6 py-3 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-lg transition-colors"
            >
              Start Quick Start Guide
            </button>
          </div>
        </div>
      </div>

      <!-- AI Tutor Panel -->
      <AITutorPanel
        v-if="aiTutorVisible"
        :tutor="currentTutor"
        :context="tutorContext"
        @close="aiTutorVisible = false"
      />
    </div>

    <!-- Floating Action Buttons -->
    <div class="fixed bottom-6 right-6 flex flex-col space-y-3">
      <button
        @click="toggleAITutor"
        :class="[
          'p-4 rounded-full shadow-lg transition-all',
          aiTutorVisible 
            ? 'bg-green-600 hover:bg-green-700 text-white' 
            : 'bg-blue-600 hover:bg-blue-700 text-white'
        ]"
        title="AI Tutor"
      >
        <MessageCircle class="w-6 h-6" />
      </button>
      
      <button
        v-if="currentCode"
        @click="startCodeTour"
        class="p-4 bg-purple-600 hover:bg-purple-700 text-white rounded-full shadow-lg transition-all"
        title="Code Tour"
      >
        <Play class="w-6 h-6" />
      </button>
      
      <button
        @click="showProgressModal = true"
        class="p-4 bg-gray-600 hover:bg-gray-700 text-white rounded-full shadow-lg transition-all"
        title="Learning Progress"
      >
        <BarChart3 class="w-6 h-6" />
      </button>
    </div>

    <!-- Modals -->
    <LearningSettings
      v-if="showSettings"
      :student-profile="studentProfile"
      @close="showSettings = false"
      @settings-updated="onSettingsUpdated"
    />

    <LearningProgressModal
      v-if="showProgressModal"
      :student-profile="studentProfile"
      :learning-paths="availablePaths"
      @close="showProgressModal = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { 
  Settings, 
  GraduationCap, 
  MessageCircle, 
  Play, 
  BarChart3 
} from 'lucide-vue-next'

// Import learning components (to be created)
import ConceptViewer from './ConceptViewer.vue'
import CodeTourOverlay from './CodeTourOverlay.vue'
import InteractiveTutorial from './InteractiveTutorial.vue'
import AITutorPanel from './AITutorPanel.vue'
import LearningSettings from './LearningSettings.vue'
import LearningProgressModal from './LearningProgressModal.vue'

// Types
interface LearningPath {
  id: string
  title: string
  description: string
  modules: string[]
  estimated_total_duration: number
  target_audience: string
  outcomes: string[]
}

interface Concept {
  id: string
  name: string
  explanation: string
  code_examples: any[]
  visual_aids: any[]
  interactive_demos: any[]
}

interface StudentProfile {
  id: string
  name: string
  learning_style: string
  current_level: string
  progress: Record<string, any>
  preferences: any
  achievements: any[]
}

interface CodeTour {
  id: string
  title: string
  steps: any[]
}

// Reactive state
const learningModeActive = ref(false)
const currentPath = ref<LearningPath | null>(null)
const currentModule = ref<any>(null)
const currentConcept = ref<Concept | null>(null)
const currentCode = ref<string>('')
const activeTour = ref<CodeTour | null>(null)
const tourStep = ref(0)
const currentTutorial = ref<any>(null)
const aiTutorVisible = ref(false)
const showSettings = ref(false)
const showProgressModal = ref(false)

// Mock data - in real implementation, these would come from API
const availablePaths = ref<LearningPath[]>([
  {
    id: 'python-fundamentals',
    title: 'Python Fundamentals',
    description: 'Learn Python programming from basics to advanced concepts',
    modules: ['variables', 'functions', 'oop', 'libraries'],
    estimated_total_duration: 7200, // 2 hours
    target_audience: 'Beginners',
    outcomes: ['Write Python programs', 'Understand OOP', 'Use libraries']
  },
  {
    id: 'rust-systems',
    title: 'Rust Systems Programming',
    description: 'Master systems programming with Rust',
    modules: ['ownership', 'borrowing', 'lifetimes', 'async'],
    estimated_total_duration: 10800, // 3 hours
    target_audience: 'Intermediate',
    outcomes: ['Memory-safe code', 'Systems programming', 'Performance optimization']
  },
  {
    id: 'web-development',
    title: 'Web Development Full Stack',
    description: 'Build complete web applications',
    modules: ['html-css', 'javascript', 'react', 'nodejs'],
    estimated_total_duration: 14400, // 4 hours
    target_audience: 'Beginner to Intermediate',
    outcomes: ['Frontend development', 'Backend APIs', 'Full stack projects']
  }
])

const studentProfile = ref<StudentProfile>({
  id: 'student-1',
  name: 'Demo Student',
  learning_style: 'visual',
  current_level: 'beginner',
  progress: {},
  preferences: {
    difficulty_preference: 0.5,
    hint_frequency: 'after_struggle',
    code_completion_level: 'smart',
    visual_aids_enabled: true,
    voice_enabled: false
  },
  achievements: []
})

const recentAchievements = ref([
  {
    id: 'first-function',
    title: 'First Function',
    description: 'Created your first function',
    icon: '🎯'
  },
  {
    id: 'debug-master',
    title: 'Debug Master',
    description: 'Fixed 10 bugs on your own',
    icon: '🐛'
  }
])

const currentTutor = ref({
  id: 'main-tutor',
  personality: 'encouraging',
  specializations: ['general_programming']
})

const tutorContext = ref({
  current_file: null,
  current_concept: null,
  error_context: null,
  student_level: 'beginner',
  code_snippet: null
})

// Computed properties
const completedConcepts = computed(() => {
  if (!currentPath.value) return 0
  // Count mastered concepts in current path
  return Object.values(studentProfile.value.progress)
    .filter((progress: any) => progress.mastery_level >= 0.8).length
})

const totalConcepts = computed(() => {
  return currentPath.value?.modules.length * 5 || 0 // Estimate 5 concepts per module
})

// Methods
const toggleLearningMode = () => {
  learningModeActive.value = !learningModeActive.value
  // Emit event to parent component
  emit('learning-mode-toggled', learningModeActive.value)
}

const selectLearningPath = (path: LearningPath) => {
  currentPath.value = path
  loadModule(path.modules[0])
  // Reset current concept
  currentConcept.value = null
  activeTour.value = null
  currentTutorial.value = null
}

const loadModule = async (moduleId: string) => {
  // In real implementation, load from API
  currentModule.value = {
    id: moduleId,
    title: 'Sample Module',
    concepts: [
      {
        id: 'concept-1',
        name: 'Variables and Data Types',
        explanation: 'Learn about variables, data types, and how to store information in your programs.',
        code_examples: [],
        visual_aids: [],
        interactive_demos: []
      },
      {
        id: 'concept-2',
        name: 'Functions',
        explanation: 'Understand how to create reusable code blocks with functions.',
        code_examples: [],
        visual_aids: [],
        interactive_demos: []
      }
    ]
  }
}

const selectConcept = (concept: Concept) => {
  currentConcept.value = concept
  // Update tutor context
  tutorContext.value.current_concept = concept.id
}

const getConceptStatus = (conceptId: string) => {
  const progress = studentProfile.value.progress[conceptId]
  if (!progress) return 'not-started'
  if (progress.mastery_level >= 0.8) return 'mastered'
  if (progress.mastery_level > 0) return 'in-progress'
  return 'not-started'
}

const getPathProgress = (pathId: string) => {
  // Calculate progress for specific path
  return Math.floor(Math.random() * 100) // Mock progress
}

const formatDuration = (seconds: number) => {
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  if (hours > 0) {
    return `${hours}h ${minutes}m`
  }
  return `${minutes}m`
}

const startQuickStart = () => {
  // Start with the first available path
  if (availablePaths.value.length > 0) {
    selectLearningPath(availablePaths.value[0])
  }
}

const toggleAITutor = () => {
  aiTutorVisible.value = !aiTutorVisible.value
}

const startCodeTour = async () => {
  if (!currentCode.value) return
  
  // In real implementation, call API to generate tour
  activeTour.value = {
    id: 'tour-1',
    title: 'Understanding This Code',
    steps: [
      {
        step_number: 1,
        title: 'File Overview',
        explanation: 'This is the main entry point of the program.',
        line_range: [1, 10]
      }
    ]
  }
  tourStep.value = 0
}

const onConceptCompleted = (conceptId: string) => {
  // Update student progress
  if (!studentProfile.value.progress[conceptId]) {
    studentProfile.value.progress[conceptId] = {
      mastery_level: 1.0,
      completed_at: new Date().toISOString()
    }
  }
  
  // Auto-advance to next concept
  onNextConcept()
}

const onRequestHelp = (context: any) => {
  // Show AI tutor with specific context
  aiTutorVisible.value = true
  tutorContext.value = { ...tutorContext.value, ...context }
}

const onNextConcept = () => {
  if (!currentModule.value || !currentConcept.value) return
  
  const concepts = currentModule.value.concepts
  const currentIndex = concepts.findIndex(c => c.id === currentConcept.value?.id)
  
  if (currentIndex < concepts.length - 1) {
    selectConcept(concepts[currentIndex + 1])
  }
}

const onTourCompleted = () => {
  activeTour.value = null
  tourStep.value = 0
}

const onNextTourStep = () => {
  if (activeTour.value && tourStep.value < activeTour.value.steps.length - 1) {
    tourStep.value++
  } else {
    onTourCompleted()
  }
}

const onTutorialCompleted = () => {
  currentTutorial.value = null
}

const onExerciseCompleted = (exerciseId: string) => {
  // Handle exercise completion
  console.log('Exercise completed:', exerciseId)
}

const onSettingsUpdated = (newSettings: any) => {
  studentProfile.value.preferences = { ...studentProfile.value.preferences, ...newSettings }
  showSettings.value = false
}

// Watch for code changes to potentially start tours
watch(currentCode, (newCode) => {
  if (learningModeActive.value && newCode && newCode.length > 50) {
    // Suggest code tour for substantial code files
    // This could be automatic or user-initiated
  }
})

// Lifecycle
onMounted(() => {
  // Initialize learning engine
  // Load student profile from backend
  // Set up real-time updates
})

// Emits
const emit = defineEmits<{
  'learning-mode-toggled': [active: boolean]
  'concept-selected': [concept: Concept]
  'help-requested': [context: any]
}>()
</script>

<style scoped>
.learning-panel {
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
}

.learning-sidebar {
  scrollbar-width: thin;
  scrollbar-color: #4b5563 #1f2937;
}

.learning-sidebar::-webkit-scrollbar {
  width: 6px;
}

.learning-sidebar::-webkit-scrollbar-track {
  background: #1f2937;
}

.learning-sidebar::-webkit-scrollbar-thumb {
  background: #4b5563;
  border-radius: 3px;
}

.learning-sidebar::-webkit-scrollbar-thumb:hover {
  background: #6b7280;
}

/* Smooth transitions for concept selection */
.concept-item {
  transition: all 0.2s ease-in-out;
}

.concept-item:hover {
  transform: translateY(-1px);
}

/* Progress bar animations */
.progress-bar {
  transition: width 0.5s ease-out;
}

/* Achievement badge animations */
.achievement-badge {
  animation: bounceIn 0.5s ease-out;
}

@keyframes bounceIn {
  0% {
    transform: scale(0);
    opacity: 0;
  }
  50% {
    transform: scale(1.1);
  }
  100% {
    transform: scale(1);
    opacity: 1;
  }
}

/* Floating action button styles */
.fab {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.fab:hover {
  transform: translateY(-2px);
  box-shadow: 0 10px 25px rgba(0, 0, 0, 0.2);
}

/* Concept status indicators */
.concept-status-mastered {
  background: linear-gradient(135deg, #065f46, #047857);
}

.concept-status-in-progress {
  background: linear-gradient(135deg, #92400e, #b45309);
}

.concept-status-not-started {
  background: #374151;
}
</style>