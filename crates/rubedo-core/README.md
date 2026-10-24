# rubedo-core

Core alchemical primitives, traits, errors, and context for the **Rubedo** agentic framework.

## Features

- **Reagent Trait**: Asynchronous tool contract with strict `'static + Send + Sync` guarantees.
- **ReagentError**: Tri-state error model differentiating validation errors, recoverable errors with guidance, and fatal errors.
- **Artificer**: Agent representation reasoning over instructions, models, and reagent toolsets.
- **AthanorContext**: Digestion context maintaining conversation history, token metrics, and execution telemetry.
- **Envelope<T>**: Zero-lifetime owned envelope wrapper for thread-safe value transfers.
