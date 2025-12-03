//! Language tools and utilities for code analysis

use std::collections::HashMap;
use tree_sitter::{Parser, Node};
use crate::ai::{FunctionInfo, VariableInfo, CodeComplexity};

/// Language-specific parsing and analysis tools
pub struct LanguageTools {
    parsers: HashMap<String, Parser>,
}

impl LanguageTools {
    pub fn new() -> Self {
        let parsers = HashMap::new();
        
        // Initialize tree-sitter parsers for different languages
        #[cfg(feature = "tree-sitter-rust")]
        {
            let mut rust_parser = Parser::new();
            rust_parser.set_language(tree_sitter_rust::language());
            parsers.insert("rust".to_string(), rust_parser);
        }
        
        #[cfg(feature = "tree-sitter-python")]
        {
            let mut python_parser = Parser::new();
            python_parser.set_language(tree_sitter_python::language());
            parsers.insert("python".to_string(), python_parser);
        }
        
        #[cfg(feature = "tree-sitter-javascript")]
        {
            let mut js_parser = Parser::new();
            js_parser.set_language(tree_sitter_javascript::language());
            parsers.insert("javascript".to_string(), js_parser);
        }
        
        Self { parsers }
    }
    
    /// Parse code and return syntax tree
    pub fn parse_code(&mut self, code: &str, language: &str) -> Option<tree_sitter::Tree> {
        if let Some(parser) = self.parsers.get_mut(language) {
            parser.parse(code, None)
        } else {
            None
        }
    }
    
    /// Extract functions from code
    pub fn extract_functions(&mut self, code: &str, language: &str) -> Vec<FunctionInfo> {
        let mut functions = Vec::new();
        
        if let Some(tree) = self.parse_code(code, language) {
            let root_node = tree.root_node();
            self.extract_functions_recursive(&root_node, code, &mut functions);
        }
        
        functions
    }
    
    /// Extract variables from code
    pub fn extract_variables(&mut self, code: &str, language: &str) -> Vec<VariableInfo> {
        let mut variables = Vec::new();
        
        if let Some(tree) = self.parse_code(code, language) {
            let root_node = tree.root_node();
            self.extract_variables_recursive(&root_node, code, &mut variables);
        }
        
        variables
    }
    
    /// Calculate code complexity
    pub fn calculate_complexity(&mut self, code: &str, language: &str) -> CodeComplexity {
        let mut cyclomatic = 1;
        let mut cognitive = 1;
        let mut nested_depth = 0;
        let mut max_depth = 0;
        
        if let Some(tree) = self.parse_code(code, language) {
            let root_node = tree.root_node();
            self.calculate_complexity_recursive(&root_node, &mut cyclomatic, &mut cognitive, &mut nested_depth, &mut max_depth);
        }
        
        let lines_of_code = code.lines().count();
        let maintainability_index = self.calculate_maintainability_index(cyclomatic, lines_of_code, max_depth);
        
        CodeComplexity {
            cyclomatic_complexity: cyclomatic,
            cognitive_complexity: cognitive,
            maintainability_index,
            lines_of_code: lines_of_code as u32,
            nested_depth: max_depth,
            line_count: lines_of_code as u32,
            function_count: 1, // Simplified: assume one function for demo
            complexity_score: cyclomatic as f32, // Simplified calculation
        }
    }
    
    /// Recursively extract functions
    fn extract_functions_recursive(&self, node: &Node, code: &str, functions: &mut Vec<FunctionInfo>) {
        let node_type = node.kind();
        
        match node_type {
            "function_item" | "function_definition" | "method_definition" => {
                if let Some(name_node) = self.find_child_by_type(node, "identifier") {
                    let name = name_node.utf8_text(code.as_bytes()).unwrap().to_string();
                    let signature = self.extract_function_signature(node);
                    
                    functions.push(FunctionInfo {
                        name,
                        line_start: 1, // Simplified: would extract from node position
                        line_end: 10, // Simplified: would extract from node position
                        parameters: vec![], // Would extract from signature
                        return_type: Some("void".to_string()), // Simplified: would parse return type
                        complexity: 1.0, // Would calculate from body
                        signature: Some(signature),
                        docstring: None, // Would extract from comments
                    });
                }
            },
            _ => {
                // Recursively process children
                let mut cursor = node.walk();
                for child in cursor.node().children(&mut cursor) {
                    self.extract_functions_recursive(&child, code, functions);
                }
            }
        }
    }
    
    /// Recursively extract variables
    fn extract_variables_recursive(&self, node: &Node, code: &str, variables: &mut Vec<VariableInfo>) {
        let node_type = node.kind();
        
        match node_type {
            "let_declaration" | "var_declaration" | "const_declaration" => {
                if let Some(name_node) = self.find_child_by_type(node, "identifier") {
                    let name = name_node.utf8_text(code.as_bytes()).unwrap().to_string();
                    
                    variables.push(VariableInfo {
                        name,
                        line: 1, // Simplified: would extract from node position
                        column: 1, // Simplified: would extract from node position
                        scope: "local".to_string(),
                        variable_type: crate::ai::VariableType::Local,
                        is_declared: true,
                        var_type: Some("unknown".to_string()),
                        is_mutable: node_type.contains("mut") || node_type.contains("var"),
                    });
                }
            },
            _ => {
                // Recursively process children
                let mut cursor = node.walk();
                for child in cursor.node().children(&mut cursor) {
                    self.extract_variables_recursive(&child, code, variables);
                }
            }
        }
    }
    
    /// Recursively calculate complexity metrics
    fn calculate_complexity_recursive(&self, node: &Node, cyclomatic: &mut u32, cognitive: &mut u32, depth: &mut u32, max_depth: &mut u32) {
        let node_type = node.kind();
        
        // Update max depth
        if *depth > *max_depth {
            *max_depth = *depth;
        }
        
        // Cyclomatic complexity incrementers
        match node_type {
            "if_statement" | "for_statement" | "while_statement" | "match_expression" | "try_expression" => {
                *cyclomatic += 1;
                *cognitive += 1;
            },
            "binary_expression" => {
                *cognitive += 1;
            },
            _ => {}
        }
        
        // Recursively process children
        let mut cursor = node.walk();
        for child in cursor.node().children(&mut cursor) {
            if self.is_control_flow_node(&child) {
                *depth += 1;
            }
            
            self.calculate_complexity_recursive(&child, cyclomatic, cognitive, depth, max_depth);
            
            if self.is_control_flow_node(&child) {
                *depth -= 1;
            }
        }
    }
    
    /// Calculate maintainability index
    fn calculate_maintainability_index(&self, cyclomatic: u32, lines_of_code: usize, max_nesting: u32) -> f32 {
        // Simplified maintainability index calculation
        let halstead = (lines_of_code as f32).ln(); // Simplified
        let cyclomatic_penalty = cyclomatic as f32 * 0.1;
        let nesting_penalty = max_nesting as f32 * 0.5;
        
        let mut_index = 171.0 - 5.2 * halstead.log10() - 0.23 * cyclomatic_penalty - 16.2 * nesting_penalty.log10();
        
        (mut_index.max(0.0).min(100.0)) as f32
    }
    
    /// Check if node represents control flow
    fn is_control_flow_node(&self, node: &Node) -> bool {
        matches!(node.kind(), 
            "if_statement" | "for_statement" | "while_statement" | "match_expression" | "try_expression"
        )
    }
    
    /// Find child node by type
    fn find_child_by_type<'a>(&self, node: &'a Node, node_type: &str) -> Option<Node<'a>> {
        let mut cursor = node.walk();
        for child in cursor.node().children(&mut cursor) {
            if child.kind() == node_type {
                return Some(child);
            }
        }
        None
    }
    
    /// Extract function signature
    fn extract_function_signature(&self, node: &Node) -> String {
        // Simplified signature extraction
        node.utf8_text(&[]).unwrap_or("").to_string()
    }
    
    /// Get language-specific keywords
    pub fn get_keywords(&self, language: &str) -> Vec<String> {
        match language {
            "rust" => vec![
                "fn", "let", "mut", "const", "struct", "enum", "trait", "impl", 
                "if", "else", "match", "for", "while", "loop", "break", "continue"
            ].iter().map(|s| s.to_string()).collect(),
            "python" => vec![
                "def", "class", "if", "elif", "else", "for", "while", "try", 
                "except", "finally", "with", "as", "import", "from"
            ].iter().map(|s| s.to_string()).collect(),
            "javascript" => vec![
                "function", "var", "let", "const", "class", "if", "else", 
                "for", "while", "try", "catch", "throw", "import", "export"
            ].iter().map(|s| s.to_string()).collect(),
            _ => vec![],
        }
    }
    
    /// Get language-specific operators
    pub fn get_operators(&self, language: &str) -> Vec<String> {
        match language {
            "rust" => vec![
                "+", "-", "*", "/", "%", "==", "!=", "<", ">", "<=", ">=", 
                "&&", "||", "!", "&", "|", "^", "<<", ">>", "=", "+=", "-=", "*="
            ].iter().map(|s| s.to_string()).collect(),
            "python" => vec![
                "+", "-", "*", "/", "%", "**", "//", "==", "!=", "<", ">", 
                "<=", ">=", "and", "or", "not", "&", "|", "^", "~", "<<", ">>"
            ].iter().map(|s| s.to_string()).collect(),
            "javascript" => vec![
                "+", "-", "*", "/", "%", "==", "===", "!=", "!==", "<", ">", 
                "<=", ">=", "&&", "||", "!", "&", "|", "^", "<<", ">>", ">>>", "="
            ].iter().map(|s| s.to_string()).collect(),
            _ => vec![],
        }
    }
}