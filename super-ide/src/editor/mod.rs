//! Code editor with syntax highlighting, auto-completion, and document management

use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use thiserror::Error;
use serde::{Deserialize, Serialize};

use crate::config::Configuration;
use crate::utils::file_manager::FileManager;

/// Editor errors
#[derive(Error, Debug)]
pub enum EditorError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Invalid syntax: {0}")]
    SyntaxError(String),
    
    #[error("Document error: {0}")]
    Document(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
}

/// Document model representing an open file
#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    pub path: std::path::PathBuf,
    pub title: String,
    pub content: Arc<RwLock<String>>,
    pub language: String,
    pub is_modified: bool,
    pub last_saved: Option<chrono::DateTime<chrono::Utc>>,
    pub syntax_tree: Option<SyntaxTree>,
    pub bookmarks: Vec<Bookmark>,
    pub fold_points: Vec<FoldPoint>,
    pub cursor_line: usize,
    pub cursor_column: usize,
}

/// Syntax tree for code structure
#[derive(Debug, Clone)]
pub struct SyntaxTree {
    pub nodes: Vec<SyntaxNode>,
    pub root: usize,
}

/// Syntax node in the parse tree
#[derive(Debug, Clone)]
pub struct SyntaxNode {
    pub node_type: String,
    pub start_byte: usize,
    pub end_byte: usize,
    pub children: Vec<usize>,
    pub text: String,
}

/// Bookmark for quick navigation
#[derive(Debug, Clone)]
pub struct Bookmark {
    pub id: String,
    pub line: usize,
    pub column: usize,
    pub description: String,
}

/// Code folding point
#[derive(Debug, Clone)]
pub struct FoldPoint {
    pub start_line: usize,
    pub end_line: usize,
    pub is_folded: bool,
}

/// Cursor position
#[derive(Debug, Clone, Copy)]
pub struct CursorPosition {
    pub line: usize,
    pub column: usize,
}

/// Selection range
#[derive(Debug, Clone)]
pub struct Selection {
    pub start: CursorPosition,
    pub end: CursorPosition,
    pub is_rectangular: bool,
}

/// Code buffer with editing operations
#[derive(Debug)]
pub struct CodeBuffer {
    document: Arc<RwLock<Document>>,
    cursor: CursorPosition,
    selection: Option<Selection>,
    undo_stack: Vec<EditOperation>,
    redo_stack: Vec<EditOperation>,
}

/// Edit operation for undo/redo
#[derive(Debug, Clone)]
pub struct EditOperation {
    pub operation_type: EditType,
    pub position: CursorPosition,
    pub text: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Types of edit operations
#[derive(Debug, Clone)]
pub enum EditType {
    Insert,
    Delete,
    Replace,
    Format,
}

/// Language support configuration
#[derive(Debug, Clone)]
pub struct LanguageSupport {
    pub name: String,
    pub extensions: Vec<String>,
    pub keywords: Vec<String>,
    pub builtins: Vec<String>,
    pub comment_syntax: CommentSyntax,
    pub syntax_highlighting: SyntaxHighlighting,
}

/// Comment syntax for a language
#[derive(Debug, Clone)]
pub struct CommentSyntax {
    pub line_comments: Vec<String>,
    pub block_comments: Option<(String, String)>,
}

/// Syntax highlighting rules
#[derive(Debug, Clone)]
pub struct SyntaxHighlighting {
    pub keywords: Vec<TokenRule>,
    pub strings: Vec<TokenRule>,
    pub numbers: Vec<TokenRule>,
    pub comments: Vec<TokenRule>,
    pub operators: Vec<TokenRule>,
}

/// Token rule for syntax highlighting
#[derive(Debug, Clone)]
pub struct TokenRule {
    pub pattern: String,
    pub token_type: String,
    pub is_regex: bool,
}

/// Auto-completion context
#[derive(Debug, Clone)]
pub struct CompletionContext {
    pub cursor_position: CursorPosition,
    pub language: String,
    pub text_before_cursor: String,
    pub text_after_cursor: String,
}

/// Completion item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionKind,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub insert_text: String,
    pub sort_text: String,
}

/// Types of completion items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompletionKind {
    Text,
    Method,
    Function,
    Constructor,
    Field,
    Variable,
    Class,
    Interface,
    Module,
    Property,
    Unit,
    Value,
    Enum,
    EnumMember,
    Struct,
    ClassMethod,
    VariableMethod,
    PropertyMethod,
    Keyword,
    Snippet,
    Color,
    File,
    Reference,
    Folder,
    TypeParameter,
    User,
    Issue,
}

/// Main editor instance
#[derive(Debug)]
pub struct Editor {
    documents: Arc<RwLock<Vec<Arc<RwLock<Document>>>>>,
    active_document: Arc<RwLock<Option<Arc<RwLock<Document>>>>>,
    file_manager: FileManager,
    language_support: Arc<RwLock<Vec<LanguageSupport>>>,
    config: Arc<RwLock<Configuration>>,
}

impl Editor {
    /// Create a new editor instance
    pub async fn new(config: &Configuration) -> Result<Self, EditorError> {
        let file_manager = FileManager::new().await
            .map_err(|e| EditorError::Config(e.to_string()))?;
            
        let mut language_support = Vec::new();
        
        // Initialize language support
        language_support.push(LanguageSupport {
            name: "Rust".to_string(),
            extensions: vec!["rs".to_string()],
            keywords: vec!["fn".to_string(), "let".to_string(), "mut".to_string(), "const".to_string()],
            builtins: vec!["String".to_string(), "Vec".to_string(), "Option".to_string()],
            comment_syntax: CommentSyntax {
                line_comments: vec!["//".to_string()],
                block_comments: Some(("/*".to_string(), "*/".to_string())),
            },
            syntax_highlighting: SyntaxHighlighting {
                keywords: vec![TokenRule {
                    pattern: r"\b(fn|let|mut|const|struct|enum|trait|impl)\b".to_string(),
                    token_type: "keyword".to_string(),
                    is_regex: true,
                }],
                strings: vec![TokenRule {
                    pattern: r#""[^"]*""#.to_string(),
                    token_type: "string".to_string(),
                    is_regex: true,
                }],
                numbers: vec![TokenRule {
                    pattern: r"\b\d+(\.\d+)?\b".to_string(),
                    token_type: "number".to_string(),
                    is_regex: true,
                }],
                comments: vec![TokenRule {
                    pattern: r"//[^\n]*".to_string(),
                    token_type: "comment".to_string(),
                    is_regex: true,
                }],
                operators: vec![TokenRule {
                    pattern: r"[+\-*/=<>!&|]".to_string(),
                    token_type: "operator".to_string(),
                    is_regex: true,
                }],
            },
        });
        
        // Add more language support...
        
        Ok(Self {
            documents: Arc::new(RwLock::new(Vec::new())),
            active_document: Arc::new(RwLock::new(None)),
            file_manager,
            language_support: Arc::new(RwLock::new(language_support)),
            config: Arc::new(RwLock::new(config.clone())),
        })
    }
    
    /// Open a file in the editor
    pub async fn open_file(&self, file_path: std::path::PathBuf) -> Result<String, EditorError> {
        let content = self.file_manager.read_file(&file_path)
            .await
            .map_err(|e| EditorError::FileNotFound(e.to_string()))?;
            
        let language = self.detect_language(&file_path).await;
        
        let document = Document {
            id: uuid::Uuid::new_v4().to_string(),
            path: file_path.clone(),
            title: file_path.file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or("Untitled")
                .to_string(),
            content: Arc::new(RwLock::new(content)),
            language,
            is_modified: false,
            last_saved: None,
            syntax_tree: None,
            bookmarks: Vec::new(),
            fold_points: Vec::new(),
            cursor_line: 0,
            cursor_column: 0,
        };
        
        let document_arc = Arc::new(RwLock::new(document));
        
        // Get document ID before moving
        let document_id = {
            let doc_read = document_arc.read().await;
            doc_read.id.clone()
        };
        
        // Add to documents list
        {
            let mut documents = self.documents.write().await;
            documents.push(document_arc.clone());
        }
        
        // Set as active document
        {
            let mut active = self.active_document.write().await;
            *active = Some(document_arc.clone());
        }
        
        // Parse syntax tree
        self.parse_syntax_tree(&document_arc).await;
        
        Ok(document_id)
    }
    
    /// Save the active document
    pub async fn save_active_document(&self) -> Result<(), EditorError> {
        let active = self.active_document.read().await;
        if let Some(doc) = active.as_ref() {
            let doc_read = doc.read().await;
            let content = {
                let content_guard = doc_read.content.read().await;
                content_guard.clone()
            };
            
            self.file_manager.write_file(&doc_read.path, &content)
                .await
                .map_err(|e| EditorError::Document(e.to_string()))?;
                
            // Mark as not modified
            let doc_id = doc_read.id.clone();
            drop(doc_read);
            drop(active);
            
            let mut active = self.active_document.write().await;
            if let Some(doc) = active.as_mut() {
                let mut doc_write = doc.write().await;
                if doc_write.id == doc_id {
                    doc_write.is_modified = false;
                    doc_write.last_saved = Some(chrono::Utc::now());
                }
            }
        }
        
        Ok(())
    }
    
    /// Get the active document
    pub async fn get_active_document(&self) -> Option<Arc<RwLock<Document>>> {
        let active = self.active_document.read().await;
        active.clone()
    }
    
    /// Get all open documents
    pub async fn get_documents(&self) -> Vec<Arc<RwLock<Document>>> {
        let documents = self.documents.read().await;
        documents.clone()
    }
    
    /// Close a document
    pub async fn close_document(&self, document_id: &str) -> Result<bool, EditorError> {
        let mut documents = self.documents.write().await;
        let mut active = self.active_document.write().await;
        
        // Check if the document being removed is the active one
        let is_active = {
            if let Some(active_doc) = active.as_ref() {
                if let Ok(active_read) = active_doc.try_read() {
                    active_read.id == document_id
                } else {
                    false
                }
            } else {
                false
            }
        };
        
        // Find and remove the document
        let index = documents.iter().position(|doc| {
            if let Ok(doc_read) = doc.try_read() {
                doc_read.id == document_id
            } else {
                false
            }
        });
        
        if let Some(index) = index {
            documents.remove(index);
            
            // If this was the active document, select another one
            if is_active {
                *active = None;
                if !documents.is_empty() {
                    *active = Some(documents[0].clone());
                }
            }
            
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// Insert text at cursor position
    pub async fn insert_text(&self, text: &str) -> Result<(), EditorError> {
        let active_doc_arc = {
            let active = self.active_document.read().await;
            active.clone()
        };
        
        if let Some(doc) = active_doc_arc {
            let mut doc_write = doc.write().await;
            
            // Calculate insertion position
            let position = {
                let content = doc_write.content.read().await;
                self.calculate_cursor_position(&content, &doc_write.cursor_line, &doc_write.cursor_column)
            };
            
            {
                let mut content = doc_write.content.write().await;
                content.insert_str(position, text);
            }
            
            doc_write.is_modified = true;
            
            // Update cursor position
            doc_write.cursor_line += text.lines().count().saturating_sub(1);
            let last_line_len = text.lines().last().map_or(0, |l| l.len());
            doc_write.cursor_column += last_line_len;
        }
        
        Ok(())
    }
    
    /// Delete text at cursor position
    pub async fn delete_text(&self, chars_to_delete: usize) -> Result<(), EditorError> {
        let active_doc_arc = {
            let active = self.active_document.read().await;
            active.clone()
        };
        
        if let Some(doc) = active_doc_arc {
            let mut doc_write = doc.write().await;
            
            // Calculate deletion position
            let position = {
                let content = doc_write.content.read().await;
                self.calculate_cursor_position(&content, &doc_write.cursor_line, &doc_write.cursor_column)
            };
            
            if position >= chars_to_delete {
                {
                    let mut content = doc_write.content.write().await;
                    content.drain(position - chars_to_delete..position);
                }
                doc_write.is_modified = true;
            }
        }
        
        Ok(())
    }
    
    /// Get auto-completion suggestions
    pub async fn get_completions(&self, context: &CompletionContext) -> Result<Vec<CompletionItem>, EditorError> {
        let active = self.active_document.read().await;
        let mut completions = Vec::new();
        
        if let Some(doc) = active.as_ref() {
            let doc_read = doc.read().await;
            let language_support = self.language_support.read().await;
            
            // Find language support
            if let Some(lang_support) = language_support.iter().find(|lang| lang.name == context.language) {
                // Get word before cursor
                let word = self.get_word_at_cursor(&context.text_before_cursor);
                
                // Add keyword completions
                for keyword in &lang_support.keywords {
                    if keyword.starts_with(&word) && !word.is_empty() {
                        completions.push(CompletionItem {
                            label: keyword.clone(),
                            kind: CompletionKind::Keyword,
                            detail: None,
                            documentation: None,
                            insert_text: keyword.clone(),
                            sort_text: keyword.clone(),
                        });
                    }
                }
                
                // Add built-in completions
                for builtin in &lang_support.builtins {
                    if builtin.starts_with(&word) && !word.is_empty() {
                        completions.push(CompletionItem {
                            label: builtin.clone(),
                            kind: CompletionKind::Class,
                            detail: Some("Built-in type".to_string()),
                            documentation: None,
                            insert_text: builtin.clone(),
                            sort_text: builtin.clone(),
                        });
                    }
                }
            }
        }
        
        // Sort completions by relevance
        completions.sort_by(|a, b| a.label.cmp(&b.label));
        
        Ok(completions)
    }
    
    /// Format the active document
    pub async fn format_document(&self) -> Result<(), EditorError> {
        let active = self.active_document.read().await;
        if let Some(doc) = active.as_ref() {
            // Get content and language to format
            let (content_to_format, language) = {
                let doc_read = doc.read().await;
                let content_clone = {
                    let content = doc_read.content.read().await;
                    content.clone()
                };
                (content_clone, doc_read.language.clone())
            };
            
            // Apply formatting based on language
            let formatted_content = match language.as_str() {
                "Rust" => {
                    // Apply basic Rust formatting (this would use rustfmt)
                    let lines: Vec<String> = content_to_format.lines().map(|line| {
                        line.trim_end().to_string()
                    }).collect();
                    lines.join("\n")
                },
                "Python" => {
                    // Apply Python formatting (this would use black/autopep8)
                    let lines: Vec<String> = content_to_format.lines().map(|line| {
                        line.trim_end().to_string()
                    }).collect();
                    lines.join("\n")
                },
                _ => {
                    // Generic formatting - remove trailing whitespace
                    let lines: Vec<String> = content_to_format.lines().map(|line| {
                        line.trim_end().to_string()
                    }).collect();
                    lines.join("\n")
                }
            };
            
            // Update content and mark as modified
            {
                let mut doc_write = doc.write().await;
                {
                    let mut content = doc_write.content.write().await;
                    *content = formatted_content;
                }
                doc_write.is_modified = true;
            }
            
            // Reparse syntax tree
            self.parse_syntax_tree(doc).await;
        }
        
        Ok(())
    }
    
    /// Get syntax highlighting tokens for a range of text
    pub async fn get_syntax_tokens(&self, document_id: &str, start_line: usize, end_line: usize) -> Result<Vec<SyntaxToken>, EditorError> {
        let documents = self.documents.read().await;
        let doc = documents.iter().find(|doc| {
            if let Ok(doc_read) = doc.try_read() {
                doc_read.id == document_id
            } else {
                false
            }
        }).ok_or_else(|| EditorError::Document("Document not found".to_string()))?;
        
        let doc_read = doc.read().await;
        let language_support = self.language_support.read().await;
        let content = doc_read.content.read().await;
        let lines: Vec<&str> = content.lines().collect();
        
        let mut tokens = Vec::new();
        
        // Get language support
        if let Some(lang_support) = language_support.iter().find(|lang| lang.name == doc_read.language) {
            for (line_num, line) in lines.iter().enumerate() {
                if line_num >= start_line && line_num <= end_line {
                    let line_tokens = self.tokenize_line(line, &lang_support.syntax_highlighting);
                    for token in line_tokens {
                        tokens.push(SyntaxToken {
                            token_type: token.token_type,
                            text: token.pattern.clone(),
                            line: line_num,
                            column: 0, // Would need more sophisticated parsing
                        });
                    }
                }
            }
        }
        
        Ok(tokens)
    }
    
    /// Parse syntax tree for a document
    async fn parse_syntax_tree(&self, document: &Arc<RwLock<Document>>) {
        // Extract content and document info without holding multiple borrows
        let (content, language) = {
            let doc_read = document.read().await;
            let content_clone = {
                let content = doc_read.content.read().await;
                content.clone()
            };
            (content_clone, doc_read.language.clone())
        };
        
        // Simple syntax tree generation (would use tree-sitter for real implementation)
        let mut nodes = Vec::new();
        
        for (_line_num, line) in content.lines().enumerate() {
            if line.starts_with("fn ") {
                nodes.push(SyntaxNode {
                    node_type: "function".to_string(),
                    start_byte: 0,
                    end_byte: 0,
                    children: Vec::new(),
                    text: line.to_string(),
                });
            }
        }
        
        let syntax_tree = SyntaxTree {
            nodes,
            root: 0,
        };
        
        let mut doc_write = document.write().await;
        doc_write.syntax_tree = Some(syntax_tree);
    }
    
    /// Detect language from file extension
    async fn detect_language(&self, file_path: &std::path::Path) -> String {
        let extension = file_path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
            
        let language_support = self.language_support.read().await;
        
        for lang in language_support.iter() {
            if lang.extensions.contains(&extension.to_string()) {
                return lang.name.clone();
            }
        }
        
        "Plain Text".to_string()
    }
    
    /// Calculate cursor position in bytes
    fn calculate_cursor_position(&self, content: &str, line: &usize, column: &usize) -> usize {
        let lines: Vec<&str> = content.lines().collect();
        
        if *line >= lines.len() {
            return content.len();
        }
        
        let mut position = 0;
        for (i, line_text) in lines.iter().enumerate() {
            if i == *line {
                break;
            }
            position += line_text.len() + 1; // +1 for newline
        }
        
        if *line < lines.len() {
            position + column.min(&lines[*line].len())
        } else {
            position
        }
    }
    
    /// Get word at cursor
    fn get_word_at_cursor(&self, text_before_cursor: &str) -> String {
        let words: Vec<&str> = text_before_cursor.split_whitespace().collect();
        words.last().unwrap_or(&"").to_string()
    }
    
    /// Tokenize a line for syntax highlighting
    fn tokenize_line(&self, line: &str, highlighting: &SyntaxHighlighting) -> Vec<TokenRule> {
        let mut tokens = Vec::new();
        
        // Simple tokenization (would use more sophisticated regex for real implementation)
        for rule in &highlighting.keywords {
            if line.contains(&rule.pattern) {
                tokens.push(rule.clone());
            }
        }
        
        for rule in &highlighting.strings {
            if line.contains(&rule.pattern) {
                tokens.push(rule.clone());
            }
        }
        
        tokens
    }
}

/// Syntax token for highlighting
#[derive(Debug, Clone)]
pub struct SyntaxToken {
    pub token_type: String,
    pub text: String,
    pub line: usize,
    pub column: usize,
}



impl Default for Document {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            path: std::path::PathBuf::new(),
            title: "Untitled".to_string(),
            content: Arc::new(RwLock::new("".to_string())),
            language: "Plain Text".to_string(),
            is_modified: false,
            last_saved: None,
            syntax_tree: None,
            bookmarks: Vec::new(),
            fold_points: Vec::new(),
            cursor_line: 0,
            cursor_column: 0,
        }
    }
}