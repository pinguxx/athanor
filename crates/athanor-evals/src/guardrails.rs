//! Token budget guardrails and resource watchdogs.

use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::EvalsError;

/// Point-in-time snapshot of cumulative token consumption.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenUsageSnapshot {
    /// Total prompt tokens consumed.
    pub prompt_tokens: u64,
    /// Total completion tokens generated.
    pub completion_tokens: u64,
    /// Combined aggregate tokens used.
    pub total_tokens: u64,
}

/// Configuration settings for the token budget watchdog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenBudgetConfig {
    /// Absolute ceiling on aggregate total token consumption.
    pub max_total_tokens: u64,
    /// Optional limit on input prompt tokens.
    pub max_prompt_tokens: Option<u64>,
    /// Optional limit on output completion tokens.
    pub max_completion_tokens: Option<u64>,
}

impl Default for TokenBudgetConfig {
    fn default() -> Self {
        Self {
            max_total_tokens: 100_000,
            max_prompt_tokens: None,
            max_completion_tokens: None,
        }
    }
}

/// Thread-safe watchdog tracking LLM token consumption and preventing budget overruns.
pub struct TokenBudgetGuard {
    config: TokenBudgetConfig,
    prompt_tokens: AtomicU64,
    completion_tokens: AtomicU64,
}

impl TokenBudgetGuard {
    /// Construct a new `TokenBudgetGuard` with a specified total token limit.
    pub fn new(max_total_tokens: u64) -> Self {
        Self {
            config: TokenBudgetConfig {
                max_total_tokens,
                max_prompt_tokens: None,
                max_completion_tokens: None,
            },
            prompt_tokens: AtomicU64::new(0),
            completion_tokens: AtomicU64::new(0),
        }
    }

    /// Construct a guard with full configuration limits.
    pub fn with_config(config: TokenBudgetConfig) -> Self {
        Self {
            config,
            prompt_tokens: AtomicU64::new(0),
            completion_tokens: AtomicU64::new(0),
        }
    }

    /// Record a consumption event. Returns an error if any configured budget is exceeded.
    pub fn record(&self, prompt: u64, completion: u64) -> Result<TokenUsageSnapshot, EvalsError> {
        let current_prompt = self.prompt_tokens.load(Ordering::Relaxed);
        let current_completion = self.completion_tokens.load(Ordering::Relaxed);

        let new_prompt = current_prompt.saturating_add(prompt);
        let new_completion = current_completion.saturating_add(completion);
        let new_total = new_prompt.saturating_add(new_completion);

        // 1. Check prompt token limit
        if let Some(max_prompt) = self.config.max_prompt_tokens {
            if new_prompt > max_prompt {
                return Err(EvalsError::TokenBudgetExceeded {
                    prompt_tokens: new_prompt,
                    completion_tokens: new_completion,
                    used: new_prompt,
                    limit: max_prompt,
                });
            }
        }

        // 2. Check completion token limit
        if let Some(max_completion) = self.config.max_completion_tokens {
            if new_completion > max_completion {
                return Err(EvalsError::TokenBudgetExceeded {
                    prompt_tokens: new_prompt,
                    completion_tokens: new_completion,
                    used: new_completion,
                    limit: max_completion,
                });
            }
        }

        // 3. Check total token ceiling
        if new_total > self.config.max_total_tokens {
            return Err(EvalsError::TokenBudgetExceeded {
                prompt_tokens: new_prompt,
                completion_tokens: new_completion,
                used: new_total,
                limit: self.config.max_total_tokens,
            });
        }

        // Atomically store new consumption tallies
        self.prompt_tokens.store(new_prompt, Ordering::SeqCst);
        self.completion_tokens
            .store(new_completion, Ordering::SeqCst);

        Ok(TokenUsageSnapshot {
            prompt_tokens: new_prompt,
            completion_tokens: new_completion,
            total_tokens: new_total,
        })
    }

    /// Return current token usage snapshot.
    pub fn usage(&self) -> TokenUsageSnapshot {
        let p = self.prompt_tokens.load(Ordering::Relaxed);
        let c = self.completion_tokens.load(Ordering::Relaxed);
        TokenUsageSnapshot {
            prompt_tokens: p,
            completion_tokens: c,
            total_tokens: p.saturating_add(c),
        }
    }

    /// Calculate remaining tokens available under total budget.
    pub fn remaining_tokens(&self) -> u64 {
        let used = self.usage().total_tokens;
        self.config.max_total_tokens.saturating_sub(used)
    }

    /// Reset all cumulative token counters to zero.
    pub fn reset(&self) {
        self.prompt_tokens.store(0, Ordering::SeqCst);
        self.completion_tokens.store(0, Ordering::SeqCst);
    }
}
