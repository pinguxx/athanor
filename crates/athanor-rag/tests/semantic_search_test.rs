use athanor_core::artificer::Artificer;
use athanor_core::reagent::Reagent;
use athanor_rag::prelude::*;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

static TEST_MUTEX: Mutex<()> = Mutex::const_new(());

#[tokio::test]
async fn test_semantic_search_cosine_similarity() {
    let _lock = TEST_MUTEX.lock().await;
    // 1. Initialize in-process SpagyricIndex
    let index = SpagyricIndex::new_in_memory()
        .await
        .expect("Failed to initialize in-memory SpagyricIndex");

    // 2. Prepare multi-paragraph technical passages
    // Paragraph A: Photovoltaic solar cells
    let doc_a = Document::with_metadata(
        "doc-solar",
        "Photovoltaic solar cells convert radiant electromagnetic energy from sunlight directly \
         into electrical current through the photoelectric effect using semiconductor silicon wafers \
         doped with phosphorus and boron.",
        json!({ "topic": "renewable_energy", "domain": "physics" }),
    );

    // Paragraph B: Rocket cryogenic propellant combustion
    let doc_b = Document::with_metadata(
        "doc-rocket",
        "Cryogenic rocket propulsion systems rely on the controlled exothermic combustion of \
         supercooled liquid hydrogen fuel and liquid oxygen oxidizer within a regenerative cooling \
         thrust chamber to produce supersonic exhaust expansion.",
        json!({ "topic": "aerospace", "domain": "propulsion" }),
    );

    // 3. Ingest documents
    index
        .add_documents(&[doc_a, doc_b])
        .await
        .expect("Failed to ingest documents into SpagyricIndex");

    // 4. Query with semantic synonyms not literally in the text:
    // "spacecraft liquid fuel reaction" -> Should match Paragraph B (doc-rocket)
    let query = "spacecraft liquid fuel reaction";
    let search_results = index
        .search(query, 2)
        .await
        .expect("Failed to perform vector search");

    assert!(
        !search_results.is_empty(),
        "Search results should not be empty"
    );
    let top_result = &search_results[0];

    println!(
        "Top result id: {}, score: {:.4}, content: {}",
        top_result.document.id, top_result.score, top_result.document.content
    );

    // Verify top match is doc-rocket
    assert_eq!(
        top_result.document.id, "doc-rocket",
        "Top result should correspond to rocket propulsion"
    );

    // Verify cosine similarity score > 0.70
    assert!(
        top_result.score > 0.70,
        "Expected cosine similarity > 0.70, but got {:.4}",
        top_result.score
    );

    // 5. Test RagSearchReagent with Artificer
    let index_arc = Arc::new(index);
    let reagent = RagSearchReagent::new(Arc::clone(&index_arc));
    assert_eq!(reagent.name(), "rag_search");

    let reagent_output = reagent
        .transmute(json!({
            "query": "solar photoelectric semiconductor",
            "limit": 1
        }))
        .await
        .expect("Reagent transmute failed");

    let results_array = reagent_output.as_array().expect("Expected JSON array");
    assert_eq!(results_array.len(), 1);
    assert_eq!(results_array[0]["document"]["id"], "doc-solar");

    // Attach to Artificer
    let artificer = Artificer::builder()
        .name("researcher")
        .model("claude-3-5-sonnet")
        .reagent(Arc::new(reagent))
        .build()
        .expect("Artificer build failed");

    assert!(artificer.has_reagent("rag_search"));
    let tool = artificer.get_reagent("rag_search").expect("Tool not found");
    let tool_res = tool
        .transmute(json!({ "query": "rocket thrust chamber", "limit": 1 }))
        .await
        .expect("Artificer tool execution failed");
    assert_eq!(tool_res[0]["document"]["id"], "doc-rocket");
}

#[test]
fn test_recursive_character_splitter() {
    let splitter = RecursiveCharacterSplitter::new(100, 20).expect("Splitter init failed");

    let text =
        "Paragraph one is relatively long and discusses important aspects of thermodynamics.\n\n\
                Paragraph two switches context to orbital mechanics and planetary gravitation.\n\n\
                Paragraph three concludes with general observations on space mission design.";

    let chunks = splitter.split_text(text);
    assert!(
        chunks.len() >= 3,
        "Expected at least 3 chunks, got {}",
        chunks.len()
    );
    for chunk in &chunks {
        assert!(
            chunk.len() <= 120, // Allow slight boundary flexibility
            "Chunk exceeded expected bound: {}",
            chunk.len()
        );
    }

    // Test document splitting with metadata propagation
    let doc = Document::with_metadata("parent-doc", text, json!({ "source": "textbook.pdf" }));

    let doc_chunks = splitter.split_documents(&[doc]);
    assert!(doc_chunks.len() >= 3);
    for (i, c) in doc_chunks.iter().enumerate() {
        assert_eq!(c.id, format!("parent-doc_{i}"));
        assert_eq!(c.metadata["source"], "textbook.pdf");
        assert_eq!(c.metadata["parent_id"], "parent-doc");
        assert_eq!(c.metadata["chunk_index"], i);
    }

    // Splitter configuration validation
    assert!(RecursiveCharacterSplitter::new(0, 10).is_err());
    assert!(RecursiveCharacterSplitter::new(50, 50).is_err());
    assert!(RecursiveCharacterSplitter::new(50, 60).is_err());
}

#[tokio::test]
async fn test_index_edge_cases() {
    let _lock = TEST_MUTEX.lock().await;
    let index = SpagyricIndex::new_in_memory()
        .await
        .expect("Failed to initialize in-memory index");

    // Empty search before documents added
    let empty_search = index
        .search("anything", 5)
        .await
        .expect("Search should succeed");
    assert!(empty_search.is_empty());

    // Search with empty query
    let empty_query = index.search("", 5).await.expect("Search should succeed");
    assert!(empty_query.is_empty());

    // Search with limit 0
    let zero_limit = index
        .search("rocket", 0)
        .await
        .expect("Search should succeed");
    assert!(zero_limit.is_empty());

    // Add empty docs slice
    let add_empty = index.add_documents(&[]).await;
    assert!(add_empty.is_ok());

    // Persistent index creation in tempdir
    let temp = tempfile::tempdir().expect("tempdir");
    let persistent_index = SpagyricIndex::new_persistent(temp.path())
        .await
        .expect("persistent index init failed");

    let doc = Document::with_id(
        "p-doc",
        "Quantum mechanics explains particle spin and wave-particle duality.",
    );
    persistent_index
        .add_documents(&[doc])
        .await
        .expect("Adding to persistent index failed");

    let p_search = persistent_index
        .search("quantum spin", 1)
        .await
        .expect("Persistent search failed");
    assert_eq!(p_search.len(), 1);
    assert_eq!(p_search[0].document.id, "p-doc");
}

#[test]
fn test_document_constructors() {
    let d1 = Document::new("simple auto id document");
    assert!(!d1.id.is_empty());
    assert_eq!(d1.content, "simple auto id document");
    assert!(d1.metadata.is_object());

    let d2 = Document::with_id("custom-1", "custom id content");
    assert_eq!(d2.id, "custom-1");
    assert_eq!(d2.content, "custom id content");
    assert!(d2.metadata.is_object());
}

#[test]
fn test_error_conversions() {
    use athanor_core::reagent::ReagentError;

    let err_cfg = RagError::Config("bad config".into());
    let r_err: ReagentError = err_cfg.into();
    assert!(r_err.is_invalid_input());

    let err_doc = RagError::Document("bad doc".into());
    let r_err: ReagentError = err_doc.into();
    assert!(r_err.is_invalid_input());

    let err_emb = RagError::Embedding("onnx crash".into());
    let r_err: ReagentError = err_emb.into();
    assert!(r_err.is_fatal());

    let err_db = RagError::Database("table locked".into());
    let r_err: ReagentError = err_db.into();
    assert!(r_err.is_fatal());

    let serde_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let err_ser = RagError::Serialization(serde_err);
    let r_err: ReagentError = err_ser.into();
    assert!(r_err.is_invalid_input());

    let arrow_err = arrow_schema::ArrowError::SchemaError("bad schema".into());
    let err_arr = RagError::Arrow(arrow_err);
    let r_err: ReagentError = err_arr.into();
    assert!(r_err.is_fatal());

    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let err_io = RagError::Io(io_err);
    let r_err: ReagentError = err_io.into();
    assert!(r_err.is_fatal());
}

#[tokio::test]
async fn test_rag_reagent_schemas_and_validation() {
    let _lock = TEST_MUTEX.lock().await;
    let index = Arc::new(SpagyricIndex::new_in_memory().await.unwrap());
    let reagent = RagSearchReagent::with_details(
        Arc::clone(&index),
        "custom_search",
        "custom knowledge search description",
    );

    assert_eq!(reagent.name(), "custom_search");
    assert_eq!(reagent.description(), "custom knowledge search description");

    let input_schema = reagent.input_schema();
    assert!(input_schema["properties"]["query"].is_object());

    let output_schema = reagent.output_schema();
    assert!(output_schema["items"]["properties"]["score"].is_object());

    // Invalid parameters (not an object with query string)
    let bad_input = reagent.transmute(json!("just a string")).await;
    assert!(bad_input.is_err());
    let err = bad_input.unwrap_err();
    assert!(err.is_invalid_input());
}

#[test]
fn test_splitter_edge_cases() {
    let splitter = RecursiveCharacterSplitter::default()
        .with_separators(vec!["|".to_string(), "".to_string()]);

    assert!(splitter.split_text("").is_empty());
    assert_eq!(splitter.split_text("hello"), vec!["hello"]);

    let split_custom = splitter.split_text("alpha|beta|gamma");
    assert_eq!(split_custom, vec!["alpha|beta|gamma"]);
}
