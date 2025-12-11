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