import { defineStore } from 'pinia'
import axios from 'axios'
import type { CodeCompletion, AIMessage } from '../types'

interface AIContext {
  filePath?: string
  fileContent?: string
  language?: string
}

interface AIResponse {
  content: string
  suggestions?: CodeCompletion[]
}

export const useAIStore = defineStore('ai', {
  state: () => ({
    messages: [] as AIMessage[],
    isLoading: false,
    error: null as string | null,
    settings: {
      provider: 'openai',
      temperature: 0.7,
      maxTokens: 2048,
      model: 'gpt-4'
    }
  }),

  actions: {
    async sendMessage(content: string, context?: AIContext): Promise<AIResponse> {
      try {
        this.isLoading = true
        this.error = null

        const response = await axios.post('/api/ai/chat', {
          message: content,
          context,
          settings: this.settings
        })

        return response.data
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to get AI response'
        throw error
      } finally {
        this.isLoading = false
      }
    },

    async getCompletions(filePath: string, content: string, position: any): Promise<CodeCompletion[]> {
      try {
        const response = await axios.post('/api/ai/completions', {
          filePath,
          content,
          position,
          settings: this.settings
        })

        return response.data.suggestions || []
      } catch (error) {
        console.error('Failed to get completions:', error)
        return []
      }
    },

    async analyzeCode(code: string, language: string): Promise<any> {
      try {
        const response = await axios.post('/api/ai/analyze', {
          code,
          language
        })

        return response.data
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to analyze code'
        throw error
      }
    },

    async explainCode(code: string, language: string): Promise<string> {
      try {
        const response = await axios.post('/api/ai/explain', {
          code,
          language
        })

        return response.data.explanation
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to explain code'
        throw error
      }
    },

    async generateTests(code: string, language: string): Promise<string> {
      try {
        const response = await axios.post('/api/ai/generate-tests', {
          code,
          language
        })

        return response.data.tests
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to generate tests'
        throw error
      }
    },

    async optimizeCode(code: string, language: string): Promise<{ optimized: string, explanation: string }> {
      try {
        const response = await axios.post('/api/ai/optimize', {
          code,
          language
        })

        return {
          optimized: response.data.optimized,
          explanation: response.data.explanation
        }
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to optimize code'
        throw error
      }
    },

    async debugCode(code: string, language: string, error?: string): Promise<string> {
      try {
        const response = await axios.post('/api/ai/debug', {
          code,
          language,
          error
        })

        return response.data.suggestions
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to debug code'
        throw error
      }
    },

    async generateDocumentation(code: string, language: string): Promise<string> {
      try {
        const response = await axios.post('/api/ai/document', {
          code,
          language
        })

        return response.data.documentation
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to generate documentation'
        throw error
      }
    },

    async translateCode(code: string, fromLanguage: string, toLanguage: string): Promise<string> {
      try {
        const response = await axios.post('/api/ai/translate', {
          code,
          fromLanguage,
          toLanguage
        })

        return response.data.translatedCode
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to translate code'
        throw error
      }
    },

    async predictBugs(code: string, language: string): Promise<any[]> {
      try {
        const response = await axios.post('/api/ai/predict-bugs', {
          code,
          language
        })

        return response.data.predictions || []
      } catch (error) {
        console.error('Failed to predict bugs:', error)
        return []
      }
    },

    async detectCodeSmells(code: string, language: string): Promise<any[]> {
      try {
        const response = await axios.post('/api/ai/code-smells', {
          code,
          language
        })

        return response.data.smells || []
      } catch (error) {
        console.error('Failed to detect code smells:', error)
        return []
      }
    },

    // Advanced AI Features
    async getSmartCompletions(filePath: string, content: string, position: any): Promise<CodeCompletion[]> {
      try {
        const response = await axios.post('/api/ai/smart-completions', {
          filePath,
          content,
          position,
          settings: this.settings
        })

        return response.data.suggestions || []
      } catch (error) {
        console.error('Failed to get smart completions:', error)
        return []
      }
    },

    async getCodeReview(code: string, language: string): Promise<any> {
      try {
        const response = await axios.post('/api/ai/code-review', {
          code,
          language
        })

        return response.data
      } catch (error) {
        console.error('Failed to get code review:', error)
        throw error
      }
    },

    async getDebuggingAssistance(code: string, language: string, issue?: string): Promise<any> {
      try {
        const response = await axios.post('/api/ai/debug-assistance', {
          code,
          language,
          issue
        })

        return response.data
      } catch (error) {
        console.error('Failed to get debugging assistance:', error)
        throw error
      }
    },

    async generateProject(config: any): Promise<any> {
      try {
        const response = await axios.post('/api/ai/generate-project', config)
        return response.data
      } catch (error) {
        console.error('Failed to generate project:', error)
        throw error
      }
    },

    async getContextAwareHelp(query: string, context: any): Promise<any> {
      try {
        const response = await axios.post('/api/ai/context-help', {
          query,
          context
        })

        return response.data
      } catch (error) {
        console.error('Failed to get context-aware help:', error)
        throw error
      }
    },

    async learnFromFeedback(pattern: string, wasPositive: boolean): Promise<void> {
      try {
        await axios.post('/api/ai/learning/feedback', {
          pattern,
          positive: wasPositive,
          timestamp: new Date().toISOString()
        })
      } catch (error) {
        console.error('Failed to send learning feedback:', error)
      }
    },

    async optimizeCodeAdvanced(code: string, language: string, optimizationGoals: string[]): Promise<any> {
      try {
        const response = await axios.post('/api/ai/optimize-advanced', {
          code,
          language,
          goals: optimizationGoals
        })

        return response.data
      } catch (error) {
        console.error('Failed to optimize code:', error)
        throw error
      }
    },

    async getRefactoringSuggestions(code: string, language: string): Promise<any[]> {
      try {
        const response = await axios.post('/api/ai/refactoring-suggestions', {
          code,
          language
        })

        return response.data.suggestions || []
      } catch (error) {
        console.error('Failed to get refactoring suggestions:', error)
        return []
      }
    },

    async applyRefactoring(code: string, suggestion: any): Promise<string> {
      try {
        const response = await axios.post('/api/ai/apply-refactoring', {
          code,
          suggestion
        })

        return response.data.refactoredCode
      } catch (error) {
        console.error('Failed to apply refactoring:', error)
        throw error
      }
    },

    async generateTestsAdvanced(code: string, language: string, testTypes: string[]): Promise<any> {
      try {
        const response = await axios.post('/api/ai/generate-tests-advanced', {
          code,
          language,
          testTypes
        })

        return response.data
      } catch (error) {
        console.error('Failed to generate tests:', error)
        throw error
      }
    },

    async getPerformanceAnalysis(code: string, language: string): Promise<any> {
      try {
        const response = await axios.post('/api/ai/performance-analysis', {
          code,
          language
        })

        return response.data
      } catch (error) {
        console.error('Failed to get performance analysis:', error)
        throw error
      }
    },

    async getSecurityAnalysis(code: string, language: string): Promise<any> {
      try {
        const response = await axios.post('/api/ai/security-analysis', {
          code,
          language
        })

        return response.data
      } catch (error) {
        console.error('Failed to get security analysis:', error)
        throw error
      }
    },

    async translateBetweenLanguages(code: string, fromLanguage: string, toLanguage: string, preserveComments: boolean = true): Promise<any> {
      try {
        const response = await axios.post('/api/ai/translate-languages', {
          code,
          fromLanguage,
          toLanguage,
          preserveComments
        })

        return response.data
      } catch (error) {
        console.error('Failed to translate code:', error)
        throw error
      }
    },

    async getCodeMetrics(code: string, language: string): Promise<any> {
      try {
        const response = await axios.post('/api/ai/code-metrics', {
          code,
          language
        })

        return response.data
      } catch (error) {
        console.error('Failed to get code metrics:', error)
        throw error
      }
    },

    updateSettings(newSettings: Partial<typeof this.settings>) {
      this.settings = { ...this.settings, ...newSettings }
      // TODO: Save to local storage or server
    },

    addMessage(message: AIMessage) {
      this.messages.push(message)
    },

    clearMessages() {
      this.messages = []
    },

    getMessages(): AIMessage[] {
      return this.messages
    }
  }
})