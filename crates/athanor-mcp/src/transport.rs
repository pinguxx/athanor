//! Transport abstractions and implementations for Model Context Protocol communication.

use async_trait::async_trait;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Stdin, Stdout};
use tokio::sync::{mpsc, Mutex};

use crate::error::McpError;

/// Asynchronous transport interface for bidirectional JSON-RPC message exchange.
#[async_trait]
pub trait Transport: Send + Sync + 'static {
    /// Transmit a framed message string to the peer.
    async fn send(&self, msg: String) -> Result<(), McpError>;

    /// Receive the next framed message string from the peer.
    ///
    /// Returns `Ok(Some(msg))` when a message arrives, or `Ok(None)` if the
    /// transport was closed or reached end-of-stream.
    async fn receive(&self) -> Result<Option<String>, McpError>;

    /// Gracefully terminate or flush the transport.
    async fn close(&self) -> Result<(), McpError> {
        Ok(())
    }
}

/// In-memory duplex transport using `tokio::sync::mpsc` channels.
///
/// Designed for zero-overhead in-process communication and unit/integration testing.
pub struct ChannelTransport {
    sender: Mutex<Option<mpsc::Sender<String>>>,
    receiver: Mutex<mpsc::Receiver<String>>,
}

impl ChannelTransport {
    /// Construct a transport wrapping an existing sender and receiver pair.
    pub fn new(sender: mpsc::Sender<String>, receiver: mpsc::Receiver<String>) -> Self {
        Self {
            sender: Mutex::new(Some(sender)),
            receiver: Mutex::new(receiver),
        }
    }

    /// Construct a linked pair of duplex channel transports with the given queue capacity.
    ///
    /// Messages sent into side A will be received on side B, and vice versa.
    pub fn create_duplex(capacity: usize) -> (Self, Self) {
        let (tx1, rx1) = mpsc::channel(capacity);
        let (tx2, rx2) = mpsc::channel(capacity);

        let side_a = Self::new(tx1, rx2);
        let side_b = Self::new(tx2, rx1);

        (side_a, side_b)
    }
}

#[async_trait]
impl Transport for ChannelTransport {
    async fn send(&self, msg: String) -> Result<(), McpError> {
        let sender_opt = {
            let guard = self.sender.lock().await;
            guard.clone()
        };

        match sender_opt {
            Some(sender) => sender
                .send(msg)
                .await
                .map_err(|e| McpError::Transport(e.to_string())),
            None => Err(McpError::Transport(
                "Cannot send over closed transport".to_string(),
            )),
        }
    }

    async fn receive(&self) -> Result<Option<String>, McpError> {
        let mut rx = self.receiver.lock().await;
        Ok(rx.recv().await)
    }

    async fn close(&self) -> Result<(), McpError> {
        {
            let mut tx = self.sender.lock().await;
            tx.take();
        }
        {
            let mut rx = self.receiver.lock().await;
            rx.close();
        }
        Ok(())
    }
}

/// Standard I/O transport communicating via newline-delimited JSON-RPC over `stdin` and `stdout`.
///
/// Compliant with Claude Desktop and Cursor process-based MCP communication.
pub struct StdioTransport {
    reader: Arc<Mutex<BufReader<Stdin>>>,
    writer: Arc<Mutex<Stdout>>,
}

impl StdioTransport {
    /// Create a new `StdioTransport` bound to the current process's standard input and output streams.
    pub fn new() -> Self {
        Self {
            reader: Arc::new(Mutex::new(BufReader::new(tokio::io::stdin()))),
            writer: Arc::new(Mutex::new(tokio::io::stdout())),
        }
    }
}

impl Default for StdioTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Transport for StdioTransport {
    async fn send(&self, msg: String) -> Result<(), McpError> {
        let mut writer = self.writer.lock().await;
        writer.write_all(msg.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;
        Ok(())
    }

    async fn receive(&self) -> Result<Option<String>, McpError> {
        let mut reader = self.reader.lock().await;
        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            return Ok(None);
        }

        // Strip trailing carriage return and/or newline
        if line.ends_with('\n') {
            line.pop();
            if line.ends_with('\r') {
                line.pop();
            }
        }

        Ok(Some(line))
    }
}

#[async_trait]
impl<T: Transport + ?Sized> Transport for Arc<T> {
    async fn send(&self, msg: String) -> Result<(), McpError> {
        (**self).send(msg).await
    }

    async fn receive(&self) -> Result<Option<String>, McpError> {
        (**self).receive().await
    }

    async fn close(&self) -> Result<(), McpError> {
        (**self).close().await
    }
}
