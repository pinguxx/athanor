use async_trait::async_trait;
use athanor_core::artificer::Artificer;
use athanor_core::reagent::{Reagent, ReagentError};
use serde_json::{json, Value};
use std::sync::Arc;

struct MockSearchReagent;

#[async_trait]
impl Reagent for MockSearchReagent {
    fn name(&self) -> &'static str {
        "search"
    }

    fn description(&self) -> &'static str {
        "Searches knowledge database"
    }

    fn input_schema(&self) -> Value {
        json!({ "type": "object", "properties": { "query": { "type": "string" } } })
    }

    fn output_schema(&self) -> Value {
        json!({ "type": "array", "items": { "type": "string" } })
    }

    async fn transmute(&self, _input: Value) -> Result<Value, ReagentError> {
        Ok(json!(["doc1", "doc2"]))
    }
}

#[test]
fn test_artificer_builder_success() {
    let search_tool = Arc::new(MockSearchReagent);
    let artificer = Artificer::builder()
        .name("Hermes")
        .instructions("You are a knowledgeable alchemical philosopher.")
        .model("claude-3-5-sonnet")
        .reagent(search_tool)
        .max_steps(15)
        .build();

    assert!(artificer.is_ok());
    let agent = artificer.unwrap();
    assert_eq!(agent.name, "Hermes");
    assert_eq!(agent.model, "claude-3-5-sonnet");
    assert_eq!(agent.max_steps, 15);
    assert_eq!(agent.reagents.len(), 1);
    assert!(agent.has_reagent("search"));
    assert!(!agent.has_reagent("nonexistent"));
    assert_eq!(agent.reagent_names(), vec!["search"]);

    let found = agent.get_reagent("search");
    assert!(found.is_some());
    assert_eq!(found.unwrap().name(), "search");

    let schemas = agent.reagent_schemas();
    assert_eq!(schemas.len(), 1);
    assert_eq!(schemas[0]["name"], "search");

    let debug_repr = format!("{:?}", agent);
    assert!(debug_repr.contains("Hermes"));
    assert!(debug_repr.contains("search"));
}

#[test]
fn test_artificer_builder_validation() {
    // Missing name
    let err_name = Artificer::builder().model("gpt-4o").build().unwrap_err();
    assert!(err_name.to_string().contains("'name' is required"));

    // Missing model
    let err_model = Artificer::builder().name("Paracelsus").build().unwrap_err();
    assert!(err_model.to_string().contains("'model' is required"));

    // Default max_steps is 25
    let agent = Artificer::builder()
        .name("Albertus")
        .model("claude-3-haiku")
        .build()
        .unwrap();
    assert_eq!(agent.max_steps, 25);
    assert_eq!(agent.instructions, "");
}

#[test]
fn test_artificer_add_reagents() {
    let mut agent = Artificer::builder()
        .name("Zosimos")
        .model("gemini-1.5-pro")
        .build()
        .unwrap();

    assert_eq!(agent.reagents.len(), 0);
    agent.add_reagent(Arc::new(MockSearchReagent));
    assert_eq!(agent.reagents.len(), 1);
    assert!(agent.has_reagent("search"));
}
