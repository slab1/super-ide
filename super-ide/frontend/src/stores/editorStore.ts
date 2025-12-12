import { defineStore } from 'pinia'
import * as monaco from 'monaco-editor'

export const useEditorStore = defineStore('editor', {
  state: () => ({
    editor: null as monaco.editor.IStandaloneCodeEditor | null,
    currentFile: null as any,
    cursorPosition: { lineNumber: 1, column: 1 },
    selections: [] as monaco.ISelection[],
    isReadOnly: false,
    language: 'plaintext',
    theme: 'vs-dark',
    fontSize: 14,
    wordWrap: true,
    minimap: true,
    lineNumbers: true,
    autoSave: true,
    formatOnSave: true,
    tabSize: 2,
    insertSpaces: true,
    autoClosingBrackets: 'always' as const,
    autoClosingQuotes: 'always' as const,
    bracketPairColorization: true,
    guides: {
      bracketPairs: true,
      indentation: true
    }
  }),

  getters: {
    getEditor: (state) => state.editor,
    
    getCurrentPosition: (state) => state.cursorPosition,
    
    getCurrentSelections: (state) => state.selections,
    
    isEditorReady: (state) => !!state.editor,
    
    getEditorState: (state) => ({
      cursorPosition: state.cursorPosition,
      selections: state.selections,
      currentFile: state.currentFile,
      language: state.language
    })
  },

  actions: {
    setEditor(editor: monaco.editor.IStandaloneCodeEditor) {
      this.editor = editor
      
      // Set up editor event listeners
      editor.onDidChangeCursorPosition((e) => {
        this.cursorPosition = {
          lineNumber: e.position.lineNumber,
          column: e.position.column
        }
      })

      editor.onDidChangeCursorSelection((e) => {
        this.selections = e.selections
      })

      editor.onDidChangeModelContent(() => {
        // Auto-save if enabled
        if (this.autoSave) {
          this.autoSaveCurrentFile()
        }
      })

      // Set up keyboard shortcuts
      this.setupKeyboardShortcuts()
    },

    async insertText(text: string, position?: monaco.Position) {
      if (!this.editor) return

      try {
        const editor = this.editor
        
        if (position) {
          // Insert at specific position
          const range = new monaco.Range(
            position.lineNumber,
            position.column,
            position.lineNumber,
            position.column
          )
          editor.executeEdits('snippet-insert', [{
            range,
            text,
            forceMoveMarkers: true
          }])
        } else {
          // Insert at current cursor position
          const selection = editor.getSelection()
          if (selection) {
            editor.executeEdits('snippet-insert', [{
              range: selection,
              text,
              forceMoveMarkers: true
            }])
          }
        }
        
        // Focus the editor
        editor.focus()
        
      } catch (error) {
        console.error('Failed to insert text:', error)
        throw error
      }
    },

    async replaceSelection(text: string) {
      if (!this.editor) return

      try {
        const editor = this.editor
        const selection = editor.getSelection()
        
        if (selection) {
          editor.executeEdits('snippet-replace', [{
            range: selection,
            text,
            forceMoveMarkers: true
          }])
          
          editor.focus()
        }
      } catch (error) {
        console.error('Failed to replace selection:', error)
        throw error
      }
    },

    async insertSnippet(snippet: string, variables?: Record<string, string>) {
      if (!this.editor) return

      try {
        let processedSnippet = snippet

        // Replace variables if provided
        if (variables) {
          Object.entries(variables).forEach(([key, value]) => {
            const regex = new RegExp(`\\$\\{${key}\\}`, 'g')
            processedSnippet = processedSnippet.replace(regex, value)
          })
        }

        // Handle tab stops (${1:default})
        const tabStopRegex = /\$\{(\d+)(?::([^}]*))?\}/g
        const matches = [...processedSnippet.matchAll(tabStopRegex)]
        
        if (matches.length > 0) {
          // Insert snippet with tab stops
          await this.insertTextWithTabStops(processedSnippet, matches)
        } else {
          // Simple insertion
          await this.insertText(processedSnippet)
        }
        
      } catch (error) {
        console.error('Failed to insert snippet:', error)
        throw error
      }
    },

    async insertTextWithTabStops(snippet: string, tabStops: RegExpMatchArray[]) {
      if (!this.editor) return

      const editor = this.editor
      
      // Insert the snippet first
      const selection = editor.getSelection()
      if (!selection) return

      editor.executeEdits('snippet-insert', [{
        range: selection,
        text: snippet.replace(/\$\{\d+(:[^}]*)?\}/g, '$2'),
        forceMoveMarkers: true
      }])

      // Set cursor to first tab stop
      const firstTabStop = tabStops[0]
      if (firstTabStop && firstTabStop[2]) {
        const lineNumber = selection.startLineNumber
        const column = selection.startColumn + (firstTabStop.index || 0)
        
        const position = new monaco.Position(lineNumber, column)
        editor.setPosition(position)
        editor.revealLineInCenter(lineNumber)
      }

      editor.focus()
    },

    getSelectedText(): string {
      if (!this.editor) return ''
      
      const selection = this.editor.getSelection()
      if (!selection) return ''
      
      return this.editor.getModel()?.getValueInRange(selection) || ''
    },

    getLineText(lineNumber?: number): string {
      if (!this.editor) return ''
      
      const model = this.editor.getModel()
      if (!model) return ''
      
      const line = lineNumber || this.cursorPosition.lineNumber
      return model.getLineContent(line)
    },

    getWordAtPosition(position?: monaco.Position): string {
      if (!this.editor) return ''
      
      const model = this.editor.getModel()
      if (!model) return ''
      
      const pos = position || new monaco.Position(
        this.cursorPosition.lineNumber,
        this.cursorPosition.column
      )
      
      const word = model.getWordAtPosition(pos)
      return word?.word || ''
    },

    async formatDocument() {
      if (!this.editor) return

      try {
        await this.editor.getAction('editor.action.formatDocument')?.run()
      } catch (error) {
        console.error('Failed to format document:', error)
      }
    },

    async goToDefinition() {
      if (!this.editor) return

      try {
        await this.editor.getAction('editor.action.goToDefinition')?.run()
      } catch (error) {
        console.error('Failed to go to definition:', error)
      }
    },

    async findReferences() {
      if (!this.editor) return

      try {
        await this.editor.getAction('editor.action.referenceSearch.trigger')?.run()
      } catch (error) {
        console.error('Failed to find references:', error)
      }
    },

    async renameSymbol() {
      if (!this.editor) return

      try {
        await this.editor.getAction('editor.action.rename')?.run()
      } catch (error) {
        console.error('Failed to rename symbol:', error)
      }
    },

    async toggleComment() {
      if (!this.editor) return

      try {
        await this.editor.getAction('editor.action.commentLine')?.run()
      } catch (error) {
        console.error('Failed to toggle comment:', error)
      }
    },

    async duplicateLine() {
      if (!this.editor) return

      try {
        const action = this.editor.getAction('editor.action.copyLinesDownAction')
        if (action) {
          await action.run()
        }
      } catch (error) {
        console.error('Failed to duplicate line:', error)
      }
    },

    async moveLineUp() {
      if (!this.editor) return

      try {
        const action = this.editor.getAction('editor.action.moveLinesUpAction')
        if (action) {
          await action.run()
        }
      } catch (error) {
        console.error('Failed to move line up:', error)
      }
    },

    async moveLineDown() {
      if (!this.editor) return

      try {
        const action = this.editor.getAction('editor.action.moveLinesDownAction')
        if (action) {
          await action.run()
        }
      } catch (error) {
        console.error('Failed to move line down:', error)
      }
    },

    setLanguage(language: string) {
      this.language = language
      
      if (this.editor) {
        const model = this.editor.getModel()
        if (model) {
          monaco.editor.setModelLanguage(model, language)
        }
      }
    },

    setTheme(theme: string) {
      this.theme = theme
      monaco.editor.setTheme(theme)
    },

    setFontSize(size: number) {
      this.fontSize = size
      
      if (this.editor) {
        this.editor.updateOptions({ fontSize: size })
      }
    },

    updateOptions(options: Partial<typeof this.$state>) {
      Object.assign(this, options)
      
      if (this.editor) {
        this.editor.updateOptions({
          fontSize: this.fontSize,
          wordWrap: this.wordWrap ? 'on' : 'off',
          minimap: { enabled: this.minimap },
          lineNumbers: this.lineNumbers ? 'on' : 'off',
          tabSize: this.tabSize,
          insertSpaces: this.insertSpaces,
          autoClosingBrackets: this.autoClosingBrackets,
          autoClosingQuotes: this.autoClosingQuotes,
          bracketPairColorization: { enabled: this.bracketPairColorization },
          guides: this.guides
        })
      }
    },

    setupKeyboardShortcuts() {
      if (!this.editor) return

      // Custom keyboard shortcuts can be registered here
      // For example, snippet insertion shortcuts
    },

    async autoSaveCurrentFile() {
      // This would integrate with the file store to auto-save
      console.log('Auto-saving current file...')
    },

    // Multi-cursor operations
    addCursor(position: monaco.Position) {
      if (!this.editor) return

      this.editor.setPositions([
        ...this.editor.getPositions(),
        position
      ])
    },

    removeCursor(index: number) {
      if (!this.editor) return

      const positions = this.editor.getPositions()
      if (index >= 0 && index < positions.length) {
        positions.splice(index, 1)
        this.editor.setPositions(positions)
      }
    },

    clearExtraCursors() {
      if (!this.editor) return

      const selection = this.editor.getSelection()
      if (selection) {
        this.editor.setPositions([selection.getPosition()])
      }
    },

    // Search and replace
    findNext() {
      if (!this.editor) return

      this.editor.getAction('editor.action.nextMatchFindAction')?.run()
    },

    findPrevious() {
      if (!this.editor) return

      this.editor.getAction('editor.action.previousMatchFindAction')?.run()
    },

    replace() {
      if (!this.editor) return

      this.editor.getAction('editor.action.replace')?.run()
    },

    replaceAll() {
      if (!this.editor) return

      this.editor.getAction('editor.action.replaceAll')?.run()
    },

    // Navigation
    goToLine(lineNumber: number) {
      if (!this.editor) return

      const position = new monaco.Position(lineNumber, 1)
      this.editor.setPosition(position)
      this.editor.revealLineInCenter(lineNumber)
    },

    goToStartOfFile() {
      if (!this.editor) return

      this.editor.setPosition(new monaco.Position(1, 1))
    },

    goToEndOfFile() {
      if (!this.editor) return

      const model = this.editor.getModel()
      if (model) {
        const lineCount = model.getLineCount()
        this.editor.setPosition(new monaco.Position(lineCount, 1))
      }
    }
  }
})