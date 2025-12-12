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

// Phase 4: Enhanced Code Intelligence Types
interface AdvancedAnalysisResult {
  issues: CodeIssue[]
  suggestions: string[]
  complexity_score: number
  bug_predictions: BugPrediction[]
  code_smells: CodeSmell[]
  security_vulnerabilities: SecurityVulnerability[]
  performance_insights: string[]
  maintainability_score: number
}

interface CodeIssue {
  id: string
  severity: 'Info' | 'Warning' | 'Error' | 'Critical'
  message: string
  line: number
  column: number
  file_path?: string
  rule_id?: string
  fix_suggestion?: string
  documentation_url?: string
}

interface BugPrediction {
  line: number
  column: number
  bug_type: string
  confidence: number
  description: string
  fix_suggestion: string
  severity: 'Info' | 'Warning' | 'Error' | 'Critical'
}

interface CodeSmell {
  id: string
  name: string
  description: string
  line: number
  column: number
  severity: 'Info' | 'Warning' | 'Error' | 'Critical'
  refactoring_suggestion: string
}

interface SecurityVulnerability {
  id: string
  cwe_id?: string
  title: string
  description: string
  severity: 'Info' | 'Warning' | 'Error' | 'Critical'
  line: number
  column: number
  recommendation: string
  cve_references: string[]
}

interface CodeExplanation {
  summary: string
  explanation: string
  key_concepts: string[]
  complexity_analysis: string
  suggestions: string[]
}

interface DebugSession {
  session_id: string
  file_path: string
  breakpoints: DebugBreakpoint[]
  current_line?: number
  variables: DebugVariable[]
  call_stack: StackFrame[]
  is_active: boolean
}

interface DebugBreakpoint {
  id: string
  line: number
  column: number
  enabled: boolean
  condition?: string
  hit_count: number
}

interface DebugVariable {
  name: string
  value: string
  type_name: string
  scope: 'Local' | 'Global' | 'Parameter' | 'Field'
  is_changed: boolean
}

interface StackFrame {
  function_name: string
  file_path: string
  line: number
  column: number
  local_variables: DebugVariable[]
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
    },
    
    // Phase 4: Enhanced Code Intelligence State
    currentAnalysis: null as AdvancedAnalysisResult | null,
    bugPredictions: [] as BugPrediction[],
    securityVulnerabilities: [] as SecurityVulnerability[],
    codeExplanation: null as CodeExplanation | null,
    generatedTests: '' as string,
    improvementSuggestions: [] as string[],
    
    // Debug session
    debugSession: null as DebugSession | null,
    debugVariables: [] as DebugVariable[],
    
    // Enhanced AI settings
    autoAnalysis: true,
    showBugPredictions: true,
    showSecurityAnalysis: true,
    showPerformanceAnalysis: false,
    isAnalyzing: false,
    isDebugging: false,
  }),

  getters: {
    hasIssues: (state) => {
      if (!state.currentAnalysis) return false
      return state.currentAnalysis.issues.length > 0
    },
    
    criticalIssues: (state) => {
      if (!state.currentAnalysis) return []
      return state.currentAnalysis.issues.filter(issue => issue.severity === 'Critical')
    },
    
    warnings: (state) => {
      if (!state.currentAnalysis) return []
      return state.currentAnalysis.issues.filter(issue => issue.severity === 'Warning')
    },
    
    highConfidenceBugs: (state) => {
      return state.bugPredictions.filter(bug => bug.confidence > 0.7)
    },
    
    criticalVulnerabilities: (state) => {
      return state.securityVulnerabilities.filter(vuln => vuln.severity === 'Critical')
    },
    
    overallScore: (state) => {
      if (!state.currentAnalysis) return 100
      const complexityPenalty = state.currentAnalysis.complexity_score * 30
      const issuePenalty = (state.currentAnalysis.issues.length * 5)
      const bugPenalty = (state.bugPredictions.length * 10)
      const vulnPenalty = (state.securityVulnerabilities.length * 15)
      
      return Math.max(0, 100 - complexityPenalty - issuePenalty - bugPenalty - vulnPenalty)
    },
  },

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

    updateAISettings(newSettings: Partial<typeof this.settings>) {
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
    },

    // Phase 4: Enhanced Code Intelligence Actions
    
    // Advanced Code Analysis
    async analyzeCodeAdvanced(code: string, language: string, filePath?: string) {
      try {
        this.isAnalyzing = true
        this.error = null
        
        const response = await axios.post('/api/ai/advanced-analysis', {
          code,
          language,
          file_path: filePath,
          include_bug_prediction: this.showBugPredictions,
          include_security_analysis: this.showSecurityAnalysis,
          include_performance_analysis: this.showPerformanceAnalysis,
        })
        
        if (!response.data.success) {
          throw new Error(response.data.error || 'Analysis failed')
        }
        
        this.currentAnalysis = response.data.data
        
        // Also get specific analyses if enabled
        if (this.showBugPredictions) {
          await this.predictBugs(code, language)
        }
        
        if (this.showSecurityAnalysis) {
          await this.analyzeSecurity(code, language)
        }
        
        return this.currentAnalysis
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Code analysis failed'
        throw error
      } finally {
        this.isAnalyzing = false
      }
    },

    // Bug Prediction
    async predictBugsEnhanced(code: string, language: string) {
      try {
        const response = await axios.post('/api/ai/bug-prediction', {
          code,
          language,
        })
        
        if (!response.data.success) {
          throw new Error(response.data.error || 'Bug prediction failed')
        }
        
        this.bugPredictions = response.data.data || []
        return this.bugPredictions
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Bug prediction failed'
        throw error
      }
    },

    // Security Analysis
    async analyzeSecurity(code: string, language: string) {
      try {
        const response = await axios.post('/api/ai/security-vulnerabilities', {
          code,
          language,
        })
        
        if (!response.data.success) {
          throw new Error(response.data.error || 'Security analysis failed')
        }
        
        this.securityVulnerabilities = response.data.data || []
        return this.securityVulnerabilities
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Security analysis failed'
        throw error
      }
    },

    // Code Explanation
    async explainCodeEnhanced(code: string, language: string, context?: string, level: 'basic' | 'detailed' | 'expert' = 'detailed') {
      try {
        this.isAnalyzing = true
        this.error = null
        
        const response = await axios.post('/api/ai/code-explanation', {
          code,
          language,
          context,
          explanation_level: level,
        })
        
        if (!response.data.success) {
          throw new Error(response.data.error || 'Code explanation failed')
        }
        
        this.codeExplanation = response.data.data
        return this.codeExplanation
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Code explanation failed'
        throw error
      } finally {
        this.isAnalyzing = false
      }
    },

    // Generate Unit Tests
    async generateTestsEnhanced(code: string, language: string, testFramework?: string) {
      try {
        this.isAnalyzing = true
        this.error = null
        
        const response = await axios.post('/api/ai/generate-tests', {
          code,
          language,
          test_framework: testFramework,
        })
        
        if (!response.data.success) {
          throw new Error(response.data.error || 'Test generation failed')
        }
        
        this.generatedTests = response.data.data || ''
        return this.generatedTests
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Test generation failed'
        throw error
      } finally {
        this.isAnalyzing = false
      }
    },

    // Get Improvement Suggestions
    async getImprovements(code: string, language: string, improvementType?: string) {
      try {
        const response = await axios.post('/api/ai/code-improvements', {
          code,
          language,
          improvement_type: improvementType,
        })
        
        if (!response.data.success) {
          throw new Error(response.data.error || 'Improvement suggestions failed')
        }
        
        this.improvementSuggestions = response.data.data || []
        return this.improvementSuggestions
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Improvement suggestions failed'
        throw error
      }
    },

    // Debug Session Management
    async startDebugSession(filePath: string, language: string, initialBreakpoints: number[] = []) {
      try {
        this.isDebugging = true
        this.error = null
        
        const response = await axios.post('/api/ai/debug-session/start', {
          file_path: filePath,
          language,
          initial_breakpoints: initialBreakpoints,
        })
        
        if (!response.data.success) {
          throw new Error(response.data.error || 'Failed to start debug session')
        }
        
        this.debugSession = response.data.data
        return this.debugSession
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to start debug session'
        throw error
      } finally {
        this.isDebugging = false
      }
    },

    async getDebugSession(sessionId: string) {
      try {
        const response = await axios.get(`/api/ai/debug-session/${sessionId}`)
        
        if (!response.data.success) {
          throw new Error(response.data.error || 'Failed to get debug session')
        }
        
        this.debugSession = response.data.data
        return this.debugSession
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to get debug session'
        throw error
      }
    },

    async setBreakpoints(sessionId: string, breakpoints: { line: number; condition?: string; enabled?: boolean }[]) {
      try {
        const response = await axios.post(`/api/ai/debug-session/${sessionId}/breakpoints`, {
          session_id: sessionId,
          breakpoints,
        })
        
        if (!response.data.success) {
          throw new Error(response.data.error || 'Failed to set breakpoints')
        }
        
        if (this.debugSession) {
          this.debugSession.breakpoints = response.data.data || []
        }
        
        return response.data.data
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to set breakpoints'
        throw error
      }
    },

    async debugStep(sessionId: string, action: 'step_over' | 'step_into' | 'step_out' | 'continue') {
      try {
        const response = await axios.post('/api/ai/debug-session/step', {
          session_id: sessionId,
          action,
        })
        
        if (!response.data.success) {
          throw new Error(response.data.error || 'Debug step failed')
        }
        
        // Refresh debug session state
        await this.getDebugSession(sessionId)
        await this.getDebugVariables(sessionId)
        
        return response.data.data
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Debug step failed'
        throw error
      }
    },

    async getDebugVariables(sessionId: string) {
      try {
        const response = await axios.get(`/api/ai/debug-session/${sessionId}/variables`)
        
        if (!response.data.success) {
          throw new Error(response.data.error || 'Failed to get debug variables')
        }
        
        this.debugVariables = response.data.data || []
        return this.debugVariables
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to get debug variables'
        throw error
      }
    },

    async stopDebugSession(sessionId: string) {
      try {
        const response = await axios.post('/api/ai/debug-session/stop', {
          session_id: sessionId,
        })
        
        if (!response.data.success) {
          throw new Error(response.data.error || 'Failed to stop debug session')
        }
        
        this.debugSession = null
        this.debugVariables = []
        this.isDebugging = false
        
        return response.data.data
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Failed to stop debug session'
        throw error
      }
    },

    // Utility methods
    clearAnalysis() {
      this.currentAnalysis = null
      this.bugPredictions = []
      this.securityVulnerabilities = []
      this.codeExplanation = null
      this.generatedTests = ''
      this.improvementSuggestions = []
    },

    clearError() {
      this.error = null
    },

    updateSettings(newSettings: Partial<{
      autoAnalysis: boolean
      showBugPredictions: boolean
      showSecurityAnalysis: boolean
      showPerformanceAnalysis: boolean
    }>) {
      Object.assign(this, newSettings)
    },

    // Auto-analysis trigger
    async triggerAutoAnalysis(code: string, language: string, filePath?: string) {
      if (this.autoAnalysis && !this.isAnalyzing) {
        try {
          await this.analyzeCodeAdvanced(code, language, filePath)
        } catch (error) {
          console.warn('Auto-analysis failed:', error)
        }
      }
    },
  }
})