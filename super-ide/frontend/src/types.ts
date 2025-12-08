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