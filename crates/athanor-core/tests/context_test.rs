use athanor_core::context::{AthanorContext, MessageRole};
use athanor_core::reagent::ReagentError;
use serde_json::json;

#[test]
fn test_athanor_context_message_history() {
    let mut ctx = AthanorContext::with_system_prompt("System rules");
    assert_eq!(ctx.messages().len(), 1);
    assert_eq!(ctx.messages()[0].role, MessageRole::System);
    assert_eq!(ctx.messages()[0].content, "System rules");

    ctx.add_user_message("Hello world");
    ctx.add_assistant_message("Greetings, seeker");
    ctx.add_tool_result("calculator", json!({ "sum": 42 }));

    assert_eq!(ctx.messages().len(), 4);
    assert_eq!(ctx.messages()[1].role, MessageRole::User);
    assert_eq!(ctx.messages()[2].role, MessageRole::Assistant);
    assert_eq!(ctx.messages()[3].role, MessageRole::Tool);
    assert_eq!(ctx.messages()[3].tool_name, Some("calculator".into()));

    ctx.clear_messages();
    assert_eq!(ctx.messages().len(), 0);
}

#[test]
fn test_athanor_context_variables() {
    let mut ctx = AthanorContext::new();
    assert!(!ctx.has_var("stage"));

    ctx.set_var("stage", json!("athanor"));
    assert!(ctx.has_var("stage"));
    assert_eq!(ctx.get_var("stage"), Some(&json!("athanor")));

    let removed = ctx.remove_var("stage");
    assert_eq!(removed, Some(json!("athanor")));
    assert!(!ctx.has_var("stage"));
}

#[test]
fn test_athanor_context_tool_journal_and_tokens() {
    let mut ctx = AthanorContext::new();

    ctx.record_tool_call(
        "weather_check",
        json!({ "city": "Prague" }),
        Ok(json!({ "temp": 18 })),
        45,
    );
    ctx.record_tool_call(
        "failing_tool",
        json!({}),
        Err(ReagentError::fatal("Network timeout")),
        120,
    );

    assert_eq!(ctx.tool_executions().len(), 2);
    assert_eq!(ctx.tool_executions()[0].tool_name, "weather_check");
    assert_eq!(ctx.tool_executions()[0].duration_ms, 45);
    assert!(ctx.tool_executions()[0].output.is_ok());

    assert_eq!(ctx.tool_executions()[1].tool_name, "failing_tool");
    assert!(ctx.tool_executions()[1].output.is_err());

    // Token accounting
    assert_eq!(ctx.token_usage().total_tokens, 0);
    ctx.record_tokens(150, 80);
    assert_eq!(ctx.token_usage().prompt_tokens, 150);
    assert_eq!(ctx.token_usage().completion_tokens, 80);
    assert_eq!(ctx.token_usage().total_tokens, 230);

    ctx.record_tokens(50, 20);
    assert_eq!(ctx.token_usage().total_tokens, 300);
}

#[test]
fn test_athanor_context_cancellation() {
    let ctx = AthanorContext::new();
    assert!(!ctx.is_cancelled());

    let ctx_clone = ctx.clone();
    ctx.cancel();
    assert!(ctx.is_cancelled());
    assert!(ctx_clone.is_cancelled());
}

#[test]
fn test_athanor_context_debug() {
    let mut ctx = AthanorContext::new();
    ctx.add_user_message("test");
    ctx.set_var("key", json!(123));
    let debug_str = format!("{:?}", ctx);
    assert!(debug_str.contains("messages_count: 1"));
    assert!(debug_str.contains("key"));
}
