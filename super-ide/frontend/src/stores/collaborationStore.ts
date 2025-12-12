import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type {
  Collaborator,
  CollaborationSession,
  CodeComment,
  ShareableLink,
  MergeConflict,
  LivePreview,
  CollaborationEvent,
  WebSocketMessage,
  ConflictResolution
} from '../types'

export const useCollaborationStore = defineStore('collaboration', () => {
  // State
  const currentSession = ref<CollaborationSession | null>(null)
  const collaborators = ref<Collaborator[]>([])
  const currentUser = ref<Collaborator | null>(null)
  const isConnected = ref(false)
  const comments = ref<CodeComment[]>([])
  const shareableLinks = ref<ShareableLink[]>([])
  const livePreviews = ref<LivePreview[]>([])
  const mergeConflicts = ref<MergeConflict[]>([])
  const connectionStatus = ref<'disconnected' | 'connecting' | 'connected' | 'error'>('disconnected')
  const connectionError = ref<string | null>(null)

  // WebSocket connection
  let ws: WebSocket | null = null
  const reconnectAttempts = ref(0)
  const maxReconnectAttempts = 5
  const reconnectDelay = 1000

  // Computed
  const onlineCollaborators = computed(() => 
    collaborators.value.filter(c => c.status === 'online')
  )

  const activeComments = computed(() => 
    comments.value.filter(c => !c.resolved)
  )

  const unresolvedConflicts = computed(() => 
    mergeConflicts.value.filter(c => !c.resolved)
  )

  const runningPreviews = computed(() => 
    livePreviews.value.filter(p => p.status === 'running')
  )

  // Actions
  function initializeCollaboration() {
    // Initialize current user
    currentUser.value = {
      id: generateId(),
      name: 'Current User',
      email: 'user@example.com',
      status: 'online',
      joinedAt: new Date(),
      color: generateUserColor()
    }
  }

  function generateId(): string {
    return Math.random().toString(36).substr(2, 9)
  }

  function generateUserColor(): string {
    const colors = [
      '#FF6B6B', '#4ECDC4', '#45B7D1', '#96CEB4', '#FFEAA7',
      '#DDA0DD', '#98D8C8', '#F7DC6F', '#BB8FCE', '#85C1E9'
    ]
    return colors[Math.floor(Math.random() * colors.length)]
  }

  function createCollaborationSession(name: string, description?: string, isPublic = false): CollaborationSession {
    const session: CollaborationSession = {
      id: generateId(),
      name,
      description,
      creator: currentUser.value?.id || '',
      collaborators: currentUser.value ? [currentUser.value] : [],
      files: [],
      isPublic,
      createdAt: new Date(),
      lastActivity: new Date(),
      status: 'active'
    }

    currentSession.value = session
    return session
  }

  function joinCollaborationSession(sessionId: string): void {
    // In a real implementation, this would connect to a WebSocket server
    console.log(`Joining collaboration session: ${sessionId}`)
    
    // Simulate adding mock collaborators
    const mockCollaborators: Collaborator[] = [
      {
        id: 'user2',
        name: 'Alice Johnson',
        email: 'alice@example.com',
        status: 'online',
        joinedAt: new Date(Date.now() - 300000), // 5 minutes ago
        color: '#FF6B6B',
        cursor: { lineNumber: 15, column: 8, filePath: '/src/components/App.vue' },
        selection: {
          startLineNumber: 10,
          startColumn: 5,
          endLineNumber: 20,
          endColumn: 10
        }
      },
      {
        id: 'user3',
        name: 'Bob Smith',
        email: 'bob@example.com',
        status: 'away',
        joinedAt: new Date(Date.now() - 600000), // 10 minutes ago
        color: '#4ECDC4',
        cursor: { lineNumber: 42, column: 15, filePath: '/src/stores/main.ts' }
      }
    ]

    collaborators.value = [...(currentUser.value ? [currentUser.value] : []), ...mockCollaborators]
    connectWebSocket()
  }

  function connectWebSocket(): void {
    if (connectionStatus.value === 'connected') return

    connectionStatus.value = 'connecting'
    connectionError.value = null

    try {
      // In a real implementation, this would connect to your WebSocket server
      // const wsUrl = `ws://localhost:8080/collaboration/${currentSession.value?.id}`
      // ws = new WebSocket(wsUrl)

      // For demo purposes, simulate WebSocket connection
      setTimeout(() => {
        isConnected.value = true
        connectionStatus.value = 'connected'
        console.log('WebSocket connected (simulated)')
        
        // Simulate receiving collaboration events
        simulateCollaborationEvents()
      }, 1000)

      // ws.onopen = () => {
      //   isConnected.value = true
      //   connectionStatus.value = 'connected'
      //   reconnectAttempts.value = 0
      // }

      // ws.onmessage = (event) => {
      //   handleWebSocketMessage(JSON.parse(event.data))
      // }

      // ws.onclose = () => {
      //   isConnected.value = false
      //   connectionStatus.value = 'disconnected'
      //   attemptReconnect()
      // }

      // ws.onerror = (error) => {
      //   connectionError.value = 'WebSocket connection error'
      //   connectionStatus.value = 'error'
      // }

    } catch (error) {
      connectionError.value = 'Failed to establish WebSocket connection'
      connectionStatus.value = 'error'
    }
  }

  function disconnectWebSocket(): void {
    if (ws) {
      ws.close()
      ws = null
    }
    isConnected.value = false
    connectionStatus.value = 'disconnected'
  }

  function attemptReconnect(): void {
    if (reconnectAttempts.value < maxReconnectAttempts) {
      reconnectAttempts.value++
      setTimeout(() => {
        connectWebSocket()
      }, reconnectDelay * reconnectAttempts.value)
    }
  }

  function handleWebSocketMessage(message: WebSocketMessage): void {
    switch (message.type) {
      case 'cursor_update':
        updateCollaboratorCursor(message.payload.userId, message.payload.cursor)
        break
      case 'code_change':
        // Handle code changes from other collaborators
        console.log('Code change received:', message.payload)
        break
      case 'user_presence':
        updateUserPresence(message.payload.userId, message.payload.status)
        break
      case 'comment_update':
        handleCommentUpdate(message.payload)
        break
      case 'heartbeat':
        // Handle heartbeat to maintain connection
        break
    }
  }

  function updateCollaboratorCursor(userId: string, cursor: any): void {
    const collaborator = collaborators.value.find(c => c.id === userId)
    if (collaborator) {
      collaborator.cursor = cursor
    }
  }

  function updateUserPresence(userId: string, status: 'online' | 'away' | 'offline'): void {
    const collaborator = collaborators.value.find(c => c.id === userId)
    if (collaborator) {
      collaborator.status = status
    }
  }

  function handleCommentUpdate(payload: any): void {
    // Handle comment updates from other users
    if (payload.action === 'add') {
      comments.value.push(payload.comment)
    } else if (payload.action === 'update') {
      const index = comments.value.findIndex(c => c.id === payload.comment.id)
      if (index !== -1) {
        comments.value[index] = payload.comment
      }
    } else if (payload.action === 'resolve') {
      const comment = comments.value.find(c => c.id === payload.commentId)
      if (comment) {
        comment.resolved = true
      }
    }
  }

  function updateCurrentUserCursor(lineNumber: number, column: number, filePath: string): void {
    if (currentUser.value) {
      currentUser.value.cursor = { lineNumber, column, filePath }
      
      // Broadcast cursor position to other collaborators
      if (ws && isConnected.value) {
        const message: WebSocketMessage = {
          type: 'cursor_update',
          sessionId: currentSession.value?.id,
          payload: {
            userId: currentUser.value.id,
            cursor: currentUser.value.cursor
          },
          timestamp: new Date()
        }
        // ws.send(JSON.stringify(message))
        console.log('Cursor update sent (simulated):', message)
      }
    }
  }

  function addCodeComment(
    filePath: string,
    lineNumber: number,
    content: string,
    tags: string[] = [],
    priority: 'low' | 'normal' | 'high' = 'normal'
  ): CodeComment {
    const comment: CodeComment = {
      id: generateId(),
      filePath,
      lineNumber,
      content,
      author: currentUser.value!,
      createdAt: new Date(),
      replies: [],
      resolved: false,
      tags,
      priority
    }

    comments.value.push(comment)

    // Broadcast comment to other collaborators
    if (ws && isConnected.value) {
      const message: WebSocketMessage = {
        type: 'comment_update',
        sessionId: currentSession.value?.id,
        payload: {
          action: 'add',
          comment
        },
        timestamp: new Date()
      }
      // ws.send(JSON.stringify(message))
      console.log('Comment added (simulated):', message)
    }

    return comment
  }

  function resolveComment(commentId: string): void {
    const comment = comments.value.find(c => c.id === commentId)
    if (comment) {
      comment.resolved = true

      // Broadcast resolution to other collaborators
      if (ws && isConnected.value) {
        const message: WebSocketMessage = {
          type: 'comment_update',
          sessionId: currentSession.value?.id,
          payload: {
            action: 'resolve',
            commentId
          },
          timestamp: new Date()
        }
        // ws.send(JSON.stringify(message))
        console.log('Comment resolved (simulated):', message)
      }
    }
  }

  function createShareableLink(
    title: string,
    code: string,
    language: string,
    description?: string,
    isPublic = true,
    expiresAt?: Date,
    password?: string
  ): ShareableLink {
    const link: ShareableLink = {
      id: generateId(),
      title,
      description,
      code,
      language,
      syntaxHighlighted: true,
      isPublic,
      expiresAt,
      password,
      viewCount: 0,
      createdBy: currentUser.value?.id || '',
      createdAt: new Date(),
      shareUrl: `https://super-ide.dev/share/${generateId()}`
    }

    shareableLinks.value.push(link)
    return link
  }

  function createLivePreview(name: string, url: string, isPublic = false): LivePreview {
    const preview: LivePreview = {
      id: generateId(),
      name,
      url,
      isPublic,
      collaborators: currentUser.value ? [currentUser.value] : [],
      createdBy: currentUser.value?.id || '',
      createdAt: new Date(),
      lastActivity: new Date(),
      status: 'running'
    }

    livePreviews.value.push(preview)
    return preview
  }

  function detectMergeConflicts(filePath: string, baseContent: string, ourContent: string, theirContent: string): MergeConflict {
    const conflicts = detectConflictSegments(baseContent, ourContent, theirContent)
    
    const conflict: MergeConflict = {
      id: generateId(),
      filePath,
      conflicts,
      baseContent,
      ourContent,
      theirContent,
      resolved: false
    }

    mergeConflicts.value.push(conflict)
    return conflict
  }

  function detectConflictSegments(base: string, ours: string, theirs: string): any[] {
    // Simple conflict detection - in a real implementation, this would use proper diff algorithms
    const conflicts: any[] = []
    
    if (ours !== theirs) {
      conflicts.push({
        startLine: 1,
        endLine: Math.max(ours.split('\n').length, theirs.split('\n').length),
        content: '<<<<<<< ours\n' + ours + '\n=======\n' + theirs + '\n>>>>>>> theirs',
        type: 'conflict'
      })
    }

    return conflicts
  }

  function resolveMergeConflict(
    conflictId: string,
    resolvedContent: string,
    strategy: 'ours' | 'theirs' | 'manual' | 'ai-suggested'
  ): void {
    const conflict = mergeConflicts.value.find(c => c.id === conflictId)
    if (conflict) {
      conflict.resolved = true
      conflict.resolution = {
        resolvedContent,
        resolvedBy: currentUser.value?.id || '',
        strategy,
        timestamp: new Date()
      }
    }
  }

  function requestAIConflictResolution(conflictId: string): void {
    const conflict = mergeConflicts.value.find(c => c.id === conflictId)
    if (conflict) {
      // Simulate AI conflict resolution
      const aiResolution: ConflictResolution = {
        resolvedContent: `// AI-suggested resolution for ${conflict.filePath}\n// This is a simulated AI resolution\n${conflict.ourContent}`,
        resolvedBy: 'ai-assistant',
        strategy: 'ai-suggested',
        timestamp: new Date()
      }

      conflict.resolution = aiResolution
      console.log('AI conflict resolution generated:', aiResolution)
    }
  }

  function simulateCollaborationEvents(): void {
    // Simulate receiving events from other collaborators
    setInterval(() => {
      if (Math.random() > 0.7) { // 30% chance every interval
        const event: CollaborationEvent = {
          type: 'cursor_move',
          sessionId: currentSession.value?.id || '',
          userId: 'user2',
          timestamp: new Date(),
          data: {
            lineNumber: Math.floor(Math.random() * 100) + 1,
            column: Math.floor(Math.random() * 50) + 1,
            filePath: '/src/components/App.vue'
          }
        }
        updateCollaboratorCursor(event.userId, event.data)
      }
    }, 3000)
  }

  function leaveSession(): void {
    disconnectWebSocket()
    currentSession.value = null
    collaborators.value = []
    comments.value = []
  }

  return {
    // State
    currentSession,
    collaborators,
    currentUser,
    isConnected,
    comments,
    shareableLinks,
    livePreviews,
    mergeConflicts,
    connectionStatus,
    connectionError,

    // Computed
    onlineCollaborators,
    activeComments,
    unresolvedConflicts,
    runningPreviews,

    // Actions
    initializeCollaboration,
    createCollaborationSession,
    joinCollaborationSession,
    connectWebSocket,
    disconnectWebSocket,
    updateCurrentUserCursor,
    addCodeComment,
    resolveComment,
    createShareableLink,
    createLivePreview,
    detectMergeConflicts,
    resolveMergeConflict,
    requestAIConflictResolution,
    leaveSession
  }
})