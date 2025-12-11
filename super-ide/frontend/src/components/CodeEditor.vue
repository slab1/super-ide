<template>
  <div ref="editorContainer" class="w-full h-full"></div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from 'vue'
import * as monaco from 'monaco-editor'
import { useFileStore } from '../stores/fileStore'
import { useAIStore } from '../stores/aiStore'
import type { FileInfo, CodeCompletion } from '../types'

interface Props {
  file: FileInfo
}

const props = defineProps<Props>()
const emit = defineEmits<{
  contentChanged: [content: string]
}>()

const editorContainer = ref<HTMLDivElement>()
let editor: monaco.editor.IStandaloneCodeEditor | null = null

const fileStore = useFileStore()
const aiStore = useAIStore()

onMounted(async () => {
  if (!editorContainer.value) return

  // Create Monaco Editor
  editor = monaco.editor.create(editorContainer.value, {
    value: '',
    language: getLanguageFromFileName(props.file.name),
    theme: 'vs-dark',
    fontSize: 14,
    fontFamily: 'Fira Code, Monaco, Consolas, monospace',
    automaticLayout: true,
    minimap: { enabled: true },
    lineNumbers: 'on',
    wordWrap: 'on',
    tabSize: 2,
    insertSpaces: true,
    formatOnPaste: true,
    formatOnType: true,
    suggestOnTriggerCharacters: true,
    acceptSuggestionOnEnter: 'on',
    quickSuggestions: {
      other: true,
      comments: true,
      strings: true
    },
    parameterHints: {
      enabled: true
    },
    hover: {
      enabled: true
    },
    folding: true,
    foldingStrategy: 'auto',
    showFoldingControls: 'always',
    bracketPairColorization: {
      enabled: true
    },
    guides: {
      bracketPairs: true,
      indentation: true
    }
  })

  // Load file content
  await loadFileContent()

  // Set up content change listener
  editor.onDidChangeModelContent(() => {
    const content = editor?.getValue() || ''
    emit('contentChanged', content)
  })

  // Set up AI completion provider
  setupAICompletions()
  
  // Set up advanced AI features
  setupAdvancedAI()
})

onUnmounted(() => {
  if (editor) {
    editor.dispose()
    editor = null
  }
})

watch(() => props.file, async () => {
  if (editor) {
    await loadFileContent()
    setupAICompletions()
    setupAdvancedAI()
  }
})

async function loadFileContent() {
  try {
    const content = await fileStore.loadFile(props.file.path)
    if (editor) {
      const model = editor.getModel()
      if (model) {
        monaco.editor.setModelLanguage(model, getLanguageFromFileName(props.file.name))
        model.setValue(content)
      }
    }
  } catch (error) {
    console.error('Failed to load file content:', error)
  }
}

function getLanguageFromFileName(filename: string): string {
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

function setupAICompletions() {
  if (!editor) return

  // Register AI completion provider
  const disposable = monaco.languages.registerCompletionItemProvider('*', {
    triggerCharacters: ['.', '(', '[', '{', ',', ' '],
    provideCompletionItems: async (model, position) => {
      try {
        const word = model.getWordUntilPosition(position)
        const range = {
          startLineNumber: position.lineNumber,
          endLineNumber: position.lineNumber,
          startColumn: word.startColumn,
          endColumn: word.endColumn
        }

        // Get AI suggestions
        const suggestions = await aiStore.getSmartCompletions(
          props.file.path,
          model.getValue(),
          position
        )

        return {
          suggestions: suggestions.map((suggestion: CodeCompletion) => ({
            ...suggestion,
            range: suggestion.range || range
          }))
        }
      } catch (error) {
        console.error('Failed to get AI completions:', error)
        return { suggestions: [] }
      }
    }
  })

  // Clean up previous provider
  editor.onDidDispose(() => {
    disposable.dispose()
  })
}

function setupAdvancedAI() {
  if (!editor) return

  // Set up AI-powered hover provider
  monaco.languages.registerHoverProvider('*', {
    provideHover: async (model, position) => {
      try {
        const word = model.getWordAtPosition(position)
        if (!word) return null

        const content = model.getValue()
        const language = getLanguageFromFileName(props.file.name)

        // Get contextual help from AI
        const help = await aiStore.getContextAwareHelp(
          `Explain the ${word.word} syntax or concept in ${language}`,
          {
            filePath: props.file.path,
            fileContent: content,
            language,
            cursorPosition: position
          }
        )

        if (help && help.length > 0) {
          return {
            range: new monaco.Range(
              position.lineNumber,
              word.startColumn,
              position.lineNumber,
              word.endColumn
            ),
            contents: [
              { value: `**${word.word}** - ${help[0].content}` }
            ]
          }
        }
      } catch (error) {
        console.error('Failed to get AI hover help:', error)
      }
      return null
    }
  })

  // Set up AI-powered diagnostics
  editor.onDidChangeModelContent(async () => {
    try {
      const content = editor?.getValue() || ''
      const language = getLanguageFromFileName(props.file.name)

      // Get AI code review
      const review = await aiStore.getCodeReview(content, language)
      
      if (review && review.length > 0) {
        // Clear previous AI diagnostics
        const markers = monaco.editor.getModelMarkers({})
        const aiMarkers = markers.filter(marker => 
          marker.source === 'AI Assistant' || 
          marker.code?.includes('AI')
        )
        aiMarkers.forEach(marker => monaco.editor.setModelMarkers(
          editor!.getModel()!,
          'AI Assistant',
          []
        ))

        // Add new AI diagnostics
        const newMarkers = review.map((issue: any) => ({
          severity: getMonacoSeverity(issue.severity),
          message: issue.title,
          startLineNumber: issue.line || 1,
          startColumn: 1,
          endLineNumber: issue.line || 1,
          endColumn: 1000,
          source: 'AI Assistant',
          code: `AI-${issue.id}`,
          relatedInformation: [
            {
              startLineNumber: issue.line || 1,
              startColumn: 1,
              endLineNumber: issue.line || 1,
              endColumn: 1000,
              message: issue.description
            }
          ]
        }))

        monaco.editor.setModelMarkers(
          editor!.getModel()!,
          'AI Assistant',
          newMarkers
        )
      }
    } catch (error) {
      console.error('Failed to get AI diagnostics:', error)
    }
  })

  // Set up AI-powered code actions
  monaco.languages.registerCodeActionProvider('*', {
    provideCodeActions: async (model, range, context, token) => {
      try {
        const content = model.getValue()
        const language = getLanguageFromFileName(props.file.name)

        // Get refactoring suggestions from AI
        const suggestions = await aiStore.getRefactoringSuggestions(content, language)
        
        const actions = suggestions
          .filter((s: any) => s.canAutoFix)
          .map((suggestion: any) => ({
            title: `AI: ${suggestion.title}`,
            kind: monaco.languages.CodeActionKind.Refactor,
            edit: {
              edits: [{
                range: range,
                text: suggestion.suggestedRefactoring || ''
              }]
            },
            isPreferred: suggestion.confidence > 0.8
          }))

        return { actions }
      } catch (error) {
        console.error('Failed to get AI code actions:', error)
        return { actions: [] }
      }
    }
  })
}

function getMonacoSeverity(severity: string): monaco.MarkerSeverity {
  switch (severity.toLowerCase()) {
    case 'error':
      return monaco.MarkerSeverity.Error
    case 'warning':
      return monaco.MarkerSeverity.Warning
    case 'info':
      return monaco.MarkerSeverity.Info
    default:
      return monaco.MarkerSeverity.Info
  }
}
</script>

<style scoped>
:deep(.monaco-editor) {
  background-color: #1a1a1a;
}

:deep(.monaco-editor .margin) {
  background-color: #252526;
}

:deep(.monaco-editor .monaco-editor-background) {
  background-color: #1e1e1e;
}
</style>