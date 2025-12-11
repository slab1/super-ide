export interface FileInfo {
  path: string
  name: string
  type: 'file' | 'directory'
  size?: number
  modified?: string
  children?: FileInfo[]
}

export interface CodeCompletion {
  label: string
  kind: string
  detail?: string
  documentation?: string
  insertText: string
  range?: {
    startLineNumber: number
    startColumn: number
    endLineNumber: number
    endColumn: number
  }
}

export interface TerminalSession {
  id: string
  name: string
  pid: number
  status: 'active' | 'inactive' | 'error'
  currentDirectory: string
}

export interface AIMessage {
  id: string
  type: 'user' | 'assistant'
  content: string
  timestamp: Date
  suggestions?: CodeCompletion[]
}

export interface ProjectInfo {
  name: string
  path: string
  language: string
  dependencies: string[]
  gitStatus?: 'clean' | 'modified' | 'untracked'
}

export interface Settings {
  theme: 'dark' | 'light'
  fontSize: number
  fontFamily: string
  tabSize: number
  wordWrap: boolean
  minimap: boolean
  lineNumbers: boolean
  autoSave: boolean
}

// Snippet types
export interface SnippetVariable {
  name: string
  description: string
  defaultValue?: string
  placeholder?: string
}

export interface Snippet {
  id: string
  name: string
  description: string
  code: string
  language: string
  languages: string[]
  category: string
  tags: string[]
  variables?: SnippetVariable[]
  favorite: boolean
  usageCount: number
  lastUsed: string
  createdAt: string
  updatedAt: string
}

export interface SnippetCategory {
  id: string
  name: string
  icon: string
}

// Code Timeline types
export interface CodeChange {
  id: string
  filePath: string
  type: 'add' | 'modify' | 'delete'
  content: string
  lineNumber?: number
  timestamp: Date
  author: string
  message: string
}

export interface CodeTimeline {
  changes: CodeChange[]
  branches: TimelineBranch[]
  currentBranch: string
}

// Multi-cursor types
export interface CursorPosition {
  lineNumber: number
  column: number
}

export interface MultiCursorSession {
  id: string
  cursors: CursorPosition[]
  selections: {
    startLineNumber: number
    startColumn: number
    endLineNumber: number
    endColumn: number
  }[]
  active: boolean
}

// Keyboard Shortcut types
export interface KeyboardShortcut {
  id: string
  name: string
  keybinding: string
  description: string
  category: string
  command: string
  contexts?: string[]
}

export interface ShortcutCategory {
  id: string
  name: string
  description: string
  shortcuts: KeyboardShortcut[]
}

// Collaboration types
export interface Collaborator {
  id: string
  name: string
  email: string
  avatar?: string
  cursor?: {
    lineNumber: number
    column: number
    filePath: string
  }
  selection?: {
    startLineNumber: number
    startColumn: number
    endLineNumber: number
    endColumn: number
  }
  status: 'online' | 'away' | 'offline'
  joinedAt: Date
  color: string
}

export interface CollaborationSession {
  id: string
  name: string
  description?: string
  creator: string
  collaborators: Collaborator[]
  files: string[]
  isPublic: boolean
  shareUrl?: string
  createdAt: Date
  lastActivity: Date
  status: 'active' | 'paused' | 'ended'
}

export interface CodeComment {
  id: string
  filePath: string
  lineNumber: number
  content: string
  author: Collaborator
  createdAt: Date
  updatedAt?: Date
  replies: CommentReply[]
  resolved: boolean
  tags: string[]
  priority: 'low' | 'normal' | 'high'
}

export interface CommentReply {
  id: string
  content: string
  author: Collaborator
  createdAt: Date
}

export interface ShareableLink {
  id: string
  title: string
  description?: string
  code: string
  language: string
  syntaxHighlighted: boolean
  isPublic: boolean
  expiresAt?: Date
  password?: string
  viewCount: number
  createdBy: string
  createdAt: Date
  shareUrl: string
}

export interface MergeConflict {
  id: string
  filePath: string
  conflicts: ConflictSegment[]
  baseContent: string
  ourContent: string
  theirContent: string
  resolved: boolean
  resolution?: ConflictResolution
}

export interface ConflictSegment {
  startLine: number
  endLine: number
  content: string
  type: 'ours' | 'theirs' | 'base'
}

export interface ConflictResolution {
  resolvedContent: string
  resolvedBy: string
  strategy: 'ours' | 'theirs' | 'manual' | 'ai-suggested'
  timestamp: Date
}

export interface LivePreview {
  id: string
  name: string
  url: string
  isPublic: boolean
  collaborators: Collaborator[]
  createdBy: string
  createdAt: Date
  lastActivity: Date
  status: 'running' | 'stopped' | 'error'
}

export interface CollaborationEvent {
  type: 'cursor_move' | 'selection_change' | 'code_edit' | 'file_open' | 'file_close' | 'user_join' | 'user_leave' | 'comment_add' | 'comment_resolve'
  sessionId: string
  userId: string
  timestamp: Date
  data: any
}

export interface WebSocketMessage {
  type: 'collaboration_event' | 'cursor_update' | 'code_change' | 'user_presence' | 'comment_update' | 'heartbeat'
  sessionId?: string
  payload: any
  timestamp: Date
}