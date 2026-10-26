use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

/// Shared workflow state memory context passed to each Vessel step.
#[derive(Clone, Debug)]
pub struct OpusContext {
    run_id: String,
    state: Value,
    resume_payload: Option<Value>,
}

impl OpusContext {
    /// Construct a new context initialized with a run ID and state snapshot.
    pub fn new(run_id: impl Into<String>, initial_state: Value) -> Self {
        let state = if initial_state.is_object() {
            initial_state
        } else if initial_state.is_null() {
            Value::Object(serde_json::Map::new())
        } else {
            serde_json::json!({ "value": initial_state })
        };

        Self {
            run_id: run_id.into(),
            state,
            resume_payload: None,
        }
    }

    /// Access the unique workflow run ID.
    pub fn run_id(&self) -> &str {
        &self.run_id
    }

    /// Access the complete state JSON snapshot.
    pub fn state(&self) -> &Value {
        &self.state
    }

    /// Overwrite the complete state JSON snapshot.
    pub fn set_state(&mut self, state: Value) {
        self.state = state;
    }

    /// Read and deserialize a typed value from the state object.
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let val = self.state.get(key)?;
        serde_json::from_value(val.clone()).ok()
    }

    /// Reference a raw JSON value by property key.
    pub fn get_value(&self, key: &str) -> Option<&Value> {
        self.state.get(key)
    }

    /// Store a serializable value into state under the specified key.
    pub fn set<T: Serialize>(
        &mut self,
        key: impl Into<String>,
        val: T,
    ) -> Result<(), serde_json::Error> {
        let json_val = serde_json::to_value(val)?;
        self.set_value(key, json_val);
        Ok(())
    }

    /// Store a raw `serde_json::Value` into state under the specified key.
    pub fn set_value(&mut self, key: impl Into<String>, val: Value) {
        if !self.state.is_object() {
            self.state = Value::Object(serde_json::Map::new());
        }
        if let Some(map) = self.state.as_object_mut() {
            map.insert(key.into(), val);
        }
    }

    /// Attach an external resumption payload upon tokenized workflow resume.
    pub fn set_resume_payload(&mut self, payload: Value) {
        self.resume_payload = Some(payload);
    }

    /// Reference the resumption payload if resumed from a suspended token.
    pub fn resume_payload(&self) -> Option<&Value> {
        self.resume_payload.as_ref()
    }

    /// Take and remove the resumption payload.
    pub fn take_resume_payload(&mut self) -> Option<Value> {
        self.resume_payload.take()
    }
}
