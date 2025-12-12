<template>
  <div class="learning-settings fixed inset-0 bg-black bg-opacity-50 z-50 flex items-center justify-center">
    <div class="bg-gray-900 border border-gray-700 rounded-lg shadow-2xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-hidden">
      <!-- Header -->
      <div class="settings-header p-6 border-b border-gray-700">
        <div class="flex items-center justify-between">
          <h2 class="text-xl font-bold text-white flex items-center">
            <Settings class="w-6 h-6 mr-2 text-blue-400" />
            Learning Settings
          </h2>
          <button
            @click="$emit('close')"
            class="p-2 hover:bg-gray-700 rounded text-gray-400 hover:text-white transition-colors"
          >
            <X class="w-5 h-5" />
          </button>
        </div>
      </div>

      <!-- Settings Content -->
      <div class="settings-content p-6 overflow-y-auto max-h-[calc(90vh-120px)]">
        <!-- Student Profile Section -->
        <div class="settings-section mb-8">
          <h3 class="text-lg font-semibold text-white mb-4">Student Profile</h3>
          
          <div class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-gray-300 mb-2">Name</label>
              <input
                v-model="settings.name"
                type="text"
                class="w-full px-3 py-2 bg-gray-800 border border-gray-600 rounded-lg text-white focus:border-blue-500 focus:outline-none"
                placeholder="Enter your name"
              />
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-300 mb-2">Learning Style</label>
              <div class="grid grid-cols-2 gap-3">
                <button
                  v-for="style in learningStyles"
                  :key="style.value"
                  @click="settings.learning_style = style.value"
                  :class="[
                    'p-3 rounded-lg border text-left transition-all',
                    settings.learning_style === style.value
                      ? 'bg-blue-900 border-blue-600 text-blue-200'
                      : 'bg-gray-800 border-gray-700 text-gray-300 hover:bg-gray-750'
                  ]"
                >
                  <div class="font-medium">{{ style.label }}</div>
                  <div class="text-xs text-gray-400 mt-1">{{ style.description }}</div>
                </button>
              </div>
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-300 mb-2">Current Skill Level</label>
              <select
                v-model="settings.current_level"
                class="w-full px-3 py-2 bg-gray-800 border border-gray-600 rounded-lg text-white focus:border-blue-500 focus:outline-none"
              >
                <option value="beginner">Beginner</option>
                <option value="intermediate">Intermediate</option>
                <option value="advanced">Advanced</option>
                <option value="expert">Expert</option>
              </select>
            </div>
          </div>
        </div>

        <!-- Learning Preferences Section -->
        <div class="settings-section mb-8">
          <h3 class="text-lg font-semibold text-white mb-4">Learning Preferences</h3>
          
          <div class="space-y-6">
            <!-- Difficulty Preference -->
            <div>
              <label class="block text-sm font-medium text-gray-300 mb-3">
                Preferred Difficulty Level
                <span class="text-blue-400 ml-2">({{ Math.round(settings.difficulty_preference * 100) }}%)</span>
              </label>
              <div class="px-3">
                <input
                  v-model="settings.difficulty_preference"
                  type="range"
                  min="0"
                  max="1"
                  step="0.1"
                  class="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer slider"
                />
                <div class="flex justify-between text-xs text-gray-400 mt-1">
                  <span>Very Easy</span>
                  <span>Balanced</span>
                  <span>Very Hard</span>
                </div>
              </div>
            </div>

            <!-- Hint Frequency -->
            <div>
              <label class="block text-sm font-medium text-gray-300 mb-2">Hint Frequency</label>
              <div class="space-y-2">
                <label
                  v-for="frequency in hintFrequencies"
                  :key="frequency.value"
                  class="flex items-center space-x-3 p-3 bg-gray-800 rounded-lg cursor-pointer hover:bg-gray-750 transition-colors"
                >
                  <input
                    v-model="settings.hint_frequency"
                    type="radio"
                    :value="frequency.value"
                    class="text-blue-600 focus:ring-blue-500"
                  />
                  <div>
                    <div class="text-white font-medium">{{ frequency.label }}</div>
                    <div class="text-sm text-gray-400">{{ frequency.description }}</div>
                  </div>
                </label>
              </div>
            </div>

            <!-- Code Completion Level -->
            <div>
              <label class="block text-sm font-medium text-gray-300 mb-2">Code Completion Assistance</label>
              <div class="space-y-2">
                <label
                  v-for="level in completionLevels"
                  :key="level.value"
                  class="flex items-center space-x-3 p-3 bg-gray-800 rounded-lg cursor-pointer hover:bg-gray-750 transition-colors"
                >
                  <input
                    v-model="settings.code_completion_level"
                    type="radio"
                    :value="level.value"
                    class="text-blue-600 focus:ring-blue-500"
                  />
                  <div>
                    <div class="text-white font-medium">{{ level.label }}</div>
                    <div class="text-sm text-gray-400">{{ level.description }}</div>
                  </div>
                </label>
              </div>
            </div>

            <!-- Visual & Audio Options -->
            <div>
              <label class="block text-sm font-medium text-gray-300 mb-3">Interface Options</label>
              <div class="space-y-3">
                <label class="flex items-center justify-between p-3 bg-gray-800 rounded-lg">
                  <div>
                    <div class="text-white font-medium">Visual Aids</div>
                    <div class="text-sm text-gray-400">Show diagrams and visual explanations</div>
                  </div>
                  <input
                    v-model="settings.visual_aids_enabled"
                    type="checkbox"
                    class="w-5 h-5 text-blue-600 bg-gray-700 border-gray-600 rounded focus:ring-blue-500"
                  />
                </label>

                <label class="flex items-center justify-between p-3 bg-gray-800 rounded-lg">
                  <div>
                    <div class="text-white font-medium">Voice Assistance</div>
                    <div class="text-sm text-gray-400">Enable voice explanations and hints</div>
                  </div>
                  <input
                    v-model="settings.voice_enabled"
                    type="checkbox"
                    class="w-5 h-5 text-blue-600 bg-gray-700 border-gray-600 rounded focus:ring-blue-500"
                  />
                </label>
              </div>
            </div>
          </div>
        </div>

        <!-- Achievement Settings -->
        <div class="settings-section mb-8">
          <h3 class="text-lg font-semibold text-white mb-4">Achievements & Gamification</h3>
          
          <div class="space-y-3">
            <label class="flex items-center justify-between p-3 bg-gray-800 rounded-lg">
              <div>
                <div class="text-white font-medium">Show Achievement Notifications</div>
                <div class="text-sm text-gray-400">Get notified when you earn achievements</div>
              </div>
              <input
                v-model="settings.achievement_notifications"
                type="checkbox"
                class="w-5 h-5 text-blue-600 bg-gray-700 border-gray-600 rounded focus:ring-blue-500"
              />
            </label>

            <label class="flex items-center justify-between p-3 bg-gray-800 rounded-lg">
              <div>
                <div class="text-white font-medium">Learning Streaks</div>
                <div class="text-sm text-gray-400">Track consecutive days of learning</div>
              </div>
              <input
                v-model="settings.learning_streaks"
                type="checkbox"
                class="w-5 h-5 text-blue-600 bg-gray-700 border-gray-600 rounded focus:ring-blue-500"
              />
            </label>
          </div>
        </div>

        <!-- Privacy & Data -->
        <div class="settings-section">
          <h3 class="text-lg font-semibold text-white mb-4">Privacy & Data</h3>
          
          <div class="space-y-3">
            <label class="flex items-center justify-between p-3 bg-gray-800 rounded-lg">
              <div>
                <div class="text-white font-medium">Learning Analytics</div>
                <div class="text-sm text-gray-400">Allow collection of learning data for personalization</div>
              </div>
              <input
                v-model="settings.analytics_enabled"
                type="checkbox"
                class="w-5 h-5 text-blue-600 bg-gray-700 border-gray-600 rounded focus:ring-blue-500"
              />
            </label>

            <label class="flex items-center justify-between p-3 bg-gray-800 rounded-lg">
              <div>
                <div class="text-white font-medium">Share Progress</div>
                <div class="text-sm text-gray-400">Share achievements with study groups</div>
              </div>
              <input
                v-model="settings.share_progress"
                type="checkbox"
                class="w-5 h-5 text-blue-600 bg-gray-700 border-gray-600 rounded focus:ring-blue-500"
              />
            </label>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="settings-footer p-6 border-t border-gray-700 bg-gray-800">
        <div class="flex justify-between">
          <button
            @click="resetToDefaults"
            class="px-4 py-2 bg-gray-600 hover:bg-gray-700 text-white rounded-lg transition-colors"
          >
            Reset to Defaults
          </button>
          
          <div class="flex space-x-3">
            <button
              @click="$emit('close')"
              class="px-6 py-2 bg-gray-600 hover:bg-gray-700 text-white rounded-lg transition-colors"
            >
              Cancel
            </button>
            <button
              @click="saveSettings"
              class="px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors"
            >
              Save Settings
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Settings, X } from 'lucide-vue-next'

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
}

const props = defineProps<Props>()

// Emits
const emit = defineEmits<{
  close: []
  'settings-updated': [settings: any]
}>()

// Reactive state
const settings = ref({
  name: '',
  learning_style: 'visual',
  current_level: 'beginner',
  difficulty_preference: 0.5,
  hint_frequency: 'after_struggle',
  code_completion_level: 'smart',
  visual_aids_enabled: true,
  voice_enabled: false,
  achievement_notifications: true,
  learning_streaks: true,
  analytics_enabled: true,
  share_progress: false
})

// Options
const learningStyles = [
  { value: 'visual', label: 'Visual', description: 'Learn best with diagrams and visual aids' },
  { value: 'auditory', label: 'Auditory', description: 'Prefer explanations and discussions' },
  { value: 'kinesthetic', label: 'Hands-on', description: 'Learn by doing and practicing' },
  { value: 'reading', label: 'Reading', description: 'Prefer written materials and documentation' }
]

const hintFrequencies = [
  { 
    value: 'never', 
    label: 'Never', 
    description: 'Figure it out on your own' 
  },
  { 
    value: 'on_request', 
    label: 'On Request', 
    description: 'Only when you ask for help' 
  },
  { 
    value: 'after_struggle', 
    label: 'After Struggle', 
    description: 'After you\'ve tried for a while' 
  },
  { 
    value: 'always', 
    label: 'Always', 
    description: 'Proactive hints and suggestions' 
  }
]

const completionLevels = [
  { 
    value: 'none', 
    label: 'None', 
    description: 'No code completion assistance' 
  },
  { 
    value: 'basic', 
    label: 'Basic', 
    description: 'Basic syntax completion' 
  },
  { 
    value: 'smart', 
    label: 'Smart', 
    description: 'Context-aware suggestions' 
  },
  { 
    value: 'full_guided', 
    label: 'Fully Guided', 
    description: 'Complete guided coding experience' 
  }
]

// Methods
const loadSettings = () => {
  if (props.studentProfile) {
    settings.value = {
      name: props.studentProfile.name,
      learning_style: props.studentProfile.learning_style,
      current_level: props.studentProfile.current_level,
      difficulty_preference: props.studentProfile.preferences?.difficulty_preference || 0.5,
      hint_frequency: props.studentProfile.preferences?.hint_frequency || 'after_struggle',
      code_completion_level: props.studentProfile.preferences?.code_completion_level || 'smart',
      visual_aids_enabled: props.studentProfile.preferences?.visual_aids_enabled ?? true,
      voice_enabled: props.studentProfile.preferences?.voice_enabled ?? false,
      achievement_notifications: true,
      learning_streaks: true,
      analytics_enabled: true,
      share_progress: false
    }
  }
}

const saveSettings = () => {
  emit('settings-updated', settings.value)
}

const resetToDefaults = () => {
  settings.value = {
    name: 'Student',
    learning_style: 'visual',
    current_level: 'beginner',
    difficulty_preference: 0.5,
    hint_frequency: 'after_struggle',
    code_completion_level: 'smart',
    visual_aids_enabled: true,
    voice_enabled: false,
    achievement_notifications: true,
    learning_streaks: true,
    analytics_enabled: true,
    share_progress: false
  }
}

// Lifecycle
onMounted(() => {
  loadSettings()
})
</script>

<style scoped>
.learning-settings {
  backdrop-filter: blur(8px);
}

.settings-content {
  scrollbar-width: thin;
  scrollbar-color: #4b5563 #1f2937;
}

.settings-content::-webkit-scrollbar {
  width: 6px;
}

.settings-content::-webkit-scrollbar-track {
  background: #1f2937;
}

.settings-content::-webkit-scrollbar-thumb {
  background: #4b5563;
  border-radius: 3px;
}

.settings-content::-webkit-scrollbar-thumb:hover {
  background: #6b7280;
}

/* Range slider styling */
.slider::-webkit-slider-thumb {
  appearance: none;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: #3b82f6;
  cursor: pointer;
  box-shadow: 0 0 0 2px #1f2937;
}

.slider::-moz-range-thumb {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: #3b82f6;
  cursor: pointer;
  border: none;
  box-shadow: 0 0 0 2px #1f2937;
}

/* Radio button styling */
input[type="radio"] {
  accent-color: #3b82f6;
}

/* Checkbox styling */
input[type="checkbox"] {
  accent-color: #3b82f6;
}

/* Setting section animations */
.settings-section {
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
button:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

/* Toggle switch animations */
input[type="checkbox"]:checked {
  animation: checkPulse 0.3s ease-out;
}

@keyframes checkPulse {
  0% {
    transform: scale(1);
  }
  50% {
    transform: scale(1.1);
  }
  100% {
    transform: scale(1);
  }
}
</style>