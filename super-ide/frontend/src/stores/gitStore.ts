import { defineStore } from 'pinia'
import axios from 'axios'

export const useGitStore = defineStore('git', {
  state: () => ({
    status: null as 'clean' | 'modified' | 'untracked' | null,
    branches: [] as string[],
    currentBranch: '' as string,
    changes: [] as any[],
    isLoading: false,
    error: null as string | null
  }),

  actions: {
    async getStatus(): Promise<any> {
      try {
        this.isLoading = true
        const response = await axios.get('/api/git/status')
        this.status = response.data.status
        this.currentBranch = response.data.currentBranch
        this.changes = response.data.changes || []
        return response.data
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to get git status'
        throw error
      } finally {
        this.isLoading = false
      }
    },

    async getBranches(): Promise<string[]> {
      try {
        const response = await axios.get('/api/git/branches')
        this.branches = response.data.branches
        return response.data.branches
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to get branches'
        throw error
      }
    },

    async createBranch(name: string): Promise<void> {
      try {
        await axios.post('/api/git/branches', { name })
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to create branch'
        throw error
      }
    },

    async switchBranch(name: string): Promise<void> {
      try {
        await axios.post('/api/git/switch', { name })
        this.currentBranch = name
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to switch branch'
        throw error
      }
    },

    async commit(message: string): Promise<void> {
      try {
        await axios.post('/api/git/commit', { message })
        // Refresh status after commit
        await this.getStatus()
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to commit'
        throw error
      }
    },

    async push(): Promise<void> {
      try {
        await axios.post('/api/git/push')
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to push'
        throw error
      }
    },

    async pull(): Promise<void> {
      try {
        await axios.post('/api/git/pull')
        // Refresh status after pull
        await this.getStatus()
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to pull'
        throw error
      }
    },

    async getDiff(filePath?: string): Promise<string> {
      try {
        const params = filePath ? { file: filePath } : {}
        const response = await axios.get('/api/git/diff', { params })
        return response.data.diff
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to get diff'
        throw error
      }
    },

    async stage(filePath: string): Promise<void> {
      try {
        await axios.post('/api/git/stage', { file: filePath })
        await this.getStatus()
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to stage file'
        throw error
      }
    },

    async unstage(filePath: string): Promise<void> {
      try {
        await axios.post('/api/git/unstage', { file: filePath })
        await this.getStatus()
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to unstage file'
        throw error
      }
    }
  }
})