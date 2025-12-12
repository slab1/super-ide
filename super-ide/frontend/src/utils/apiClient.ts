import axios, { AxiosResponse } from 'axios'
import type { 
  ApiResponse, 
  FileTreeNode, 
  FileOperationResult, 
  SearchResult, 
  GitStatus, 
  GitBranch 
} from '../types'

// Create axios instance with default configuration
const api = axios.create({
  baseURL: '/api',
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json'
  }
})

// Response interceptor for consistent error handling
api.interceptors.response.use(
  (response: AxiosResponse<ApiResponse<any>>) => {
    return response
  },
  (error) => {
    console.error('API Error:', error)
    return Promise.reject(error)
  }
)

// Generic API call helper
async function apiCall<T>(
  method: 'GET' | 'POST' | 'PUT' | 'DELETE',
  url: string,
  data?: any,
  params?: any
): Promise<T> {
  try {
    const response = await api({
      method,
      url,
      data,
      params
    })
    
    if (!response.data.success) {
      throw new Error(response.data.error || 'API call failed')
    }
    
    return response.data.data
  } catch (error) {
    if (error instanceof Error) {
      throw error
    }
    throw new Error('Unknown API error')
  }
}

// File Operations API
export class FileAPI {
  static async loadFile(path: string): Promise<string> {
    return apiCall<string>('GET', `/files/${encodeURIComponent(path)}`)
  }

  static async saveFile(path: string, content: string): Promise<FileOperationResult> {
    return apiCall<FileOperationResult>('PUT', `/files/${encodeURIComponent(path)}`, {
      content
    })
  }

  static async createFile(path: string, content?: string): Promise<FileOperationResult> {
    return apiCall<FileOperationResult>('POST', '/files/create', {
      path,
      content: content || '',
      is_directory: false
    })
  }

  static async createDirectory(path: string): Promise<FileOperationResult> {
    return apiCall<FileOperationResult>('POST', '/files/create', {
      path,
      is_directory: true
    })
  }

  static async deleteFile(path: string): Promise<FileOperationResult> {
    return apiCall<FileOperationResult>('DELETE', `/files/${encodeURIComponent(path)}`)
  }

  static async getFileTree(): Promise<FileTreeNode[]> {
    return apiCall<FileTreeNode[]>('GET', '/files/tree')
  }

  static async searchFiles(pattern: string, root?: string): Promise<SearchResult[]> {
    const params = new URLSearchParams({ pattern })
    if (root) params.append('root', root)
    return apiCall<SearchResult[]>('GET', `/files/search?${params}`)
  }

  static async getProjectInfo(): Promise<any> {
    return apiCall<any>('GET', '/project/info')
  }
}

// Git Operations API
export class GitAPI {
  static async getStatus(): Promise<GitStatus | null> {
    try {
      return await apiCall<GitStatus>('GET', '/git/status')
    } catch (error) {
      // If not a repository, return null instead of throwing
      if (error instanceof Error && error.message.includes('Not a git repository')) {
        return null
      }
      throw error
    }
  }

  static async getBranches(): Promise<GitBranch[]> {
    try {
      return await apiCall<GitBranch[]>('GET', '/git/branches')
    } catch (error) {
      // If not a repository, return empty array instead of throwing
      if (error instanceof Error && error.message.includes('Not a git repository')) {
        return []
      }
      throw error
    }
  }

  static async commit(message: string): Promise<string> {
    return apiCall<string>('POST', '/git/commit', { message })
  }

  // Placeholder methods for Phase 3 enhancement
  static async push(): Promise<void> {
    throw new Error('Git push not yet implemented - requires new API endpoint')
  }

  static async pull(): Promise<void> {
    throw new Error('Git pull not yet implemented - requires new API endpoint')
  }

  static async createBranch(name: string): Promise<void> {
    throw new Error('Git create branch not yet implemented - requires new API endpoint')
  }

  static async switchBranch(name: string): Promise<void> {
    throw new Error('Git switch branch not yet implemented - requires new API endpoint')
  }

  static async getDiff(filePath?: string): Promise<string> {
    throw new Error('Git diff not yet implemented - requires new API endpoint')
  }

  static async stageFiles(files: string[]): Promise<void> {
    throw new Error('Git staging not yet implemented - requires new API endpoint')
  }

  static async unstageFiles(files: string[]): Promise<void> {
    throw new Error('Git unstaging not yet implemented - requires new API endpoint')
  }
}

// AI class AIAPI {
 Operations API
export  static async chat(message: string, context?: any): Promise<string> {
    return apiCall<string>('POST', '/ai/chat', {
      message,
      context
    })
  }

  static async getCompletions(code: string, position: number, language: string): Promise<any[]> {
    return apiCall<any[]>('POST', '/ai/completions', {
      code,
      position,
      language
    })
  }

  static async analyzeCode(code: string, language: string): Promise<any> {
    return apiCall<any>('POST', '/ai/analyze', {
      code,
      language
    })
  }
}

// Utility functions
export function isApiError(error: any): error is Error {
  return error instanceof Error && error.message.includes('API')
}

export function getErrorMessage(error: any): string {
  if (error instanceof Error) {
    return error.message
  }
  if (typeof error === 'string') {
    return error
  }
  return 'An unknown error occurred'
}

// Health check
export async function healthCheck(): Promise<boolean> {
  try {
    const response = await api.get('/health')
    return response.data.success === true
  } catch {
    return false
  }
}

export default api