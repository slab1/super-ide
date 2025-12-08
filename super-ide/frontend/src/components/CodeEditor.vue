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
        const suggestions = await aiStore.getCompletions(
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