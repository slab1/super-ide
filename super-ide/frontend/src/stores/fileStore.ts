import { defineStore } from 'pinia'
import axios from 'axios'
import type { FileInfo, ProjectInfo } from '../types'

export const useFileStore = defineStore('file', {
  state: () => ({
    currentFile: null as FileInfo | null,
    fileTree: [] as FileInfo[],
    projectInfo: null as ProjectInfo | null,
    isLoading: false,
    error: null as string | null
  }),

  actions: {
    async loadFile(path: string): Promise<string> {
      try {
        this.isLoading = true
        const response = await axios.get(`/api/files/${encodeURIComponent(path)}`)
        return response.data.content
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to load file'
        throw error
      } finally {
        this.isLoading = false
      }
    },

    async saveFile(path: string, content: string): Promise<void> {
      try {
        await axios.put(`/api/files/${encodeURIComponent(path)}`, { content })
        this.error = null
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to save file'
        throw error
      }
    },

    async getFileTree(): Promise<FileInfo[]> {
      try {
        this.isLoading = true
        const response = await axios.get('/api/files/tree')
        this.fileTree = response.data
        return response.data
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to load file tree'
        throw error
      } finally {
        this.isLoading = false
      }
    },

    async createFile(name: string, path?: string): Promise<FileInfo> {
      try {
        const response = await axios.post('/api/files/create', { name, path })
        return response.data
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to create file'
        throw error
      }
    },

    async createFolder(name: string, path?: string): Promise<FileInfo> {
      try {
        const response = await axios.post('/api/folders/create', { name, path })
        return response.data
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to create folder'
        throw error
      }
    },

    async deleteFile(path: string): Promise<void> {
      try {
        await axios.delete(`/api/files/${encodeURIComponent(path)}`)
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to delete file'
        throw error
      }
    },

    async getProjectInfo(): Promise<ProjectInfo> {
      try {
        const response = await axios.get('/api/project')
        this.projectInfo = response.data
        return response.data
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to load project info'
        throw error
      }
    },

    async formatCurrentFile(): Promise<void> {
      if (!this.currentFile) return
      
      try {
        const response = await axios.post('/api/format', {
          path: this.currentFile.path,
          language: this.getLanguageFromFileName(this.currentFile.name)
        })
        
        // Update current file content with formatted version
        if (this.currentFile && response.data.content) {
          this.currentFile.content = response.data.content
        }
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to format file'
        throw error
      }
    },

    getCurrentFile(): FileInfo | null {
      return this.currentFile
    },

    setCurrentFile(file: FileInfo | null) {
      this.currentFile = file
    },

    getLanguageFromFileName(filename: string): string {
      const ext = filename.split('.').pop()?.toLowerCase()
      const languageMap: Record<string, string> = {
        'rs': 'rust',
        'py': 'python',
        'js': 'javascript',
        'ts': 'typescript',
        'jsx': 'javascript',
        'tsx': 'typescript',
        'go': 'go',
        'java': 'java',
        'cpp': 'cpp',
        'c': 'c',
        'h': 'cpp',
        'hpp': 'cpp',
        'cs': 'csharp',
        'php': 'php',
        'rb': 'ruby',
        'swift': 'swift',
        'kt': 'kotlin',
        'scala': 'scala',
        'sh': 'shell',
        'bash': 'shell',
        'zsh': 'shell',
        'fish': 'shell',
        'ps1': 'powershell',
        'json': 'json',
        'yaml': 'yaml',
        'yml': 'yaml',
        'xml': 'xml',
        'html': 'html',
        'css': 'css',
        'scss': 'scss',
        'sass': 'sass',
        'less': 'less',
        'sql': 'sql',
        'md': 'markdown',
        'markdown': 'markdown',
        'txt': 'plaintext',
        'dockerfile': 'dockerfile',
        'toml': 'toml',
        'ini': 'ini',
        'conf': 'ini'
      }
      return languageMap[ext || ''] || 'plaintext'
    }
  }
})