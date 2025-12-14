//! Real-time Collaboration Module
//!
//! Provides Google Docs style collaborative editing capabilities with:
//! - Real-time document synchronization
//! - Operational Transform for conflict resolution
//! - User presence indicators
//! - Live cursors and selections
//! - Comment threads and discussions

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Collaboration session identifier
pub type SessionId = String;

/// User identifier in collaboration
pub type UserId = String;

/// Document identifier
pub type DocumentId = String;

/// User information in collaboration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationUser {
    pub id: UserId,
    pub name: String,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub color: String, // Color for cursor/selection display
    pub is_online: bool,
    pub joined_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}

/// Document in collaboration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationDocument {
    pub id: DocumentId,
    pub path: String,
    pub content: String,
    pub version: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub participants: HashSet<UserId>,
    pub comments: Vec<Comment>,
}

/// Comment in document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub author_id: UserId,
    pub content: String,
    pub line_number: Option<usize>,
    pub column_start: Option<usize>,
    pub column_end: Option<usize>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub resolved: bool,
    pub replies: Vec<CommentReply>,
}

/// Reply to a comment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentReply {
    pub id: String,
    pub author_id: UserId,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

/// Operational Transform operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    /// Insert text at position
    Insert {
        position: usize,
        text: String,
        timestamp: DateTime<Utc>,
        user_id: UserId,
    },
    /// Delete text at position
    Delete {
        position: usize,
        length: usize,
        timestamp: DateTime<Utc>,
        user_id: UserId,
    },
    /// Replace text range
    Replace {
        position: usize,
        old_length: usize,
        new_text: String,
        timestamp: DateTime<Utc>,
        user_id: UserId,
    },
}

/// User presence information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPresence {
    pub user_id: UserId,
    pub cursor_position: Option<(usize, usize)>,
    pub selection_range: Option<(usize, usize)>,
    pub is_typing: bool,
    pub last_seen: DateTime<Utc>,
}

/// Collaboration event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollaborationEvent {
    /// User joined session
    UserJoined {
        user: CollaborationUser,
        document: CollaborationDocument,
    },
    /// User left session
    UserLeft {
        user_id: UserId,
    },
    /// Document operation applied
    OperationApplied {
        operation: Operation,
        new_version: u64,
    },
    /// Cursor/selection updated
    PresenceUpdated {
        presence: UserPresence,
    },
    /// New comment added
    CommentAdded {
        comment: Comment,
    },
    /// Comment updated
    CommentUpdated {
        comment: Comment,
    },
    /// Comment resolved
    CommentResolved {
        comment_id: String,
    },
    /// Document synced
    DocumentSynced {
        document: CollaborationDocument,
    },
}

/// Collaboration session manager
#[derive(Debug)]
pub struct CollaborationManager {
    sessions: Arc<RwLock<HashMap<SessionId, CollaborationSession>>>,
    documents: Arc<RwLock<HashMap<DocumentId, CollaborationDocument>>>,
    users: Arc<RwLock<HashMap<UserId, CollaborationUser>>>,
    event_broadcasters: Arc<RwLock<HashMap<SessionId, broadcast::Sender<CollaborationEvent>>>>,
}

/// Individual collaboration session
#[derive(Debug, Clone)]
pub struct CollaborationSession {
    pub id: SessionId,
    pub document_id: DocumentId,
    pub participants: HashMap<UserId, UserPresence>,
    pub operations: Vec<Operation>,
    pub version: u64,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}

impl CollaborationManager {
    /// Create new collaboration manager
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            documents: Arc::new(RwLock::new(HashMap::new())),
            users: Arc::new(RwLock::new(HashMap::new())),
            event_broadcasters: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create new collaboration session
    pub async fn create_session(&self, document_id: DocumentId, creator_id: UserId) -> Result<SessionId> {
        let session_id = Uuid::new_v4().to_string();
        
        // Create collaboration document if it doesn't exist
        let mut documents = self.documents.write().await;
        if !documents.contains_key(&document_id) {
            let document = CollaborationDocument {
                id: document_id.clone(),
                path: "unknown".to_string(), // Will be updated by file manager
                content: String::new(),
                version: 0,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                participants: HashSet::new(),
                comments: Vec::new(),
            };
            documents.insert(document_id.clone(), document);
        }

        // Create collaboration session
        let mut sessions = self.sessions.write().await;
        let session = CollaborationSession {
            id: session_id.clone(),
            document_id: document_id.clone(),
            participants: HashMap::new(),
            operations: Vec::new(),
            version: 0,
            created_at: Utc::now(),
            last_activity: Utc::now(),
        };
        sessions.insert(session_id.clone(), session);

        // Create event broadcaster
        let (tx, _) = broadcast::channel(1000);
        let mut broadcasters = self.event_broadcasters.write().await;
        broadcasters.insert(session_id.clone(), tx);

        // Add creator as participant
        self.join_session(&session_id, &creator_id).await?;

        Ok(session_id)
    }

    /// Join existing collaboration session
    pub async fn join_session(&self, session_id: &SessionId, user_id: &UserId) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

        // Add user presence
        let presence = UserPresence {
            user_id: user_id.to_string(),
            cursor_position: None,
            selection_range: None,
            is_typing: false,
            last_seen: Utc::now(),
        };
        session.participants.insert(user_id.to_string(), presence);
        session.last_activity = Utc::now();

        // Update document participants
        let mut documents = self.documents.write().await;
        if let Some(document) = documents.get_mut(&session.document_id) {
            document.participants.insert(user_id.to_string());
        }

        // Broadcast user joined event
        self.broadcast_event(session_id, CollaborationEvent::UserJoined {
            user: self.get_user(user_id).await.unwrap_or(CollaborationUser {
                id: user_id.to_string(),
                name: "Unknown User".to_string(),
                email: None,
                avatar_url: None,
                color: "#3B82F6".to_string(),
                is_online: true,
                joined_at: Utc::now(),
                last_activity: Utc::now(),
            }),
            document: documents.get(&session.document_id).unwrap().clone(),
        }).await;

        Ok(())
    }

    /// Leave collaboration session
    pub async fn leave_session(&self, session_id: &SessionId, user_id: &UserId) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

        // Remove user from participants
        session.participants.remove(user_id);
        session.last_activity = Utc::now();

        // Update document participants
        let mut documents = self.documents.write().await;
        if let Some(document) = documents.get_mut(&session.document_id) {
            document.participants.remove(user_id);
        }

        // Broadcast user left event
        self.broadcast_event(session_id, CollaborationEvent::UserLeft {
            user_id: user_id.to_string(),
        }).await;

        Ok(())
    }

    /// Apply operation to document
    pub async fn apply_operation(&self, session_id: &SessionId, operation: Operation) -> Result<u64> {
        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

        let mut documents = self.documents.write().await;
        let document = documents.get_mut(&session.document_id)
            .ok_or_else(|| anyhow::anyhow!("Document not found"))?;

        // Apply operation to document content
        match &operation {
            Operation::Insert { position, text, .. } => {
                let mut content = document.content.clone();
                content.insert_str(*position, text);
                document.content = content;
            }
            Operation::Delete { position, length, .. } => {
                let mut content = document.content.clone();
                content.drain(*position..(*position + *length));
                document.content = content;
            }
            Operation::Replace { position, old_length, new_text, .. } => {
                let mut content = document.content.clone();
                content.drain(*position..(*position + *old_length));
                content.insert_str(*position, new_text);
                document.content = content;
            }
        }

        // Update document version and timestamp
        document.version += 1;
        document.updated_at = Utc::now();

        // Add operation to session history
        session.operations.push(operation.clone());
        session.version = document.version;
        session.last_activity = Utc::now();

        // Broadcast operation applied event
        self.broadcast_event(session_id, CollaborationEvent::OperationApplied {
            operation,
            new_version: document.version,
        }).await;

        Ok(document.version)
    }

    /// Update user presence (cursor position, selection, typing status)
    pub async fn update_presence(&self, session_id: &SessionId, presence: UserPresence) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

        session.participants.insert(presence.user_id.clone(), presence.clone());
        session.last_activity = Utc::now();

        // Broadcast presence updated event
        self.broadcast_event(session_id, CollaborationEvent::PresenceUpdated {
            presence,
        }).await;

        Ok(())
    }

    /// Add comment to document
    pub async fn add_comment(&self, session_id: &SessionId, comment: Comment) -> Result<()> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

        let mut documents = self.documents.write().await;
        let document = documents.get_mut(&session.document_id)
            .ok_or_else(|| anyhow::anyhow!("Document not found"))?;

        document.comments.push(comment.clone());

        // Broadcast comment added event
        self.broadcast_event(session_id, CollaborationEvent::CommentAdded {
            comment,
        }).await;

        Ok(())
    }

    /// Get collaboration session
    pub async fn get_session(&self, session_id: &SessionId) -> Option<CollaborationSession> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }

    /// Get collaboration document
    pub async fn get_document(&self, document_id: &DocumentId) -> Option<CollaborationDocument> {
        let documents = self.documents.read().await;
        documents.get(document_id).cloned()
    }

    /// Get user information
    pub async fn get_user(&self, user_id: &UserId) -> Option<CollaborationUser> {
        let users = self.users.read().await;
        users.get(user_id).cloned()
    }

    /// Register new user
    pub async fn register_user(&self, user: CollaborationUser) {
        let mut users = self.users.write().await;
        users.insert(user.id.clone(), user);
    }

    /// Get event broadcaster for session
    pub async fn get_event_broadcaster(&self, session_id: &SessionId) -> Option<broadcast::Receiver<CollaborationEvent>> {
        let broadcasters = self.event_broadcasters.read().await;
        broadcasters.get(session_id).map(|tx| tx.subscribe())
    }

    /// Broadcast event to session participants
    async fn broadcast_event(&self, session_id: &SessionId, event: CollaborationEvent) {
        let broadcasters = self.event_broadcasters.read().await;
        if let Some(tx) = broadcasters.get(session_id) {
            let _ = tx.send(event);
        }
    }

    /// List active sessions
    pub async fn list_sessions(&self) -> Vec<SessionId> {
        let sessions = self.sessions.read().await;
        sessions.keys().cloned().collect()
    }

    /// Get session participants
    pub async fn get_session_participants(&self, session_id: &SessionId) -> Option<HashMap<UserId, UserPresence>> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).map(|s| s.participants.clone())
    }
}

/// Operational Transform implementation
pub struct OperationalTransform;

impl OperationalTransform {
    /// Transform operation against another operation for conflict resolution
    pub fn transform(op1: &Operation, op2: &Operation) -> Operation {
        match (op1, op2) {
            (Operation::Insert { position: pos1, .. }, Operation::Insert { position: pos2, .. }) => {
                if pos1 <= pos2 {
                    op1.clone()
                } else {
                    // Shift position if second operation inserts before first
                    Operation::Insert {
                        position: pos1 + 1,
                        text: String::new(), // Will be filled by caller
                        timestamp: Utc::now(),
                        user_id: String::new(), // Will be filled by caller
                    }
                }
            }
            (Operation::Insert { position: pos1, .. }, Operation::Delete { position: pos2, length, .. }) => {
                if pos1 < pos2 {
                    op1.clone()
                } else if pos1 >= pos2 + length {
                    Operation::Insert {
                        position: pos1 - length,
                        text: String::new(), // Will be filled by caller
                        timestamp: Utc::now(),
                        user_id: String::new(), // Will be filled by caller
                    }
                } else {
                    // Insert position is within deleted range, adjust to after deletion
                    Operation::Insert {
                        position: pos2,
                        text: String::new(), // Will be filled by caller
                        timestamp: Utc::now(),
                        user_id: String::new(), // Will be filled by caller
                    }
                }
            }
            (Operation::Delete { position: pos1, length: len1, .. }, Operation::Insert { position: pos2, .. }) => {
                if pos1 <= pos2 {
                    op1.clone()
                } else {
                    // Shift position if second operation inserts before first
                    Operation::Delete {
                        position: pos1 + 1,
                        length: len1,
                        timestamp: Utc::now(),
                        user_id: String::new(), // Will be filled by caller
                    }
                }
            }
            (Operation::Delete { position: pos1, length: len1, .. }, Operation::Delete { position: pos2, length: len2, .. }) => {
                if pos1 + len1 <= pos2 {
                    op1.clone()
                } else if pos2 + len2 <= pos1 {
                    Operation::Delete {
                        position: pos1 - len2,
                        length: len1,
                        timestamp: Utc::now(),
                        user_id: String::new(), // Will be filled by caller
                    }
                } else {
                    // Overlapping deletions
                    let start = pos1.min(pos2);
                    let end = (pos1 + len1).max(pos2 + len2);
                    Operation::Delete {
                        position: start,
                        length: end - start,
                        timestamp: Utc::now(),
                        user_id: String::new(), // Will be filled by caller
                    }
                }
            }
            _ => op1.clone(),
        }
    }

    /// Apply operation to text
    pub fn apply_operation(text: &str, operation: &Operation) -> String {
        match operation {
            Operation::Insert { position, text: insert_text, .. } => {
                let mut result = text.to_string();
                result.insert_str(*position, insert_text);
                result
            }
            Operation::Delete { position, length, .. } => {
                let mut result = text.to_string();
                result.drain(*position..(*position + *length));
                result
            }
            Operation::Replace { position, old_length, new_text, .. } => {
                let mut result = text.to_string();
                result.drain(*position..(*position + *old_length));
                result.insert_str(*position, new_text);
                result
            }
        }
    }
}
