use athanor::prelude::*;
use serde_json::json;
use std::sync::Arc;

#[reagent(name = "test_adder")]
async fn test_adder(a: i64, b: i64) -> Result<i64, ReagentError> {
    Ok(a + b)
}

#[tokio::test]
async fn test_umbrella_artificer_and_reagent() {
    let artificer = Artificer::builder()
        .name("UmbrellaAgent")
        .instructions("Test agent utilizing the top-level athanor umbrella crate.")
        .model("claude-3-5-sonnet")
        .reagent(Arc::new(TestAdderReagent))
        .build()
        .expect("build artificer");

    assert_eq!(artificer.name, "UmbrellaAgent");
    assert!(artificer.has_reagent("test_adder"));

    let tool = artificer.get_reagent("test_adder").expect("get tool");
    let res = tool
        .transmute(json!({ "a": 20, "b": 22 }))
        .await
        .expect("transmute");
    assert_eq!(res, json!(42));
}

#[cfg(feature = "opus")]
#[tokio::test]
async fn test_umbrella_opus_and_crucible() {
    let crucible = Arc::new(
        CrucibleEngine::connect_in_memory()
            .await
            .expect("in-memory crucible"),
    );
    let engine = OpusEngine::new(crucible);

    struct SimpleVessel;

    #[async_trait]
    impl Vessel for SimpleVessel {
        fn id(&self) -> &'static str {
            "simple_vessel"
        }

        async fn distill(&self, ctx: &mut OpusContext) -> Result<VesselStatus, VesselError> {
            ctx.set("executed", true)
                .map_err(|e| VesselError::Execution(e.to_string()))?;
            Ok(VesselStatus::Transmuted {
                output: json!({ "done": true }),
            })
        }
    }

    let opus = Opus::builder("umbrella_workflow")
        .step(Arc::new(SimpleVessel))
        .build()
        .expect("build opus");

    let result = engine.execute(&opus, None).await.expect("start opus");
    assert!(matches!(result, OpusRunResult::Transmuted { .. }));
}
