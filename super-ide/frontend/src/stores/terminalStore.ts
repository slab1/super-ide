import { defineStore } from 'pinia'
import type { TerminalSession, TerminalOutput } from '../types'

export const useTerminalStore = defineStore('terminal', {
  state: () => ({
    sessions: [] as TerminalSession[],
    activeSessionId: '' as string,
    isConnected: false as boolean,
    ws: null as WebSocket | null,
    outputHistory: new Map<string, TerminalOutput[]>()
  }),

  actions: {
    connect() {
      if (this.isConnected) return

      try {
        this.ws = new WebSocket('ws://localhost:3001/ws/terminal')
        
        this.ws.onopen = () => {
          this.isConnected = true
          console.log('Terminal WebSocket connected')
        }

        this.ws.onmessage = (event) => {
          try {
            const data = JSON.parse(event.data)
            this.handleMessage(data)
          } catch (error) {
            console.error('Failed to parse WebSocket message:', error)
          }
        }

        this.ws.onclose = () => {
          this.isConnected = false
          this.ws = null
          console.log('Terminal WebSocket disconnected')
        }

        this.ws.onerror = (error) => {
          console.error('Terminal WebSocket error:', error)
        }
      } catch (error) {
        console.error('Failed to connect to terminal WebSocket:', error)
      }
    },

    disconnect() {
      if (this.ws) {
        this.ws.close()
        this.ws = null
      }
      this.isConnected = false
    },

    handleMessage(data: any) {
      switch (data.type) {
        case 'terminal_output':
          this.handleOutput(data.sessionId, data.output)
          break
        case 'session_created':
          this.handleSessionCreated(data.session)
          break
        case 'session_closed':
          this.handleSessionClosed(data.sessionId)
          break
        case 'error':
          console.error('Terminal error:', data.message)
          break
        default:
          console.log('Unknown terminal message:', data)
      }
    },

    handleOutput(sessionId: string, output: TerminalOutput) {
      if (!this.outputHistory.has(sessionId)) {
        this.outputHistory.set(sessionId, [])
      }
      
      const history = this.outputHistory.get(sessionId)!
      history.push(output)
      
      // Keep only last 1000 lines to prevent memory issues
      if (history.length > 1000) {
        history.splice(0, history.length - 1000)
      }
    },

    handleSessionCreated(session: TerminalSession) {
      this.sessions.push(session)
      if (!this.activeSessionId) {
        this.activeSessionId = session.id
      }
    },

    handleSessionClosed(sessionId: string) {
      const index = this.sessions.findIndex(s => s.id === sessionId)
      if (index !== -1) {
        this.sessions.splice(index, 1)
      }
      
      this.outputHistory.delete(sessionId)
      
      if (this.activeSessionId === sessionId && this.sessions.length > 0) {
        this.activeSessionId = this.sessions[0].id
      }
    },

    async createSession(): Promise<TerminalSession> {
      return new Promise((resolve, reject) => {
        if (!this.ws || !this.isConnected) {
          reject(new Error('WebSocket not connected'))
          return
        }

        const sessionId = Date.now().toString()
        
        this.ws.send(JSON.stringify({
          type: 'create_session',
          sessionId,
          name: `Terminal ${this.sessions.length + 1}`
        }))

        // Wait for session creation response
        const timeout = setTimeout(() => {
          reject(new Error('Session creation timeout'))
        }, 5000)

        const checkSession = () => {
          const session = this.sessions.find(s => s.id === sessionId)
          if (session) {
            clearTimeout(timeout)
            resolve(session)
          } else {
            setTimeout(checkSession, 100)
          }
        }

        checkSession()
      })
    },

    async executeCommand(sessionId: string, command: string): Promise<void> {
      return new Promise((resolve, reject) => {
        if (!this.ws || !this.isConnected) {
          reject(new Error('WebSocket not connected'))
          return
        }

        this.ws.send(JSON.stringify({
          type: 'execute_command',
          sessionId,
          command
        }))

        resolve()
      })
    },

    async switchToSession(sessionId: string): Promise<void> {
      if (!this.ws || !this.isConnected) {
        throw new Error('WebSocket not connected')
      }

      this.activeSessionId = sessionId
      
      this.ws.send(JSON.stringify({
        type: 'switch_session',
        sessionId
      }))
    },

    async killSession(sessionId: string): Promise<void> {
      return new Promise((resolve, reject) => {
        if (!this.ws || !this.isConnected) {
          reject(new Error('WebSocket not connected'))
          return
        }

        this.ws.send(JSON.stringify({
          type: 'kill_session',
          sessionId
        }))

        resolve()
      })
    },

    getSessionOutput(sessionId: string): TerminalOutput[] {
      return this.outputHistory.get(sessionId) || []
    },

    onOutput(callback: (output: TerminalOutput) => void) {
      // Set up output callback for the active session
      const originalHandleOutput = this.handleOutput.bind(this)
      this.handleOutput = (sessionId: string, output: TerminalOutput) => {
        originalHandleOutput(sessionId, output)
        if (sessionId === this.activeSessionId) {
          callback(output)
        }
      }
    },

    clearSessionOutput(sessionId: string) {
      this.outputHistory.set(sessionId, [])
    },

    getActiveSession(): TerminalSession | null {
      return this.sessions.find(s => s.id === this.activeSessionId) || null
    },

    getAllSessions(): TerminalSession[] {
      return [...this.sessions]
    }
  }
})