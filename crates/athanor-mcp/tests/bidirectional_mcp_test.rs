use athanor_core::artificer::Artificer;
use athanor_core::reagent::{Reagent, ReagentError};
use athanor_macros::reagent;
use athanor_mcp::prelude::*;
use athanor_mcp::protocol::{JsonRpcRequest, RequestId};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct CalcOutput {
    sum: i64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct DivOutput {
    quotient: f64,
}

/// A test calculator reagent adding two integers.
#[reagent(name = "calculator")]
async fn calculator(
    /// First operand
    a: i64,
    /// Second operand
    b: i64,
) -> Result<CalcOutput, ReagentError> {
    Ok(CalcOutput { sum: a + b })
}

/// A division tool that returns an error when dividing by zero.
#[reagent(name = "divider")]
async fn divider(
    /// Numerator
    numerator: f64,
    /// Denominator
    denominator: f64,
) -> Result<DivOutput, ReagentError> {
    if denominator == 0.0 {
        return Err(ReagentError::invalid_input("Denominator cannot be zero"));
    }
    Ok(DivOutput {
        quotient: numerator / denominator,
    })
}

#[tokio::test]
async fn test_bidirectional_mcp_flow() {
    // 1. Create in-memory duplex channel transport
    let (server_transport, client_transport) = ChannelTransport::create_duplex(64);

    // 2. Instantiate and configure McpServer with native Reagent
    let server = Arc::new(McpServer::new("athanor-mcp-server", "0.1.0"));
    server.register_reagent(Arc::new(CalculatorReagent)).await;

    // 3. Spawn McpServer loop in the background
    let server_clone = Arc::clone(&server);
    let server_handle =
        tokio::spawn(async move { server_clone.run(Arc::new(server_transport)).await });

    // 4. Instantiate McpClient
    let client = McpClient::new(Arc::new(client_transport));

    // 5. Initialize handshake
    let init_res = client
        .initialize("athanor-mcp-client", "0.1.0")
        .await
        .expect("Handshake initialization should succeed");
    assert_eq!(init_res.protocol_version, PROTOCOL_VERSION);
    assert_eq!(init_res.server_info.name, "athanor-mcp-server");

    // 6. Discover tools via tools/list
    let tools = client
        .list_tools()
        .await
        .expect("tools/list RPC should succeed");
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "calculator");
    assert!(tools[0].input_schema["properties"]["a"].is_object());
    assert!(tools[0].input_schema["properties"]["b"].is_object());

    // 7. Invoke tool via client.call_tool()
    let result = client
        .call_tool("calculator", json!({ "a": 15, "b": 27 }))
        .await
        .expect("tools/call RPC should succeed");
    assert_eq!(result, json!({ "sum": 42 }));

    // 8. Discover and synthesize dynamic Reagent instances
    let discovered_reagents = client
        .discover_reagents()
        .await
        .expect("Dynamic reagent discovery should succeed");
    assert_eq!(discovered_reagents.len(), 1);

    let dyn_reagent = Arc::clone(&discovered_reagents[0]);
    assert_eq!(dyn_reagent.name(), "calculator");

    let transmute_res = dyn_reagent
        .transmute(json!({ "a": 100, "b": 200 }))
        .await
        .expect("Dynamic reagent transmute should succeed");
    assert_eq!(transmute_res, json!({ "sum": 300 }));

    // 9. Wire dynamic Reagent into an Artificer
    let artificer = Artificer::builder()
        .name("grand-alchemist")
        .model("claude-3-5-sonnet")
        .reagent(dyn_reagent)
        .build()
        .expect("Artificer build should succeed");

    assert!(artificer.has_reagent("calculator"));
    let artificer_tool = artificer
        .get_reagent("calculator")
        .expect("Artificer should contain calculator tool");

    let artificer_call_res = artificer_tool
        .transmute(json!({ "a": 50, "b": -10 }))
        .await
        .expect("Artificer tool transmute should succeed");
    assert_eq!(artificer_call_res, json!({ "sum": 40 }));

    // Cleanup server
    client.close().await.expect("Client close should succeed");
    let _ = server_handle.await;
}

#[tokio::test]
async fn test_tool_execution_error_and_not_found() {
    let (server_transport, client_transport) = ChannelTransport::create_duplex(32);

    let server = Arc::new(McpServer::new("error-server", "1.0.0"));
    server.register_reagent(Arc::new(DividerReagent)).await;

    let server_clone = Arc::clone(&server);
    tokio::spawn(async move {
        let _ = server_clone.run(Arc::new(server_transport)).await;
    });

    let client = McpClient::new(Arc::new(client_transport));
    client
        .initialize("error-client", "1.0.0")
        .await
        .expect("Initialize should succeed");

    // Success call
    let success = client
        .call_tool("divider", json!({ "numerator": 10.0, "denominator": 2.0 }))
        .await
        .expect("Division should succeed");
    assert_eq!(success, json!({ "quotient": 5.0 }));

    // Failing call (division by zero)
    let fail_res = client
        .call_tool("divider", json!({ "numerator": 10.0, "denominator": 0.0 }))
        .await;
    assert!(fail_res.is_err());
    let err = fail_res.unwrap_err();
    assert!(err.to_string().contains("Denominator cannot be zero"));

    // Call unknown tool
    let not_found_res = client.call_tool("nonexistent", json!({})).await;
    assert!(not_found_res.is_err());
    let err = not_found_res.unwrap_err();
    assert!(err.to_string().contains("Tool 'nonexistent' not found"));
}

#[tokio::test]
async fn test_server_message_parsing_and_ping() {
    let server = McpServer::new("ping-server", "1.0.0");

    // Invalid JSON
    let parse_err_resp = server
        .handle_message("invalid json content")
        .await
        .expect("handle_message should handle parse error")
        .expect("Response expected for parse error");
    assert!(parse_err_resp.contains("-32700"));

    // Ping request
    let ping_req =
        serde_json::to_string(&JsonRpcRequest::new(RequestId::Number(99), "ping", None)).unwrap();
    let ping_resp = server
        .handle_message(&ping_req)
        .await
        .expect("handle_message should handle ping")
        .expect("Response expected for ping");
    assert!(ping_resp.contains("\"result\":{}"));

    // Notification (no id)
    let notif = serde_json::to_string(&JsonRpcRequest::notification(
        "notifications/initialized",
        None,
    ))
    .unwrap();
    let notif_resp = server
        .handle_message(&notif)
        .await
        .expect("Notification should succeed");
    assert!(notif_resp.is_none());

    // Unknown method
    let unknown_req = serde_json::to_string(&JsonRpcRequest::new(
        RequestId::Number(100),
        "unknown_custom_method",
        None,
    ))
    .unwrap();
    let unknown_resp = server
        .handle_message(&unknown_req)
        .await
        .expect("handle_message should handle unknown method")
        .expect("Response expected");
    assert!(unknown_resp.contains("-32601"));

    // Missing params on tools/call
    let missing_params_req = serde_json::to_string(&JsonRpcRequest::new(
        RequestId::Number(101),
        "tools/call",
        None,
    ))
    .unwrap();
    let missing_params_resp = server
        .handle_message(&missing_params_req)
        .await
        .expect("handle_message should handle missing params")
        .expect("Response expected");
    assert!(missing_params_resp.contains("-32602"));

    // Invalid params type on tools/call
    let invalid_params_req = serde_json::to_string(&JsonRpcRequest::new(
        RequestId::Number(102),
        "tools/call",
        Some(json!("not_an_object")),
    ))
    .unwrap();
    let invalid_params_resp = server
        .handle_message(&invalid_params_req)
        .await
        .expect("handle_message should handle invalid params")
        .expect("Response expected");
    assert!(invalid_params_resp.contains("-32602"));
}

#[test]
fn test_mcp_error_conversions() {
    let err_tool = McpError::ToolExecution("fatal failure".into());
    let reagent_err: ReagentError = err_tool.into();
    assert!(reagent_err.is_fatal());

    let err_rpc_invalid = McpError::JsonRpc {
        code: -32602,
        message: "bad args".into(),
    };
    let reagent_err: ReagentError = err_rpc_invalid.into();
    assert!(reagent_err.is_invalid_input());

    let err_rpc_other = McpError::JsonRpc {
        code: -32603,
        message: "crash".into(),
    };
    let reagent_err: ReagentError = err_rpc_other.into();
    assert!(reagent_err.is_fatal());

    let serde_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
    let err_ser = McpError::Serialization(serde_err);
    let reagent_err: ReagentError = err_ser.into();
    assert!(reagent_err.is_invalid_input());

    let err_trans = McpError::Transport("connection reset".into());
    let reagent_err: ReagentError = err_trans.into();
    assert!(reagent_err.is_fatal());
}

#[test]
fn test_protocol_types_and_constructors() {
    let id_num = RequestId::from(100i64);
    assert_eq!(id_num, RequestId::Number(100));

    let id_str = RequestId::from("req-1".to_string());
    assert_eq!(id_str, RequestId::String("req-1".into()));

    let id_ref = RequestId::from("req-2");
    assert_eq!(id_ref, RequestId::String("req-2".into()));

    let err_int = athanor_mcp::protocol::JsonRpcError::internal_error("internal");
    assert_eq!(
        err_int.code,
        athanor_mcp::protocol::error_codes::INTERNAL_ERROR
    );

    let err_req = athanor_mcp::protocol::JsonRpcError::invalid_request("invalid req");
    assert_eq!(
        err_req.code,
        athanor_mcp::protocol::error_codes::INVALID_REQUEST
    );

    let err_params = athanor_mcp::protocol::JsonRpcError::invalid_params("bad params");
    assert_eq!(
        err_params.code,
        athanor_mcp::protocol::error_codes::INVALID_PARAMS
    );

    let err_parse = athanor_mcp::protocol::JsonRpcError::parse_error("parse fail");
    assert_eq!(
        err_parse.code,
        athanor_mcp::protocol::error_codes::PARSE_ERROR
    );

    let err_method = athanor_mcp::protocol::JsonRpcError::method_not_found("not found");
    assert_eq!(
        err_method.code,
        athanor_mcp::protocol::error_codes::METHOD_NOT_FOUND
    );
}

#[tokio::test]
async fn test_transport_semantics_and_arc_delegation() {
    let (t1, t2) = ChannelTransport::create_duplex(10);
    let arc_t1: Arc<dyn Transport> = Arc::new(t1);

    // Test send and receive via Arc delegation
    arc_t1.send("hello".into()).await.unwrap();
    let received = t2.receive().await.unwrap();
    assert_eq!(received, Some("hello".into()));

    // Test close
    arc_t1.close().await.unwrap();
    let send_err = arc_t1.send("world".into()).await;
    assert!(send_err.is_err());

    let eof = t2.receive().await.unwrap();
    assert_eq!(eof, None);
}

/// A reagent that returns plain non-JSON text.
struct PlainTextReagent;

#[async_trait::async_trait]
impl Reagent for PlainTextReagent {
    fn name(&self) -> &'static str {
        "plain_text"
    }
    fn description(&self) -> &'static str {
        "Returns plain text string"
    }
    fn input_schema(&self) -> serde_json::Value {
        json!({ "type": "object" })
    }
    fn output_schema(&self) -> serde_json::Value {
        json!({ "type": "string" })
    }
    async fn transmute(
        &self,
        _input: serde_json::Value,
    ) -> Result<serde_json::Value, ReagentError> {
        Ok(serde_json::Value::String(
            "Plain text non-json output".into(),
        ))
    }
}

#[tokio::test]
async fn test_client_plain_text_and_notify() {
    let (server_transport, client_transport) = ChannelTransport::create_duplex(16);
    let server = Arc::new(McpServer::new("text-server", "1.0.0"));
    server.register_reagent(Arc::new(PlainTextReagent)).await;

    let server_clone = Arc::clone(&server);
    tokio::spawn(async move {
        let _ = server_clone.run(Arc::new(server_transport)).await;
    });

    let client = McpClient::new(Arc::new(client_transport));
    client.initialize("test-client", "1.0.0").await.unwrap();

    // Invoke tool returning string
    let res = client.call_tool("plain_text", json!({})).await.unwrap();
    assert_eq!(res, json!("Plain text non-json output"));

    // Send custom notification
    let notif_res = client
        .notify("custom/event", Some(json!({"status": "active"})))
        .await;
    assert!(notif_res.is_ok());

    client.close().await.unwrap();
}
