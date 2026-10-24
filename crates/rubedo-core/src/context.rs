use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::reagent::ReagentError;

/// Role indicator for conversation participants.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageRole {
    /// System-level instructional directive.
    System,
    /// Human user prompt message.
    User,
    /// AI assistant model completion message.
    Assistant,
    /// Executable tool result message.
    Tool,
}

/// A single message in the agent conversation trajectory.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    /// The participant role.
    pub role: MessageRole,
    /// Text content of the message.
    pub content: String,
    /// Optional tool identifier if this is a tool response message.
    pub tool_name: Option<String>,
    /// Creation timestamp in milliseconds since UNIX epoch.
    pub timestamp_ms: u64,
}

impl Message {
    /// Construct a new message with current timestamp.
    pub fn new(role: MessageRole, content: impl Into<String>) -> Self {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        Self {
            role,
            content: content.into(),
            tool_name: None,
            timestamp_ms,
        }
    }

    /// Construct a tool response message.
    pub fn tool_response(tool_name: impl Into<String>, content: impl Into<String>) -> Self {
        let mut msg = Self::new(MessageRole::Tool, content);
        msg.tool_name = Some(tool_name.into());
        msg
    }
}

/// Token accounting metrics for prompt generation and response distillation.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenUsage {
    /// Number of prompt/context tokens consumed.
    pub prompt_tokens: u64,
    /// Number of completion tokens generated.
    pub completion_tokens: u64,
    /// Cumulative sum of prompt and completion tokens.
    pub total_tokens: u64,
}

impl TokenUsage {
    /// Accumulate token counts.
    pub fn record(&mut self, prompt: u64, completion: u64) {
        self.prompt_tokens += prompt;
        self.completion_tokens += completion;
        self.total_tokens = self.prompt_tokens + self.completion_tokens;
    }
}

/// Journal entry recording an individual Reagent invocation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolExecutionRecord {
    /// Name of the invoked reagent.
    pub tool_name: String,
    /// Input arguments passed to the reagent.
    pub input: Value,
    /// Output result or error payload.
    pub output: Result<Value, ReagentError>,
    /// Execution duration in milliseconds.
    pub duration_ms: u64,
    /// Timestamp when invocation was recorded.
    pub timestamp_ms: u64,
}

/// The alchemical furnace (*Athanor*) context providing uninterrupted digestion.
///
/// Houses conversation messages, scratchpad memory variables, cumulative token metrics,
/// execution journals, and cancellation handles across asynchronous agent steps.
#[derive(Clone)]
pub struct AthanorContext {
    messages: Vec<Message>,
    variables: HashMap<String, Value>,
    tool_executions: Vec<ToolExecutionRecord>,
    token_usage: TokenUsage,
    cancellation_flag: Arc<AtomicBool>,
}

impl std::fmt::Debug for AthanorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AthanorContext")
            .field("messages_count", &self.messages.len())
            .field("variables_keys", &self.variables.keys().collect::<Vec<_>>())
            .field("tool_executions_count", &self.tool_executions.len())
            .field("token_usage", &self.token_usage)
            .field("is_cancelled", &self.is_cancelled())
            .finish()
    }
}

impl Default for AthanorContext {
    fn default() -> Self {
        Self::new()
    }
}

impl AthanorContext {
    /// Initialize a pristine, empty context.
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            variables: HashMap::new(),
            tool_executions: Vec::new(),
            token_usage: TokenUsage::default(),
            cancellation_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Initialize a context seeded with a system prompt.
    pub fn with_system_prompt(prompt: impl Into<String>) -> Self {
        let mut ctx = Self::new();
        ctx.add_system_message(prompt);
        ctx
    }

    /// Append a system message.
    pub fn add_system_message(&mut self, content: impl Into<String>) {
        self.messages.push(Message::new(MessageRole::System, content));
    }

    /// Append a user prompt message.
    pub fn add_user_message(&mut self, content: impl Into<String>) {
        self.messages.push(Message::new(MessageRole::User, content));
    }

    /// Append an assistant model message.
    pub fn add_assistant_message(&mut self, content: impl Into<String>) {
        self.messages.push(Message::new(MessageRole::Assistant, content));
    }

    /// Append a tool execution result.
    pub fn add_tool_result(&mut self, tool_name: impl Into<String>, result: Value) {
        self.messages.push(Message::tool_response(tool_name, result.to_string()));
    }

    /// Get an immutable slice of all messages in history.
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    /// Clear all conversation history.
    pub fn clear_messages(&mut self) {
        self.messages.clear();
    }

    /// Store a scratchpad variable.
    pub fn set_var(&mut self, key: impl Into<String>, value: Value) {
        self.variables.insert(key.into(), value);
    }

    /// Retrieve a reference to a stored scratchpad variable.
    pub fn get_var(&self, key: &str) -> Option<&Value> {
        self.variables.get(key)
    }

    /// Remove a scratchpad variable.
    pub fn remove_var(&mut self, key: &str) -> Option<Value> {
        self.variables.remove(key)
    }

    /// Check if a variable key is defined.
    pub fn has_var(&self, key: &str) -> bool {
        self.variables.contains_key(key)
    }

    /// Record a reagent execution event into the journal.
    pub fn record_tool_call(
        &mut self,
        tool_name: impl Into<String>,
        input: Value,
        output: Result<Value, ReagentError>,
        duration_ms: u64,
    ) {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        self.tool_executions.push(ToolExecutionRecord {
            tool_name: tool_name.into(),
            input,
            output,
            duration_ms,
            timestamp_ms,
        });
    }

    /// Access the journal of executed tool calls.
    pub fn tool_executions(&self) -> &[ToolExecutionRecord] {
        &self.tool_executions
    }

    /// Accumulate token usage metrics.
    pub fn record_tokens(&mut self, prompt: u64, completion: u64) {
        self.token_usage.record(prompt, completion);
    }

    /// Current cumulative token usage.
    pub fn token_usage(&self) -> TokenUsage {
        self.token_usage
    }

    /// Request cooperative cancellation of ongoing operations.
    pub fn cancel(&self) {
        self.cancellation_flag.store(true, Ordering::SeqCst);
    }

    /// Check if cancellation has been requested.
    pub fn is_cancelled(&self) -> bool {
        self.cancellation_flag.load(Ordering::SeqCst)
    }
}
