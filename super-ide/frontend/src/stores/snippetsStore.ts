import { defineStore } from 'pinia'
import type { Snippet, SnippetCategory } from '../types'

export const useSnippetsStore = defineStore('snippets', {
  state: () => ({
    snippets: [] as Snippet[],
    categories: [
      { id: 'functions', name: 'Functions', icon: 'Zap' },
      { id: 'loops', name: 'Loops', icon: 'RotateCw' },
      { id: 'conditionals', name: 'Conditionals', icon: 'GitBranch' },
      { id: 'classes', name: 'Classes', icon: 'Package' },
      { id: 'async', name: 'Async', icon: 'Clock' },
      { id: 'testing', name: 'Testing', icon: 'TestTube' },
      { id: 'debugging', name: 'Debugging', icon: 'Bug' },
      { id: 'database', name: 'Database', icon: 'Database' },
      { id: 'api', name: 'API', icon: 'Globe' },
      { id: 'utils', name: 'Utilities', icon: 'Tool' }
    ] as SnippetCategory[],
    loading: false,
    error: null as string | null
  }),

  getters: {
    getSnippets: (state) => state.snippets,
    
    getSnippetsByCategory: (state) => (category: string) => {
      if (category === 'favorites') {
        return state.snippets.filter(s => s.favorite)
      }
      return state.snippets.filter(s => s.category === category)
    },

    getFavoriteSnippets: (state) => state.snippets.filter(s => s.favorite),

    getSnippetsByLanguage: (state) => (language: string) => 
      state.snippets.filter(s => s.languages.includes(language)),

    getMostUsedSnippets: (state) => 
      [...state.snippets].sort((a, b) => b.usageCount - a.usageCount).slice(0, 10),

    searchSnippets: (state) => (query: string) => {
      const lowerQuery = query.toLowerCase()
      return state.snippets.filter(s =>
        s.name.toLowerCase().includes(lowerQuery) ||
        s.description.toLowerCase().includes(lowerQuery) ||
        s.code.toLowerCase().includes(lowerQuery) ||
        s.tags.some(tag => tag.toLowerCase().includes(lowerQuery))
      )
    }
  },

  actions: {
    async loadSnippets() {
      this.loading = true
      this.error = null
      
      try {
        // Try to load from localStorage first
        const saved = localStorage.getItem('super-ide-snippets')
        if (saved) {
          this.snippets = JSON.parse(saved)
        } else {
          // Load default snippets if none saved
          this.snippets = this.getDefaultSnippets()
          this.saveSnippets()
        }
      } catch (error) {
        this.error = 'Failed to load snippets'
        console.error('Error loading snippets:', error)
        this.snippets = this.getDefaultSnippets()
      } finally {
        this.loading = false
      }
    },

    async saveSnippets() {
      try {
        localStorage.setItem('super-ide-snippets', JSON.stringify(this.snippets))
      } catch (error) {
        console.error('Error saving snippets:', error)
        this.error = 'Failed to save snippets'
      }
    },

    addSnippet(snippet: Snippet) {
      const newSnippet: Snippet = {
        ...snippet,
        id: snippet.id || Date.now().toString(),
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        usageCount: 0,
        lastUsed: ''
      }
      
      this.snippets.push(newSnippet)
      this.saveSnippets()
    },

    updateSnippet(snippet: Snippet) {
      const index = this.snippets.findIndex(s => s.id === snippet.id)
      if (index !== -1) {
        this.snippets[index] = {
          ...snippet,
          updatedAt: new Date().toISOString()
        }
        this.saveSnippets()
      }
    },

    removeSnippet(snippetId: string) {
      this.snippets = this.snippets.filter(s => s.id !== snippetId)
      this.saveSnippets()
    },

    toggleFavorite(snippetId: string) {
      const snippet = this.snippets.find(s => s.id === snippetId)
      if (snippet) {
        snippet.favorite = !snippet.favorite
        snippet.updatedAt = new Date().toISOString()
        this.saveSnippets()
      }
    },

    incrementUsage(snippetId: string) {
      const snippet = this.snippets.find(s => s.id === snippetId)
      if (snippet) {
        snippet.usageCount++
        this.saveSnippets()
      }
    },

    updateLastUsed(snippetId: string) {
      const snippet = this.snippets.find(s => s.id === snippetId)
      if (snippet) {
        snippet.lastUsed = new Date().toISOString()
        this.saveSnippets()
      }
    },

    importSnippets(snippets: Snippet[]) {
      // Merge with existing snippets, avoiding duplicates
      const existingIds = new Set(this.snippets.map(s => s.id))
      const newSnippets = snippets.filter(s => !existingIds.has(s.id))
      
      this.snippets.push(...newSnippets)
      this.saveSnippets()
    },

    exportSnippets(): Snippet[] {
      return [...this.snippets]
    },

    getDefaultSnippets(): Snippet[] {
      const now = new Date().toISOString()
      
      return [
        // JavaScript/TypeScript Functions
        {
          id: '1',
          name: 'Async Function',
          description: 'Create an async function with try-catch',
          code: `async function functionName() {
  try {
    // Async operation
    const result = await someAsyncCall();
    return result;
  } catch (error) {
    console.error('Error:', error);
    throw error;
  }
}`,
          language: 'javascript',
          languages: ['javascript', 'typescript'],
          category: 'functions',
          tags: ['async', 'function', 'error-handling'],
          favorite: false,
          usageCount: 0,
          lastUsed: '',
          createdAt: now,
          updatedAt: now
        },
        
        {
          id: '2',
          name: 'Array Map',
          description: 'Transform array elements with map',
          code: `array.map((item, index) => {
  // Transform item
  return transformedItem;
});`,
          language: 'javascript',
          languages: ['javascript', 'typescript'],
          category: 'functions',
          tags: ['array', 'map', 'transform'],
          favorite: false,
          usageCount: 0,
          lastUsed: '',
          createdAt: now,
          updatedAt: now
        },

        // Python Functions
        {
          id: '3',
          name: 'Python Class',
          description: 'Create a Python class with init method',
          code: `class ClassName:
    def __init__(self, parameter):
        self.parameter = parameter
    
    def method_name(self):
        # Method implementation
        pass`,
          language: 'python',
          languages: ['python'],
          category: 'classes',
          tags: ['class', 'oop', 'python'],
          favorite: false,
          usageCount: 0,
          lastUsed: '',
          createdAt: now,
          updatedAt: now
        },

        {
          id: '4',
          name: 'Try-Except Block',
          description: 'Python exception handling',
          code: `try:
    # Risky operation
    result = risky_function()
except Exception as e:
    print(f"Error occurred: {e}")
    # Handle error
else:
    # Success case
    print("Operation successful")
finally:
    # Cleanup
    cleanup_function()`,
          language: 'python',
          languages: ['python'],
          category: 'utils',
          tags: ['exception', 'error-handling', 'try-catch'],
          favorite: false,
          usageCount: 0,
          lastUsed: '',
          createdAt: now,
          updatedAt: now
        },

        // Rust Functions
        {
          id: '5',
          name: 'Rust Function',
          description: 'Rust function with error handling',
          code: `fn function_name(parameter: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Function implementation
    let result = process_data(parameter)?;
    Ok(result)
}`,
          language: 'rust',
          languages: ['rust'],
          category: 'functions',
          tags: ['function', 'error-handling', 'rust'],
          favorite: false,
          usageCount: 0,
          lastUsed: '',
          createdAt: now,
          updatedAt: now
        },

        {
          id: '6',
          name: 'Rust Match Expression',
          description: 'Rust pattern matching',
          code: `match value {
    Pattern1 => {
        // Handle pattern 1
    },
    Pattern2 => {
        // Handle pattern 2
    },
    _ => {
        // Default case
    }
}`,
          language: 'rust',
          languages: ['rust'],
          category: 'utils',
          tags: ['match', 'pattern', 'rust'],
          favorite: false,
          usageCount: 0,
          lastUsed: '',
          createdAt: now,
          updatedAt: now
        },

        // Testing
        {
          id: '7',
          name: 'Jest Test',
          description: 'Jest test case template',
          code: `describe('ComponentName', () => {
  beforeEach(() => {
    // Setup
  });

  test('should do something', () => {
    // Arrange
    const input = 'test';
    
    // Act
    const result = functionUnderTest(input);
    
    // Assert
    expect(result).toBe(expectedValue);
  });

  afterEach(() => {
    // Cleanup
  });
});`,
          language: 'javascript',
          languages: ['javascript', 'typescript'],
          category: 'testing',
          tags: ['jest', 'test', 'unit-testing'],
          favorite: false,
          usageCount: 0,
          lastUsed: '',
          createdAt: now,
          updatedAt: now
        },

        {
          id: '8',
          name: 'pytest Test',
          description: 'Pytest test function',
          code: `def test_function():
    # Arrange
    input_data = "test"
    expected = "expected"
    
    # Act
    result = function_under_test(input_data)
    
    # Assert
    assert result == expected

@pytest.fixture
def fixture_name():
    # Fixture setup
    return some_value`,
          language: 'python',
          languages: ['python'],
          category: 'testing',
          tags: ['pytest', 'test', 'python'],
          favorite: false,
          usageCount: 0,
          lastUsed: '',
          createdAt: now,
          updatedAt: now
        },

        // API
        {
          id: '9',
          name: 'Fetch API Call',
          description: 'JavaScript fetch with error handling',
          code: `async function fetchData(url) {
  try {
    const response = await fetch(url);
    
    if (!response.ok) {
      throw new Error(\`HTTP error! status: \${response.status}\`);
    }
    
    const data = await response.json();
    return data;
  } catch (error) {
    console.error('Fetch error:', error);
    throw error;
  }
}`,
          language: 'javascript',
          languages: ['javascript', 'typescript'],
          category: 'api',
          tags: ['fetch', 'api', 'http'],
          favorite: false,
          usageCount: 0,
          lastUsed: '',
          createdAt: now,
          updatedAt: now
        },

        // Loops
        {
          id: '10',
          name: 'For-of Loop',
          description: 'Modern JavaScript for-of loop',
          code: `for (const item of array) {
  // Process item
  console.log(item);
}`,
          language: 'javascript',
          languages: ['javascript', 'typescript'],
          category: 'loops',
          tags: ['loop', 'for-of', 'array'],
          favorite: false,
          usageCount: 0,
          lastUsed: '',
          createdAt: now,
          updatedAt: now
        },

        // Debugging
        {
          id: '11',
          name: 'Console Debug',
          description: 'Debug logging with console',
          code: `console.group('Debug Info');
console.log('Variable:', variable);
console.log('Object:', obj);
console.table(dataArray);
console.groupEnd();`,
          language: 'javascript',
          languages: ['javascript', 'typescript'],
          category: 'debugging',
          tags: ['console', 'debug', 'logging'],
          favorite: false,
          usageCount: 0,
          lastUsed: '',
          createdAt: now,
          updatedAt: now
        }
      ]
    }
  }
})