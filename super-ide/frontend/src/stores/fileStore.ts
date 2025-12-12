import { defineStore } from 'pinia'
import axios from 'axios'
import type { FileInfo, FileTreeNode, FileOperationResult, SearchResult, ApiResponse } from '../types'

export const useFileStore = defineStore('file', {
  state: () => ({
    currentFile: null as FileInfo | null,
    currentFileContent: '' as string,
    fileTree: [] as FileTreeNode[],
    searchResults: [] as SearchResult[],
    projectInfo: null as any,
    isLoading: false,
    error: null as string | null
  }),

  actions: {
    async loadFile(path: string): Promise<string> {
      try {
        this.isLoading = true
        const response = await axios.get<ApiResponse<string>>(`/api/files/${encodeURIComponent(path)}`)
        
        if (!response.data.success) {
          throw new Error(response.data.error || 'Failed to load file')
        }
        
        const content = response.data.data || ''
        this.currentFileContent = content
        
        // Update current file info
        this.currentFile = {
          path,
          name: path.split('/').pop() || '',
          is_directory: false,
          size: content.length,
          modified_at: new Date().toISOString(),
          is_hidden: false
        }
        
        return content
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to load file'
        throw error
      } finally {
        this.isLoading = false
      }
    },

    async saveFile(path: string, content: string): Promise<FileOperationResult> {
      try {
        this.isLoading = true
        const response = await axios.put<ApiResponse<string>>(`/api/files/${encodeURIComponent(path)}`, { 
          content 
        })
        
        if (!response.data.success) {
          throw new Error(response.data.error || 'Failed to save file')
        }
        
        const result: FileOperationResult = {
          success: true,
          message: response.data.data || 'File saved successfully',
          bytes_written: content.length
        }
        
        this.error = null
        return result
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to save file'
        throw error
      } finally {
        this.isLoading = false
      }
    },

    async createFile(path: string, content?: string): Promise<FileOperationResult> {
      try {
        const response = await axios.post<ApiResponse<string>>('/api/files/create', { 
          path,
          content: content || '',
          is_directory: false 
        })
        
        if (!response.data.success) {
          throw new Error(response.data.error || 'Failed to create file')
        }
        
        const result: FileOperationResult = {
          success: true,
          message: response.data.data || 'File created successfully'
        }
        
        // Refresh file tree
        await this.getFileTree()
        return result
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to create file'
        throw error
      }
    },

    async createDirectory(path: string): Promise<FileOperationResult> {
      try {
        const response = await axios.post<ApiResponse<string>>('/api/files/create', { 
          path,
          is_directory: true 
        })
        
        if (!response.data.success) {
          throw new Error(response.data.error || 'Failed to create directory')
        }
        
        const result: FileOperationResult = {
          success: true,
          message: response.data.data || 'Directory created successfully'
        }
        
        // Refresh file tree
        await this.getFileTree()
        return result
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to create directory'
        throw error
      }
    },

    async deleteFile(path: string): Promise<FileOperationResult> {
      try {
        this.isLoading = true
        const response = await axios.delete<ApiResponse<string>>(`/api/files/${encodeURIComponent(path)}`)
        
        if (!response.data.success) {
          throw new Error(response.data.error || 'Failed to delete file')
        }
        
        const result: FileOperationResult = {
          success: true,
          message: response.data.data || 'File deleted successfully'
        }
        
        // Refresh file tree
        await this.getFileTree()
        return result
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to delete file'
        throw error
      } finally {
        this.isLoading = false
      }
    },

    async getFileTree(): Promise<FileTreeNode[]> {
      try {
        this.isLoading = true
        const response = await axios.get<ApiResponse<FileTreeNode[]>>('/api/files/tree')
        
        if (!response.data.success) {
          throw new Error(response.data.error || 'Failed to load file tree')
        }
        
        this.fileTree = response.data.data || []
        return this.fileTree
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to load file tree'
        throw error
      } finally {
        this.isLoading = false
      }
    },

    async searchFiles(pattern: string, root?: string): Promise<SearchResult[]> {
      try {
        this.isLoading = true
        const params = new URLSearchParams({ pattern })
        if (root) params.append('root', root)
        
        const response = await axios.get<ApiResponse<SearchResult[]>>(`/api/files/search?${params}`)
        
        if (!response.data.success) {
          throw new Error(response.data.error || 'Failed to search files')
        }
        
        this.searchResults = response.data.data || []
        return this.searchResults
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to search files'
        throw error
      } finally {
        this.isLoading = false
      }
    },

    async getProjectInfo(): Promise<any> {
      try {
        const response = await axios.get<ApiResponse<any>>('/api/project/info')
        
        if (!response.data.success) {
          throw new Error(response.data.error || 'Failed to load project info')
        }
        
        this.projectInfo = response.data.data
        return this.projectInfo
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to load project info'
        throw error
      }
    },

    // Utility methods
    getCurrentFile(): FileInfo | null {
      return this.currentFile
    },

    getCurrentFileContent(): string {
      return this.currentFileContent
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