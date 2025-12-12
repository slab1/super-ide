<template>
  <div class="collaboration-panel h-full flex flex-col bg-gray-900 text-white">
    <!-- Header -->
    <div class="flex items-center justify-between p-4 border-b border-gray-700">
      <div class="flex items-center space-x-3">
        <div class="w-3 h-3 rounded-full" :class="connectionStatusClass"></div>
        <h2 class="text-lg font-semibold">Collaboration</h2>
        <span v-if="currentSession" class="text-sm text-gray-400">({{ currentSession.name }})</span>
      </div>
      
      <div class="flex items-center space-x-2">
        <button
          v-if="!currentSession"
          @click="showCreateSession = true"
          class="px-3 py-1 bg-blue-600 hover:bg-blue-700 rounded text-sm font-medium transition-colors"
        >
          Start Session
        </button>
        
        <button
          v-else
          @click="leaveSession"
          class="px-3 py-1 bg-red-600 hover:bg-red-700 rounded text-sm font-medium transition-colors"
        >
          Leave
        </button>
      </div>
    </div>

    <!-- Connection Status -->
    <div v-if="connectionError" class="p-3 bg-red-900/50 border-b border-red-700">
      <p class="text-sm text-red-200">{{ connectionError }}</p>
    </div>

    <!-- No Session State -->
    <div v-if="!currentSession" class="flex-1 flex flex-col items-center justify-center p-8">
      <div class="text-center">
        <svg class="w-16 h-16 mx-auto mb-4 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
        </svg>
        <h3 class="text-xl font-semibold mb-2">No Active Session</h3>
        <p class="text-gray-400 mb-6">Start collaborating with your team in real-time</p>
        
        <div class="space-y-4">
          <button
            @click="showCreateSession = true"
            class="w-full px-4 py-3 bg-blue-600 hover:bg-blue-700 rounded-lg font-medium transition-colors"
          >
            Create New Session
          </button>
          
          <button
            @click="showJoinSession = true"
            class="w-full px-4 py-3 bg-gray-700 hover:bg-gray-600 rounded-lg font-medium transition-colors"
          >
            Join Existing Session
          </button>
        </div>
      </div>
    </div>

    <!-- Active Session UI -->
    <div v-else class="flex-1 flex flex-col">
      <!-- Collaborators -->
      <div class="p-4 border-b border-gray-700">
        <div class="flex items-center justify-between mb-3">
          <h3 class="text-sm font-semibold text-gray-300">Collaborators ({{ onlineCollaborators.length }})</h3>
          <button
            @click="showInviteModal = true"
            class="text-xs text-blue-400 hover:text-blue-300 transition-colors"
          >
            Invite
          </button>
        </div>
        
        <div class="space-y-2">
          <div
            v-for="collaborator in onlineCollaborators"
            :key="collaborator.id"
            class="flex items-center space-x-3 p-2 rounded-lg bg-gray-800"
          >
            <div class="relative">
              <div
                class="w-8 h-8 rounded-full flex items-center justify-center text-white text-sm font-medium"
                :style="{ backgroundColor: collaborator.color }"
              >
                {{ getInitials(collaborator.name) }}
              </div>
              <div class="absolute -bottom-1 -right-1 w-3 h-3 rounded-full border-2 border-gray-900" :class="getStatusClass(collaborator.status)"></div>
            </div>
            
            <div class="flex-1 min-w-0">
              <p class="text-sm font-medium truncate">{{ collaborator.name }}</p>
              <p class="text-xs text-gray-400 truncate">{{ collaborator.email }}</p>
            </div>
            
            <div v-if="collaborator.cursor" class="text-xs text-gray-500">
              L{{ collaborator.cursor.lineNumber }}:{{ collaborator.cursor.column }}
            </div>
          </div>
        </div>
      </div>

      <!-- Tabs -->
      <div class="flex border-b border-gray-700">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          @click="activeTab = tab.id"
          class="flex-1 px-3 py-2 text-sm font-medium transition-colors"
          :class="activeTab === tab.id ? 'text-blue-400 border-b-2 border-blue-400' : 'text-gray-400 hover:text-gray-300'"
        >
          {{ tab.label }}
        </button>
      </div>

      <!-- Tab Content -->
      <div class="flex-1 overflow-auto">
        <!-- Real-time Editing Tab -->
        <div v-show="activeTab === 'editing'" class="p-4">
          <div class="mb-4">
            <h3 class="text-sm font-semibold mb-2">Session Info</h3>
            <div class="bg-gray-800 rounded-lg p-3 space-y-2">
              <div class="flex justify-between text-sm">
                <span class="text-gray-400">Session ID:</span>
                <code class="text-xs bg-gray-700 px-2 py-1 rounded">{{ currentSession.id }}</code>
              </div>
              <div class="flex justify-between text-sm">
                <span class="text-gray-400">Status:</span>
                <span class="capitalize">{{ currentSession.status }}</span>
              </div>
              <div class="flex justify-between text-sm">
                <span class="text-gray-400">Created:</span>
                <span>{{ formatDate(currentSession.createdAt) }}</span>
              </div>
            </div>
          </div>

          <div class="mb-4">
            <h3 class="text-sm font-semibold mb-2">Share Session</h3>
            <div class="flex space-x-2">
              <input
                :value="sessionShareUrl"
                readonly
                class="flex-1 px-3 py-2 bg-gray-800 border border-gray-700 rounded text-sm"
              />
              <button
                @click="copySessionUrl"
                class="px-3 py-2 bg-blue-600 hover:bg-blue-700 rounded text-sm font-medium transition-colors"
              >
                Copy
              </button>
            </div>
          </div>

          <div>
            <h3 class="text-sm font-semibold mb-2">Active Files</h3>
            <div class="space-y-1">
              <div
                v-for="file in currentSession.files"
                :key="file"
                class="flex items-center space-x-2 p-2 bg-gray-800 rounded text-sm"
              >
                <svg class="w-4 h-4 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                </svg>
                <span class="flex-1 truncate">{{ file }}</span>
              </div>
              
              <div v-if="currentSession.files.length === 0" class="text-gray-500 text-sm italic text-center py-4">
                No files shared yet
              </div>
            </div>
          </div>
        </div>

        <!-- Comments Tab -->
        <div v-show="activeTab === 'comments'" class="p-4">
          <div class="mb-4 flex items-center justify-between">
            <h3 class="text-sm font-semibold">Code Comments ({{ activeComments.length }})</h3>
            <button
              @click="showCommentModal = true"
              class="px-3 py-1 bg-blue-600 hover:bg-blue-700 rounded text-sm font-medium transition-colors"
            >
              Add Comment
            </button>
          </div>

          <div class="space-y-3">
            <div
              v-for="comment in activeComments"
              :key="comment.id"
              class="bg-gray-800 rounded-lg p-3"
            >
              <div class="flex items-start justify-between mb-2">
                <div class="flex items-center space-x-2">
                  <div
                    class="w-6 h-6 rounded-full flex items-center justify-center text-white text-xs font-medium"
                    :style="{ backgroundColor: comment.author.color }"
                  >
                    {{ getInitials(comment.author.name) }}
                  </div>
                  <span class="text-sm font-medium">{{ comment.author.name }}</span>
                  <span class="text-xs text-gray-400">{{ formatDate(comment.createdAt) }}</span>
                </div>
                
                <div class="flex items-center space-x-2">
                  <span class="text-xs px-2 py-1 rounded" :class="getPriorityClass(comment.priority)">
                    {{ comment.priority }}
                  </span>
                  <button
                    @click="resolveComment(comment.id)"
                    class="text-xs text-green-400 hover:text-green-300 transition-colors"
                  >
                    Resolve
                  </button>
                </div>
              </div>
              
              <div class="mb-2">
                <div class="text-xs text-gray-400 mb-1">
                  {{ comment.filePath }}:L{{ comment.lineNumber }}
                </div>
                <p class="text-sm">{{ comment.content }}</p>
              </div>
              
              <div v-if="comment.tags.length > 0" class="flex flex-wrap gap-1">
                <span
                  v-for="tag in comment.tags"
                  :key="tag"
                  class="text-xs bg-gray-700 px-2 py-1 rounded"
                >
                  {{ tag }}
                </span>
              </div>
            </div>
            
            <div v-if="activeComments.length === 0" class="text-gray-500 text-sm italic text-center py-8">
              No active comments
            </div>
          </div>
        </div>

        <!-- Sharing Tab -->
        <div v-show="activeTab === 'sharing'" class="p-4">
          <div class="mb-4">
            <h3 class="text-sm font-semibold mb-3">Shareable Links</h3>
            <button
              @click="showCreateLinkModal = true"
              class="w-full px-3 py-2 bg-blue-600 hover:bg-blue-700 rounded text-sm font-medium transition-colors"
            >
              Create New Link
            </button>
          </div>

          <div class="space-y-3">
            <div
              v-for="link in shareableLinks"
              :key="link.id"
              class="bg-gray-800 rounded-lg p-3"
            >
              <div class="flex items-start justify-between mb-2">
                <div class="flex-1">
                  <h4 class="text-sm font-medium">{{ link.title }}</h4>
                  <p v-if="link.description" class="text-xs text-gray-400 mt-1">{{ link.description }}</p>
                  <div class="flex items-center space-x-4 mt-2 text-xs text-gray-500">
                    <span>{{ link.language }}</span>
                    <span>{{ link.viewCount }} views</span>
                    <span>{{ formatDate(link.createdAt) }}</span>
                  </div>
                </div>
                
                <div class="flex items-center space-x-2 ml-3">
                  <button
                    @click="copyShareUrl(link.shareUrl)"
                    class="text-blue-400 hover:text-blue-300 transition-colors"
                  >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.367 2.684 3 3 0 00-5.367-2.684z" />
                    </svg>
                  </button>
                  <button
                    @click="deleteShareableLink(link.id)"
                    class="text-red-400 hover:text-red-300 transition-colors"
                  >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                  </button>
                </div>
              </div>
              
              <div class="flex items-center justify-between">
                <code class="text-xs bg-gray-700 px-2 py-1 rounded flex-1 mr-2 truncate">
                  {{ link.shareUrl }}
                </code>
                <div class="flex items-center space-x-2">
                  <span class="text-xs px-2 py-1 rounded" :class="link.isPublic ? 'bg-green-900 text-green-200' : 'bg-gray-700 text-gray-300'">
                    {{ link.isPublic ? 'Public' : 'Private' }}
                  </span>
                </div>
              </div>
            </div>
            
            <div v-if="shareableLinks.length === 0" class="text-gray-500 text-sm italic text-center py-8">
              No shareable links created yet
            </div>
          </div>
        </div>

        <!-- Live Preview Tab -->
        <div v-show="activeTab === 'preview'" class="p-4">
          <div class="mb-4">
            <h3 class="text-sm font-semibold mb-3">Live Previews</h3>
            <button
              @click="showCreatePreviewModal = true"
              class="w-full px-3 py-2 bg-blue-600 hover:bg-blue-700 rounded text-sm font-medium transition-colors"
            >
              Create New Preview
            </button>
          </div>

          <div class="space-y-3">
            <div
              v-for="preview in runningPreviews"
              :key="preview.id"
              class="bg-gray-800 rounded-lg p-3"
            >
              <div class="flex items-start justify-between mb-2">
                <div class="flex-1">
                  <h4 class="text-sm font-medium">{{ preview.name }}</h4>
                  <div class="flex items-center space-x-4 mt-1 text-xs text-gray-500">
                    <span>{{ preview.collaborators.length }} viewers</span>
                    <span>{{ formatDate(preview.createdAt) }}</span>
                  </div>
                </div>
                
                <div class="flex items-center space-x-2 ml-3">
                  <span class="text-xs px-2 py-1 rounded" :class="getPreviewStatusClass(preview.status)">
                    {{ preview.status }}
                  </span>
                  <button
                    @click="openLivePreview(preview)"
                    class="text-blue-400 hover:text-blue-300 transition-colors"
                  >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                    </svg>
                  </button>
                  <button
                    @click="stopLivePreview(preview.id)"
                    class="text-red-400 hover:text-red-300 transition-colors"
                  >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 10a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z" />
                    </svg>
                  </button>
                </div>
              </div>
              
              <div class="text-xs bg-gray-700 px-2 py-1 rounded text-gray-300">
                {{ preview.url }}
              </div>
            </div>
            
            <div v-if="runningPreviews.length === 0" class="text-gray-500 text-sm italic text-center py-8">
              No active live previews
            </div>
          </div>
        </div>

        <!-- Merge Conflicts Tab -->
        <div v-show="activeTab === 'conflicts'" class="p-4">
          <div class="mb-4">
            <h3 class="text-sm font-semibold mb-3">Merge Conflicts ({{ unresolvedConflicts.length }})</h3>
          </div>

          <div class="space-y-3">
            <div
              v-for="conflict in unresolvedConflicts"
              :key="conflict.id"
              class="bg-red-900/20 border border-red-700 rounded-lg p-3"
            >
              <div class="flex items-start justify-between mb-2">
                <div class="flex-1">
                  <h4 class="text-sm font-medium text-red-200">{{ conflict.filePath }}</h4>
                  <div class="text-xs text-red-300 mt-1">
                    {{ conflict.conflicts.length }} conflict(s) detected
                  </div>
                </div>
                
                <div class="flex items-center space-x-2 ml-3">
                  <button
                    @click="requestAIConflictResolution(conflict.id)"
                    class="px-3 py-1 bg-purple-600 hover:bg-purple-700 rounded text-xs font-medium transition-colors"
                  >
                    AI Resolve
                  </button>
                  <button
                    @click="resolveConflictManually(conflict.id)"
                    class="px-3 py-1 bg-blue-600 hover:bg-blue-700 rounded text-xs font-medium transition-colors"
                  >
                    Resolve
                  </button>
                </div>
              </div>
              
              <div class="bg-gray-800 rounded p-2 mt-2">
                <pre class="text-xs text-red-300 whitespace-pre-wrap">{{ formatConflictContent(conflict.conflicts[0]?.content) }}</pre>
              </div>
            </div>
            
            <div v-if="unresolvedConflicts.length === 0" class="text-gray-500 text-sm italic text-center py-8">
              No merge conflicts detected
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Modals -->
    <CreateSessionModal
      v-if="showCreateSession"
      @close="showCreateSession = false"
      @create="handleCreateSession"
    />
    
    <JoinSessionModal
      v-if="showJoinSession"
      @close="showJoinSession = false"
      @join="handleJoinSession"
    />
    
    <CreateLinkModal
      v-if="showCreateLinkModal"
      @close="showCreateLinkModal = false"
      @create="handleCreateLink"
    />
    
    <CreatePreviewModal
      v-if="showCreatePreviewModal"
      @close="showCreatePreviewModal = false"
      @create="handleCreatePreview"
    />
    
    <CommentModal
      v-if="showCommentModal"
      @close="showCommentModal = false"
      @submit="handleAddComment"
    />
    
    <InviteModal
      v-if="showInviteModal"
      @close="showInviteModal = false"
      @invite="handleInvite"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useCollaborationStore } from '../stores/collaborationStore'
import CreateSessionModal from './collaboration/CreateSessionModal.vue'
import JoinSessionModal from './collaboration/JoinSessionModal.vue'
import CreateLinkModal from './collaboration/CreateLinkModal.vue'
import CreatePreviewModal from './collaboration/CreatePreviewModal.vue'
import CommentModal from './collaboration/CommentModal.vue'
import InviteModal from './collaboration/InviteModal.vue'

const collaborationStore = useCollaborationStore()

// State
const activeTab = ref('editing')
const showCreateSession = ref(false)
const showJoinSession = ref(false)
const showCreateLinkModal = ref(false)
const showCreatePreviewModal = ref(false)
const showCommentModal = ref(false)
const showInviteModal = ref(false)

// Tabs configuration
const tabs = [
  { id: 'editing', label: 'Editing' },
  { id: 'comments', label: 'Comments' },
  { id: 'sharing', label: 'Sharing' },
  { id: 'preview', label: 'Live Preview' },
  { id: 'conflicts', label: 'Conflicts' }
]

// Computed properties
const currentSession = computed(() => collaborationStore.currentSession)
const onlineCollaborators = computed(() => collaborationStore.onlineCollaborators)
const activeComments = computed(() => collaborationStore.activeComments)
const shareableLinks = computed(() => collaborationStore.shareableLinks)
const runningPreviews = computed(() => collaborationStore.runningPreviews)
const unresolvedConflicts = computed(() => collaborationStore.unresolvedConflicts)
const connectionStatus = computed(() => collaborationStore.connectionStatus)
const connectionError = computed(() => collaborationStore.connectionError)

const connectionStatusClass = computed(() => {
  switch (connectionStatus.value) {
    case 'connected': return 'bg-green-500'
    case 'connecting': return 'bg-yellow-500'
    case 'error': return 'bg-red-500'
    default: return 'bg-gray-500'
  }
})

const sessionShareUrl = computed(() => {
  return currentSession.value ? `https://super-ide.dev/collaborate/${currentSession.value.id}` : ''
})

// Methods
function getInitials(name: string): string {
  return name.split(' ').map(n => n[0]).join('').toUpperCase().slice(0, 2)
}

function getStatusClass(status: string): string {
  switch (status) {
    case 'online': return 'bg-green-500'
    case 'away': return 'bg-yellow-500'
    default: return 'bg-gray-500'
  }
}

function getPriorityClass(priority: string): string {
  switch (priority) {
    case 'high': return 'bg-red-900 text-red-200'
    case 'normal': return 'bg-blue-900 text-blue-200'
    default: return 'bg-gray-700 text-gray-300'
  }
}

function getPreviewStatusClass(status: string): string {
  switch (status) {
    case 'running': return 'bg-green-900 text-green-200'
    case 'error': return 'bg-red-900 text-red-200'
    default: return 'bg-gray-700 text-gray-300'
  }
}

function formatDate(date: Date): string {
  return new Intl.DateTimeFormat('en-US', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  }).format(date)
}

function formatConflictContent(content: string): string {
  return content?.replace(/<<<<<<< ours|=======|>>>>>>> theirs/g, '\n$&\n') || ''
}

function copySessionUrl(): void {
  navigator.clipboard.writeText(sessionShareUrl.value)
  // Show toast notification in real implementation
}

function copyShareUrl(url: string): void {
  navigator.clipboard.writeText(url)
  // Show toast notification in real implementation
}

function leaveSession(): void {
  collaborationStore.leaveSession()
  activeTab.value = 'editing'
}

function handleCreateSession(data: { name: string; description?: string; isPublic: boolean }): void {
  collaborationStore.createCollaborationSession(data.name, data.description, data.isPublic)
  showCreateSession.value = false
}

function handleJoinSession(sessionId: string): void {
  collaborationStore.joinCollaborationSession(sessionId)
  showJoinSession.value = false
}

function handleCreateLink(data: { title: string; code: string; language: string; description?: string; isPublic: boolean }): void {
  collaborationStore.createShareableLink(data.title, data.code, data.language, data.description, data.isPublic)
  showCreateLinkModal.value = false
}

function handleCreatePreview(data: { name: string; url: string; isPublic: boolean }): void {
  collaborationStore.createLivePreview(data.name, data.url, data.isPublic)
  showCreatePreviewModal.value = false
}

function handleAddComment(data: { filePath: string; lineNumber: number; content: string; tags: string[]; priority: string }): void {
  collaborationStore.addCodeComment(data.filePath, data.lineNumber, data.content, data.tags, data.priority as any)
  showCommentModal.value = false
}

function handleInvite(emails: string[]): void {
  // In real implementation, send invitations
  console.log('Inviting users:', emails)
  showInviteModal.value = false
}

function resolveComment(commentId: string): void {
  collaborationStore.resolveComment(commentId)
}

function deleteShareableLink(linkId: string): void {
  const index = collaborationStore.shareableLinks.findIndex(link => link.id === linkId)
  if (index !== -1) {
    collaborationStore.shareableLinks.splice(index, 1)
  }
}

function openLivePreview(preview: any): void {
  window.open(preview.url, '_blank')
}

function stopLivePreview(previewId: string): void {
  const preview = collaborationStore.livePreviews.find(p => p.id === previewId)
  if (preview) {
    preview.status = 'stopped'
  }
}

function resolveConflictManually(conflictId: string): void {
  // In real implementation, open manual conflict resolution modal
  // console.log('Opening manual conflict resolution for:', conflictId)
}

function requestAIConflictResolution(conflictId: string): void {
  collaborationStore.requestAIConflictResolution(conflictId)
}

// Lifecycle
onMounted(() => {
  collaborationStore.initializeCollaboration()
})

onUnmounted(() => {
  collaborationStore.disconnectWebSocket()
})
</script>

<style scoped>
.collaboration-panel {
  min-height: 400px;
}

/* Custom scrollbar for dark theme */
.collaboration-panel ::-webkit-scrollbar {
  width: 6px;
}

.collaboration-panel ::-webkit-scrollbar-track {
  background: rgb(31, 41, 55);
}

.collaboration-panel ::-webkit-scrollbar-thumb {
  background: rgb(75, 85, 99);
  border-radius: 3px;
}

.collaboration-panel ::-webkit-scrollbar-thumb:hover {
  background: rgb(107, 114, 128);
}
</style>