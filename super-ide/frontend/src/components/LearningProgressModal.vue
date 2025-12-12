<template>
  <div class="learning-progress-modal fixed inset-0 bg-black bg-opacity-50 z-50 flex items-center justify-center">
    <div class="bg-gray-900 border border-gray-700 rounded-lg shadow-2xl max-w-4xl w-full mx-4 max-h-[90vh] overflow-hidden">
      <!-- Header -->
      <div class="progress-header p-6 border-b border-gray-700">
        <div class="flex items-center justify-between">
          <h2 class="text-2xl font-bold text-white flex items-center">
            <BarChart3 class="w-6 h-6 mr-3 text-blue-400" />
            Learning Progress
          </h2>
          <button
            @click="$emit('close')"
            class="p-2 hover:bg-gray-700 rounded text-gray-400 hover:text-white transition-colors"
          >
            <X class="w-6 h-6" />
          </button>
        </div>
      </div>

      <!-- Progress Content -->
      <div class="progress-content p-6 overflow-y-auto max-h-[calc(90vh-120px)]">
        <!-- Student Overview -->
        <div class="student-overview mb-8">
          <div class="bg-gradient-to-r from-blue-900 to-purple-900 rounded-lg p-6">
            <div class="flex items-center justify-between">
              <div>
                <h3 class="text-xl font-bold text-white">{{ studentProfile.name }}</h3>
                <p class="text-blue-200">{{ formatSkillLevel(studentProfile.current_level) }}</p>
              </div>
              <div class="text-right">
                <div class="text-3xl font-bold text-white">{{ totalProgress }}%</div>
                <p class="text-blue-200 text-sm">Overall Progress</p>
              </div>
            </div>
          </div>
        </div>

        <!-- Learning Paths Progress -->
        <div class="paths-progress mb-8">
          <h3 class="text-xl font-semibold text-white mb-4">Learning Paths</h3>
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div
              v-for="path in learningPaths"
              :key="path.id"
              class="bg-gray-800 rounded-lg p-4 border border-gray-700"
            >
              <div class="flex items-center justify-between mb-3">
                <h4 class="font-semibold text-white">{{ path.title }}</h4>
                <span class="text-sm text-gray-400">{{ getPathProgress(path.id) }}%</span>
              </div>
              
              <!-- Progress Bar -->
              <div class="w-full bg-gray-700 rounded-full h-2 mb-3">
                <div 
                  class="bg-gradient-to-r from-blue-500 to-green-500 h-2 rounded-full transition-all duration-500"
                  :style="{ width: `${getPathProgress(path.id)}%` }"
                ></div>
              </div>
              
              <p class="text-sm text-gray-400 mb-2">{{ path.description }}</p>
              
              <div class="flex items-center justify-between text-xs text-gray-500">
                <span>{{ path.modules.length }} modules</span>
                <span>{{ formatDuration(path.estimated_total_duration) }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Concept Mastery Heatmap -->
        <div class="concept-mastery mb-8">
          <h3 class="text-xl font-semibold text-white mb-4">Concept Mastery</h3>
          <div class="bg-gray-800 rounded-lg p-4">
            <div class="grid grid-cols-5 md:grid-cols-8 gap-2">
              <div
                v-for="(concept, index) in masteredConcepts"
                :key="concept.id"
                :class="[
                  'aspect-square rounded-lg flex items-center justify-center text-xs font-medium transition-all cursor-pointer',
                  concept.mastery_level >= 0.8 ? 'bg-green-600 hover:bg-green-700 text-white' :
                  concept.mastery_level >= 0.5 ? 'bg-yellow-600 hover:bg-yellow-700 text-white' :
                  'bg-gray-600 hover:bg-gray-700 text-gray-300'
                ]"
                :title="`${concept.name}: ${Math.round(concept.mastery_level * 100)}%`"
              >
                {{ index + 1 }}
              </div>
            </div>
            <div class="flex items-center justify-between mt-4 text-sm">
              <div class="flex items-center space-x-4">
                <div class="flex items-center space-x-2">
                  <div class="w-3 h-3 bg-green-600 rounded"></div>
                  <span class="text-gray-400">Mastered (80%+)</span>
                </div>
                <div class="flex items-center space-x-2">
                  <div class="w-3 h-3 bg-yellow-600 rounded"></div>
                  <span class="text-gray-400">Learning (50-79%)</span>
                </div>
                <div class="flex items-center space-x-2">
                  <div class="w-3 h-3 bg-gray-600 rounded"></div>
                  <span class="text-gray-400">Not Started</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Learning Analytics -->
        <div class="learning-analytics mb-8">
          <h3 class="text-xl font-semibold text-white mb-4">Learning Analytics</h3>
          <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
            <!-- Time Spent -->
            <div class="bg-gray-800 rounded-lg p-4">
              <div class="flex items-center justify-between mb-2">
                <h4 class="font-medium text-white">Time Spent</h4>
                <Clock class="w-5 h-5 text-blue-400" />
              </div>
              <div class="text-2xl font-bold text-blue-400">{{ totalTimeSpent }}</div>
              <p class="text-sm text-gray-400">Total learning time</p>
            </div>

            <!-- Concepts Learned -->
            <div class="bg-gray-800 rounded-lg p-4">
              <div class="flex items-center justify-between mb-2">
                <h4 class="font-medium text-white">Concepts</h4>
                <BookOpen class="w-5 h-5 text-green-400" />
              </div>
              <div class="text-2xl font-bold text-green-400">{{ conceptsLearned }}</div>
              <p class="text-sm text-gray-400">Concepts mastered</p>
            </div>

            <!-- Streak -->
            <div class="bg-gray-800 rounded-lg p-4">
              <div class="flex items-center justify-between mb-2">
                <h4 class="font-medium text-white">Current Streak</h4>
                <Zap class="w-5 h-5 text-yellow-400" />
              </div>
              <div class="text-2xl font-bold text-yellow-400">{{ currentStreak }}</div>
              <p class="text-sm text-gray-400">Consecutive days</p>
            </div>
          </div>
        </div>

        <!-- Recent Achievements -->
        <div class="recent-achievements mb-8">
          <h3 class="text-xl font-semibold text-white mb-4">Recent Achievements</h3>
          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            <div
              v-for="achievement in recentAchievements"
              :key="achievement.id"
              class="bg-gray-800 rounded-lg p-4 border border-gray-700"
            >
              <div class="flex items-center space-x-3">
                <div class="text-3xl">{{ achievement.icon }}</div>
                <div>
                  <h4 class="font-medium text-white">{{ achievement.title }}</h4>
                  <p class="text-sm text-gray-400">{{ achievement.description }}</p>
                  <p class="text-xs text-gray-500 mt-1">{{ formatDate(achievement.earned_at) }}</p>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Learning Recommendations -->
        <div class="learning-recommendations">
          <h3 class="text-xl font-semibold text-white mb-4">Recommendations</h3>
          <div class="space-y-3">
            <div
              v-for="recommendation in recommendations"
              :key="recommendation.id"
              class="bg-blue-900 border border-blue-700 rounded-lg p-4"
            >
              <div class="flex items-start space-x-3">
                <Lightbulb class="w-5 h-5 text-blue-400 mt-0.5" />
                <div>
                  <h4 class="font-medium text-blue-200">{{ recommendation.title }}</h4>
                  <p class="text-sm text-blue-100 mt-1">{{ recommendation.description }}</p>
                  <button class="mt-2 text-sm text-blue-300 hover:text-blue-200 underline">
                    {{ recommendation.action }}
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="progress-footer p-6 border-t border-gray-700 bg-gray-800">
        <div class="flex justify-between items-center">
          <div class="text-sm text-gray-400">
            Last updated: {{ new Date().toLocaleDateString() }}
          </div>
          <button
            @click="$emit('close')"
            class="px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { 
  BarChart3, 
  X, 
  Clock, 
  BookOpen, 
  Zap, 
  Lightbulb 
} from 'lucide-vue-next'

// Props
interface Props {
  studentProfile: {
    id: string
    name: string
    learning_style: string
    current_level: string
    progress: Record<string, any>
    preferences: any
    achievements: any[]
  }
  learningPaths: Array<{
    id: string
    title: string
    description: string
    modules: string[]
    estimated_total_duration: number
    target_audience: string
    outcomes: string[]
  }>
}

const props = defineProps<Props>()

// Emits
const emit = defineEmits<{
  close: []
}>()

// Mock data for demonstration
const masteredConcepts = ref([
  { id: '1', name: 'Variables', mastery_level: 0.95 },
  { id: '2', name: 'Functions', mastery_level: 0.87 },
  { id: '3', name: 'Loops', mastery_level: 0.92 },
  { id: '4', name: 'Arrays', mastery_level: 0.76 },
  { id: '5', name: 'Objects', mastery_level: 0.65 },
  { id: '6', name: 'Classes', mastery_level: 0.45 },
  { id: '7', name: 'Inheritance', mastery_level: 0.23 },
  { id: '8', name: 'Async/Await', mastery_level: 0.15 }
])

const recentAchievements = ref([
  {
    id: 'first-function',
    title: 'First Function',
    description: 'Created your first function',
    icon: '🎯',
    earned_at: new Date(Date.now() - 86400000) // 1 day ago
  },
  {
    id: 'debug-master',
    title: 'Debug Master',
    description: 'Fixed 10 bugs on your own',
    icon: '🐛',
    earned_at: new Date(Date.now() - 172800000) // 2 days ago
  },
  {
    id: 'algorithm-solver',
    title: 'Algorithm Solver',
    description: 'Solved 5 algorithm challenges',
    icon: '🧩',
    earned_at: new Date(Date.now() - 259200000) // 3 days ago
  }
])

const recommendations = ref([
  {
    id: 'practice-functions',
    title: 'Practice More Functions',
    description: 'You\'re doing great with functions! Practice with more complex examples.',
    action: 'Start Function Practice'
  },
  {
    id: 'explore-classes',
    title: 'Explore Object-Oriented Programming',
    description: 'Ready to dive into classes and objects? They\'re fundamental to many programming paradigms.',
    action: 'Learn About Classes'
  },
  {
    id: 'async-programming',
    title: 'Master Asynchronous Programming',
    description: 'Async/await concepts are challenging but essential for modern programming.',
    action: 'Start Async Tutorial'
  }
])

// Computed properties
const totalProgress = computed(() => {
  const totalConcepts = masteredConcepts.value.length
  const masteredCount = masteredConcepts.value.filter(c => c.mastery_level >= 0.8).length
  return Math.round((masteredCount / totalConcepts) * 100)
})

const totalTimeSpent = computed(() => {
  // Mock calculation - in real implementation, this would come from actual data
  return '12h 45m'
})

const conceptsLearned = computed(() => {
  return masteredConcepts.value.filter(c => c.mastery_level >= 0.8).length
})

const currentStreak = computed(() => {
  // Mock streak calculation
  return 7
})

// Methods
const formatSkillLevel = (level: string) => {
  return level.charAt(0).toUpperCase() + level.slice(1)
}

const getPathProgress = (pathId: string) => {
  // Mock progress calculation
  const progressMap: Record<string, number> = {
    'python-fundamentals': 75,
    'rust-systems': 45,
    'web-development': 30
  }
  return progressMap[pathId] || 0
}

const formatDuration = (seconds: number) => {
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  if (hours > 0) {
    return `${hours}h ${minutes}m`
  }
  return `${minutes}m`
}

const formatDate = (date: Date) => {
  return date.toLocaleDateString('en-US', { 
    month: 'short', 
    day: 'numeric' 
  })
}
</script>

<style scoped>
.learning-progress-modal {
  backdrop-filter: blur(8px);
}

.progress-content {
  scrollbar-width: thin;
  scrollbar-color: #4b5563 #1f2937;
}

.progress-content::-webkit-scrollbar {
  width: 6px;
}

.progress-content::-webkit-scrollbar-track {
  background: #1f2937;
}

.progress-content::-webkit-scrollbar-thumb {
  background: #4b5563;
  border-radius: 3px;
}

.progress-content::-webkit-scrollbar-thumb:hover {
  background: #6b7280;
}

/* Progress bar animations */
.progress-bar {
  animation: progressGrow 1s ease-out;
}

@keyframes progressGrow {
  0% {
    width: 0%;
  }
}

/* Heatmap cell animations */
.aspect-square > div {
  transition: all 0.2s ease;
}

.aspect-square > div:hover {
  transform: scale(1.1);
  z-index: 10;
}

/* Achievement card animations */
.achievement-card {
  animation: slideInUp 0.3s ease-out;
}

@keyframes slideInUp {
  0% {
    transform: translateY(20px);
    opacity: 0;
  }
  100% {
    transform: translateY(0);
    opacity: 1;
  }
}

/* Recommendation cards */
.recommendation-card {
  animation: slideInLeft 0.3s ease-out;
}

@keyframes slideInLeft {
  0% {
    transform: translateX(-20px);
    opacity: 0;
  }
  100% {
    transform: translateX(0);
    opacity: 1;
  }
}

/* Stats card hover effects */
.stats-card {
  transition: all 0.2s ease;
}

.stats-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 25px rgba(0, 0, 0, 0.2);
}

/* Progress indicator animations */
.progress-indicator {
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.7;
  }
}
</style>