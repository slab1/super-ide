import { defineStore } from 'pinia'
import axios from 'axios'
import type { GitStatus, GitBranch, GitCommit, ApiResponse } from '../types'

export const useGitStore = defineStore('git', {
  state: () => ({
    status: null as GitStatus | null,
    branches: [] as GitBranch[],
    currentBranch: '' as string,
    commits: [] as GitCommit[],
    isLoading: false,
    error: null as string | null,
    isRepository: false
  }),

  actions: {
    async getStatus(): Promise<GitStatus | null> {
      try {
        this.isLoading = true
        const response = await axios.get<ApiResponse<GitStatus>>('/api/git/status')
        
        if (!response.data.success) {
          // If not a repository, that's OK - just set isRepository to false
          if (response.data.error?.includes('Not a git repository')) {
            this.isRepository = false
            this.status = null
            return null
          }
          throw new Error(response.data.error || 'Failed to get git status')
        }
        
        this.isRepository = true
        this.status = response.data.data
        return this.status
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to get git status'
        if (error instanceof Error && error.message.includes('Not a git repository')) {
          this.isRepository = false
          this.status = null
        }
        throw error
      } finally {
        this.isLoading = false
      }
    },

    async getBranches(): Promise<GitBranch[]> {
      try {
        const response = await axios.get<ApiResponse<GitBranch[]>>('/api/git/branches')
        
        if (!response.data.success) {
          if (response.data.error?.includes('Not a git repository')) {
            this.isRepository = false
            this.branches = []
            return []
          }
          throw new Error(response.data.error || 'Failed to get branches')
        }
        
        this.isRepository = true
        this.branches = response.data.data || []
        
        // Set current branch
        const current = this.branches.find(b => b.is_current)
        if (current) {
          this.currentBranch = current.name
        }
        
        return this.branches
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to get branches'
        throw error
      }
    },

    async commit(message: string): Promise<string> {
      try {
        this.isLoading = true
        const response = await axios.post<ApiResponse<string>>('/api/git/commit', { 
          message 
        })
        
        if (!response.data.success) {
          throw new Error(response.data.error || 'Failed to commit')
        }
        
        // Refresh status after commit
        await this.getStatus()
        return response.data.data || 'Commit successful'
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to commit'
        throw error
      } finally {
        this.isLoading = false
      }
    },

    async createBranch(name: string): Promise<void> {
      try {
        // This would require a new API endpoint
        // For now, we'll implement it as a placeholder
        this.error = 'Branch creation not yet implemented - requires new API endpoint'
        throw new Error('Branch creation not yet implemented')
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to create branch'
        throw error
      }
    },

    async switchBranch(name: string): Promise<void> {
      try {
        // This would require a new API endpoint
        // For now, we'll implement it as a placeholder
        this.error = 'Branch switching not yet implemented - requires new API endpoint'
        throw new Error('Branch switching not yet implemented')
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to switch branch'
        throw error
      }
    },

    async push(): Promise<void> {
      try {
        // This would require a new API endpoint
        // For now, we'll implement it as a placeholder
        this.error = 'Git push not yet implemented - requires new API endpoint'
        throw new Error('Git push not yet implemented')
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to push'
        throw error
      }
    },

    async pull(): Promise<void> {
      try {
        // This would require a new API endpoint
        // For now, we'll implement it as a placeholder
        this.error = 'Git pull not yet implemented - requires new API endpoint'
        throw new Error('Git pull not yet implemented')
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to pull'
        throw error
      }
    },

    async getDiff(filePath?: string): Promise<string> {
      try {
        // This would require a new API endpoint
        // For now, we'll implement it as a placeholder
        this.error = 'Git diff not yet implemented - requires new API endpoint'
        throw new Error('Git diff not yet implemented')
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to get diff'
        throw error
      }
    },

    async stage(filePath: string): Promise<void> {
      try {
        // This would require a new API endpoint
        // For now, we'll implement it as a placeholder
        this.error = 'Git staging not yet implemented - requires new API endpoint'
        throw new Error('Git staging not yet implemented')
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to stage file'
        throw error
      }
    },

    async unstage(filePath: string): Promise<void> {
      try {
        // This would require a new API endpoint
        // For now, we'll implement it as a placeholder
        this.error = 'Git unstaging not yet implemented - requires new API endpoint'
        throw new Error('Git unstaging not yet implemented')
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to unstage file'
        throw error
      }
    },

    // Additional git operations
    async deleteBranch(branchName: string): Promise<void> {
      try {
        this.isLoading = true
        const response = await axios.delete<ApiResponse<string>>('/api/git/delete_branch', {
          data: { branch_name: branchName }
        })
        
        if (!response.data.success) {
          throw new Error(response.data.error || 'Failed to delete branch')
        }
        
        // Refresh branches after deletion
        await this.getBranches()
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to delete branch'
        throw error
      } finally {
        this.isLoading = false
      }
    },

    async getLog(): Promise<GitCommit[]> {
      try {
        const response = await axios.get<ApiResponse<GitCommit[]>>('/api/git/log')
        
        if (!response.data.success) {
          throw new Error(response.data.error || 'Failed to get commit log')
        }
        
        this.commits = response.data.data || []
        return this.commits
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to get commit log'
        throw error
      }
    },

    // Stash operations
    async stash(message?: string): Promise<void> {
      try {
        this.isLoading = true
        const response = await axios.post<ApiResponse<string>>('/api/git/stash', { 
          message 
        })
        
        if (!response.data.success) {
          throw new Error(response.data.error || 'Failed to stash changes')
        }
        
        // Refresh status after stash
        await this.getStatus()
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to stash changes'
        throw error
      } finally {
        this.isLoading = false
      }
    },

    async stashPop(index?: number): Promise<void> {
      try {
        this.isLoading = true
        const response = await axios.post<ApiResponse<string>>('/api/git/stash_pop', { 
          index 
        })
        
        if (!response.data.success) {
          throw new Error(response.data.error || 'Failed to pop stash')
        }
        
        // Refresh status after pop
        await this.getStatus()
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to pop stash'
        throw error
      } finally {
        this.isLoading = false
      }
    },

    async stashApply(index?: number): Promise<void> {
      try {
        this.isLoading = true
        const response = await axios.post<ApiResponse<string>>('/api/git/stash_apply', { 
          index 
        })
        
        if (!response.data.success) {
          throw new Error(response.data.error || 'Failed to apply stash')
        }
        
        // Refresh status after apply
        await this.getStatus()
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to apply stash'
        throw error
      } finally {
        this.isLoading = false
      }
    },

    async stashDrop(index?: number): Promise<void> {
      try {
        this.isLoading = true
        const response = await axios.post<ApiResponse<string>>('/api/git/stash_drop', { 
          index 
        })
        
        if (!response.data.success) {
          throw new Error(response.data.error || 'Failed to drop stash')
        }
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to drop stash'
        throw error
      } finally {
        this.isLoading = false
      }
    },

    async merge(branchName: string, noFF?: boolean): Promise<void> {
      try {
        this.isLoading = true
        const response = await axios.post<ApiResponse<string>>('/api/git/merge', { 
          branch_name: branchName,
          no_ff: noFF 
        })
        
        if (!response.data.success) {
          throw new Error(response.data.error || 'Failed to merge branch')
        }
        
        // Refresh status after merge
        await this.getStatus()
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to merge branch'
        throw error
      } finally {
        this.isLoading = false
      }
    },

    // Utility methods
    getStagedFiles() {
      return this.status?.staged_files || []
    },

    getUnstagedFiles() {
      return this.status?.unstaged_files || []
    },

    getUntrackedFiles() {
      return this.status?.untracked_files || []
    },

    getTotalChanges(): number {
      if (!this.status) return 0
      return this.status.staged_files.length + this.status.unstaged_files.length + this.status.untracked_files.length
    },

    isClean(): boolean {
      return this.getTotalChanges() === 0
    },

    getRepositoryStatus(): 'clean' | 'modified' | 'untracked' | 'not-a-repo' {
      if (!this.isRepository) return 'not-a-repo'
      if (this.isClean()) return 'clean'
      return 'modified'
    },

    async refresh(): Promise<void> {
      await Promise.all([
        this.getStatus(),
        this.getBranches()
      ])
    }
  }
})