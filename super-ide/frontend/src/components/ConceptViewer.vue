<template>
  <div class="concept-viewer p-6">
    <div v-if="concept" class="max-w-4xl mx-auto">
      <!-- Concept Header -->
      <div class="concept-header mb-6">
        <div class="flex items-center justify-between mb-4">
          <h1 class="text-2xl font-bold text-white">{{ concept.name }}</h1>
          <div class="flex items-center space-x-3">
            <span 
              :class="[
                'px-3 py-1 rounded-full text-sm font-medium',
                masteryLevel >= 0.8 ? 'bg-green-900 text-green-200' :
                masteryLevel > 0 ? 'bg-yellow-900 text-yellow-200' :
                'bg-gray-700 text-gray-300'
              ]"
            >
              {{ getStatusText() }}
            </span>
            <button
              @click="$emit('request-help', { concept_id: concept.id })"
              class="p-2 bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors"
              title="Ask for help"
            >
              <HelpCircle class="w-4 h-4" />
            </button>
          </div>
        </div>
        
        <p class="text-gray-300 text-lg leading-relaxed">{{ concept.explanation }}</p>
      </div>

      <!-- Code Examples -->
      <div v-if="concept.code_examples?.length" class="code-examples mb-8">
        <h2 class="text-xl font-semibold text-white mb-4 flex items-center">
          <Code class="w-5 h-5 mr-2" />
          Code Examples
        </h2>
        <div class="space-y-4">
          <div
            v-for="example in concept.code_examples"
            :key="example.title"
            class="bg-gray-800 rounded-lg overflow-hidden"
          >
            <div class="px-4 py-2 bg-gray-700 border-b border-gray-600">
              <h3 class="font-medium text-white">{{ example.title }}</h3>
              <p class="text-sm text-gray-400">{{ example.explanation }}</p>
            </div>
            <div class="p-4">
              <pre class="bg-gray-900 rounded p-3 text-sm overflow-x-auto"><code>{{ example.code }}</code></pre>
              <div v-if="example.key_points?.length" class="mt-3">
                <h4 class="text-sm font-medium text-gray-300 mb-2">Key Points:</h4>
                <ul class="list-disc list-inside text-sm text-gray-400 space-y-1">
                  <li v-for="point in example.key_points" :key="point">{{ point }}</li>
                </ul>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Interactive Demo -->
      <div vactive_demos?.-if="concept.interlength" class="interactive-demo mb-8">
        <h2 class="text-xl font-semibold text-white mb-4 flex items-center">
          <Play class="w-5 h-5 mr-2" />
          Interactive Demo
        </h2>
        <div class="bg-gray-800 rounded-lg p-6">
          <div class="text-center text-gray-400">
            <PlayCircle class="w-12 h-12 mx-auto mb-3 text-gray-500" />
            <p>Interactive demo would be embedded here</p>
            <p class="text-sm">This would show a live coding environment or visualization</p>
          </div>
        </div>
      </div>

      <!-- Visual Aids -->
      <div v-if="concept.visual_aids?.length" class="visual-aids mb-8">
        <h2 class="text-xl font-semibold text-white mb-4 flex items-center">
          <Image class="w-5 h-5 mr-2" />
          Visual Aids
        </h2>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div
            v-for="aidids"
            : in concept.visual_akey="aid.id"
            class="bg-gray-800 rounded-lg p-4"
          >
            <h3 class="font-medium text-white mb-2">{{ aid.title }}</h3>
            <p class="text-sm text-gray-400 mb-3">{{ aid.description }}</p>
            <div class="bg-gray-900 rounded p-3 text-center text-gray-500">
              {{ aid.aid_type }} visualization would be shown here
            </div>
          </div>
        </div>
      </div>

      <!-- Practice Section -->
      <div class="practice-section">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-xl font-semibold text-white flex items-center">
            <Target class="w-5 h-5 mr-2" />
            Practice
          </h2>
          <div class="text-sm text-gray-400">
            Mastery: {{ Math.round(masteryLevel * 100) }}%
          </div>
        </div>

        <!-- Progress Bar -->
        <div class="w-full bg-gray-700 rounded-full h-2 mb-6">
          <div 
            class="bg-gradient-to-r from-blue-500 to-green-500 h-2 rounded-full transition-all duration-500"
            :style="{ width: `${masteryLevel * 100}%` }"
          ></div>
        </div>

        <!-- Action Buttons -->
        <div class="flex space-x-4">
          <button
            v-if="masteryLevel < 0.8"
            @click="startPractice"
            class="px-6 py-3 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-lg transition-colors"
          >
            Start Practice
          </button>
          <button
            v-if="masteryLevel >= 0.8"
            @click="$emit('next-concept')"
            class="px-6 py-3 bg-green-600 hover:bg-green-700 text-white font-medium rounded-lg transition-colors"
          >
            Next Concept
          </button>
          <button
            @click="requestHint"
            class="px-6 py-3 bg-yellow-600 hover:bg-yellow-700 text-white font-medium rounded-lg transition-colors"
          >
            Get Hint
          </button>
        </div>
      </div>
    </div>

    <!-- No Concept Selected -->
    <div v-else class="flex items-center justify-center h-full text-gray-400">
      <div class="text-center">
        <BookOpen class="w-16 h-16 mx-auto mb-4 text-gray-500" />
        <h3 class="text-lg font-medium mb-2">Select a Concept</h3>
        <p class="text-sm">Choose a concept from the sidebar to start learning</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { 
  HelpCircle, 
  Code, 
  Play, 
  PlayCircle, 
  Image, 
  Target, 
  BookOpen 
} from 'lucide-vue-next'

// Props
interface Props {
  concept: {
    id: string
    name: string
    explanation: string
    code_examples?: any[]
    visual_aids?: any[]
    interactive_demos?: any[]
  } | null
  codeContext?: string
  studentProgress?: any
}

const props = defineProps<Props>()

// Emits
const emit = defineEmits<{
  'concept-completed': [conceptId: string]
  'request-help': [context: any]
  'next-concept': []
}>()

// Reactive state
const practiceStarted = ref(false)

// Computed properties
const masteryLevel = computed(() => {
  if (!props.studentProgress || !props.concept) return 0
  return props.studentProgress[props.concept.id]?.mastery_level || 0
})

// Methods
const getStatusText = () => {
  if (masteryLevel.value >= 0.8) return 'Mastered'
  if (masteryLevel.value > 0) return 'In Progress'
  return 'Not Started'
}

const startPractice = () => {
  practiceStarted.value = true
  // In real implementation, this would open a practice interface
  console.log('Starting practice for concept:', props.concept?.id)
}

const requestHint = () => {
  emit('request-help', { 
    concept_id: props.concept?.id,
    request_type: 'hint'
  })
}

// Watch for concept changes
watch(() => props.concept, (newConcept) => {
  if (newConcept) {
    practiceStarted.value = false
  }
})
</script>

<style scoped>
.concept-viewer {
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  max-height: calc(100vh - 120px);
  overflow-y: auto;
}

.concept-viewer::-webkit-scrollbar {
  width: 6px;
}

.concept-viewer::-webkit-scrollbar-track {
  background: #1f2937;
}

.concept-viewer::-webkit-scrollbar-thumb {
  background: #4b5563;
  border-radius: 3px;
}

.concept-viewer::-webkit-scrollbar-thumb:hover {
  background: #6b7280;
}

/* Code block styling */
pre {
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
}

/* Practice section animations */
.practice-section {
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

/* Button hover effects */
button:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

/* Concept status indicator animations */
.status-mastered {
  animation: successPulse 2s ease-in-out infinite;
}

@keyframes successPulse {
  0%, 100% {
    box-shadow: 0 0 0 0 rgba(34, 197, 94, 0.7);
  }
  50% {
    box-shadow: 0 0 0 10px rgba(34, 197, 94, 0);
  }
}
</style>