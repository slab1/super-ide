//! Event bus for inter-component communication in Super IDE

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, oneshot};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use once_cell::sync::Lazy;

/// Event types for the IDE
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IdeEvent {
    /// File system events
    FileChanged {
        path: String,
        event_type: FileEventType,
    },
    
    /// Editor events
    EditorOpened {
        document_id: String,
        file_path: String,
    },
    EditorClosed {
        document_id: String,
    },
    CodeChanged {
        document_id: String,
        content: String,
        position: (usize, usize),
    },
    
    /// AI events
    AiSuggestion {
        document_id: String,
        suggestion: String,
        confidence: f32,
    },
    AiCompleted {
        request_id: String,
        result: AiResult,
    },
    
    /// Collaboration events
    UserJoined {
        user_id: String,
        username: String,
    },
    UserLeft {
        user_id: String,
    },
    CursorMoved {
        user_id: String,
        document_id: String,
        position: (usize, usize),
    },
    
    /// System events
    Startup,
    Shutdown,
    Error {
        source: String,
        message: String,
    },
    ConfigurationChanged {
        key: String,
        value: serde_json::Value,
    },
}

/// File event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileEventType {
    Created,
    Modified,
    Deleted,
    Renamed {
        from: String,
        to: String,
    },
}

/// AI operation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AiResult {
    Completion {
        text: String,
    },
    Analysis {
        report: String,
    },
    Error {
        message: String,
    },
}

/// Event bus error types
#[derive(Error, Debug)]
pub enum EventBusError {
    #[error("Channel closed")]
    ChannelClosed,
    
    #[error("Timeout waiting for response")]
    Timeout,
    
    #[error("Invalid event type")]
    InvalidEvent,
    
    #[error("Mutex poisoned: {0}")]
    MutexPoisoned(String),
    
    #[error("Channel not found: {0}")]
    ChannelNotFound(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Event subscriber handle
pub struct EventSubscriber {
    receiver: broadcast::Receiver<IdeEvent>,
    _id: String, // Keep alive
}

/// Event broadcaster
pub struct EventBroadcaster {
    sender: broadcast::Sender<IdeEvent>,
}

impl EventBroadcaster {
    /// Create a new event broadcaster
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Send an event
    pub fn send(&self, event: IdeEvent) -> Result<usize, broadcast::error::SendError<IdeEvent>> {
        self.sender.send(event)
    }

    /// Get the number of receivers
    pub fn receiver_count(&self) -> usize {
        self.sender.receiver_count()
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<IdeEvent> {
        self.sender.subscribe()
    }
}

/// Main event bus
#[derive(Clone)]
#[derive(Debug)]
pub struct EventBus {
    channels: Arc<std::sync::Mutex<HashMap<String, broadcast::Sender<IdeEvent>>>>,
    request_channels: Arc<std::sync::Mutex<HashMap<String, mpsc::UnboundedSender<EventRequest>>>>,
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        Self {
            channels: Arc::new(std::sync::Mutex::new(HashMap::new())),
            request_channels: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }
    
    /// Subscribe to events on a channel
    pub fn subscribe(&self, channel: &str) -> Result<EventSubscriber, EventBusError> {
        let mut channels = self.channels.lock()
            .map_err(|e| EventBusError::MutexPoisoned("channels".to_string()))?;
        
        if !channels.contains_key(channel) {
            let (sender, _) = broadcast::channel(100);
            channels.insert(channel.to_string(), sender);
        }
        
        let sender = channels.get(channel)
            .cloned()
            .ok_or_else(|| EventBusError::ChannelNotFound(channel.to_string()))?;
        let receiver = sender.subscribe();
        
        Ok(EventSubscriber {
            receiver,
            _id: format!("subscriber_{}", uuid::Uuid::new_v4()),
        })
    }
    
    /// Publish an event to a channel
    pub fn publish(&self, channel: &str, event: IdeEvent) -> Result<(), EventBusError> {
        let channels = self.channels.lock()
            .map_err(|e| EventBusError::MutexPoisoned("channels".to_string()))?;
        
        if let Some(sender) = channels.get(channel) {
            if sender.send(event).is_err() {
                return Err(EventBusError::ChannelClosed);
            }
        }
        
        Ok(())
    }
    
    /// Publish event to all subscribers
    pub fn broadcast(&self, event: IdeEvent) -> Result<(), EventBusError> {
        let channels = self.channels.lock()
            .map_err(|e| EventBusError::MutexPoisoned("channels".to_string()))?;
        
        for (_, sender) in channels.iter() {
            if sender.send(event.clone()).is_err() {
                // Channel closed, continue with others
                continue;
            }
        }
        
        Ok(())
    }
    
    /// Create a request-response channel
    pub fn create_request_channel(&self, name: &str) -> EventRequestHandler {
        let (sender, mut receiver) = mpsc::unbounded_channel::<EventRequest>();
        
        // Handle mutex lock failure gracefully
        if let Ok(mut request_channels) = self.request_channels.lock() {
            request_channels.insert(name.to_string(), sender);
        } else {
            e!("Failed to acquire request channels lock for '{}'", name);
        }
        
        // Start background task to handle requests
        tokio::spawn(async move {
            while let Some(request) = receiver.recv().await {
                match request {
                    EventRequest::GetDocuments => {
                        // Would query the actual document list
                        let response = EventResponse::Documents {
                            documents: vec![],
                        };
                        let _ = request.respond_with(response);
                    }
                    EventRequest::GetActiveDocument => {
                        let response = EventResponse::ActiveDocument {
                            document_id: None,
                        };
                        let _ = request.respond_with(response);
                    }
                    EventRequest::ExecuteCommand { ref command, .. } => {
                        let response = EventResponse::CommandResult {
                            success: false,
                            output: format!("Command '{}' not implemented", command),
                        };
                        let _ = request.respond_with(response);
                    }
                    // Handle remaining variants
                    _ => {
                        let response = EventResponse::Error {
                            message: "Unhandled request type".to_string(),
                        };
                        let _ = request.respond_with(response);
                    }
                }
            }
        });
        
        EventRequestHandler {
            name: name.to_string(),
        }
    }
    
    /// Send a request and wait for response
    pub async fn send_request<R: Into<EventRequest>>(
        &self,
        name: &str,
        request: R,
    ) -> Result<EventResponse, EventBusError> {
        let request_channels = self.request_channels.lock()
            .map_err(|e| EventBusError::MutexPoisoned("request_channels".to_string()))?;
        
        if let Some(sender) = request_channels.get(name) {
            let (response_sender, response_receiver) = oneshot::channel::<EventResponse>();
            
            let wrapped_request = request.into().with_response_sender(response_sender);
            
            if sender.send(wrapped_request).is_err() {
                return Err(EventBusError::ChannelClosed);
            }
            
            tokio::time::timeout(std::time::Duration::from_secs(5), response_receiver)
                .await
                .map_err(|_| EventBusError::Timeout)?
                .map_err(|_| EventBusError::ChannelClosed)
        } else {
            Err(EventBusError::InvalidEvent)
        }
    }
}

/// Request handler for a specific channel
pub struct EventRequestHandler {
    name: String,
}

impl EventRequestHandler {
    /// Get the handler name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Check if this handler matches a name
    pub fn matches(&self, name: &str) -> bool {
        self.name == name
    }
}

/// Event request types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventRequest {
    GetDocuments,
    GetActiveDocument,
    ExecuteCommand {
        command: String,
        args: Vec<String>,
    },
    SaveDocument {
        document_id: String,
    },
    OpenFile {
        path: String,
    },
    CloseDocument {
        document_id: String,
    },
    AIAnalyze {
        document_id: String,
        code: String,
        language: String,
    },
}

impl EventRequest {
    /// Attach a response sender to this request
    fn with_response_sender(self, _sender: oneshot::Sender<EventResponse>) -> Self {
        // This is a workaround since we can't modify after move
        self
    }
}

/// Event response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventResponse {
    Documents {
        documents: Vec<String>, // Document IDs
    },
    ActiveDocument {
        document_id: Option<String>,
    },
    CommandResult {
        success: bool,
        output: String,
    },
    Error {
        message: String,
    },
    SaveResult {
        success: bool,
        error: Option<String>,
    },
    OpenResult {
        success: bool,
        document_id: Option<String>,
        error: Option<String>,
    },
    CloseResult {
        success: bool,
    },
    AIAnalysisResult {
        success: bool,
        analysis: Option<String>,
        error: Option<String>,
    },
}

/// Event subscriber extension for common event types
impl EventSubscriber {
    /// Wait for the next event of a specific type
    pub async fn next_of_type<T>(&mut self) -> Option<T>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        while let Ok(event) = self.receiver.recv().await {
            if let Ok(serialized) = serde_json::to_value(event)
                .map_err(|e| eprintln!("Serialization error: {}", e))
            {
                if let Ok(deserialized) = serde_json::from_value::<T>(serialized) {
                    return Some(deserialized);
                }
            }
        }
        None
    }
    
    /// Stream events of a specific type
    pub async fn stream_of_type<T>(&self, mut _receiver: mpsc::UnboundedReceiver<IdeEvent>) -> mpsc::UnboundedReceiver<T>
    where
        T: for<'de> serde::Deserialize<'de> + Send + 'static,
    {
        let (sender, receiver) = mpsc::unbounded_channel::<T>();
        
        tokio::spawn(async move {
            while let Some(event) = _receiver.recv().await {
                if let Ok(serialized) = serde_json::to_value(event)
                    .map_err(|e| eprintln!("Serialization error: {}", e))
                {
                    if let Ok(deserialized) = serde_json::from_value::<T>(serialized) {
                        let _ = sender.send(deserialized);
                    }
                }
            }
        });
        
        receiver
    }
}

// Global event bus instance
static GLOBAL_EVENT_BUS: Lazy<EventBus> = Lazy::new(|| EventBus::new());

/// Get the global event bus instance
pub fn global_event_bus() -> &'static EventBus {
    &GLOBAL_EVENT_BUS
}

// Implement response sender attachment for EventRequest
impl EventRequest {
    fn respond_with(self, _response: EventResponse) {
        // This is a placeholder - in a real implementation,
        // you'd store the response sender in the request
        // and have the handler use it
    }
}
