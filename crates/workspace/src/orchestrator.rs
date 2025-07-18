use std::sync::Arc;
use client::Client;
use gpui::AsyncApp;
use node_runtime::NodeRuntime;

pub struct Orchestrator {
    node_runtime: NodeRuntime,
    client: Arc<Client>,
}

impl Orchestrator {
    pub fn new(node_runtime: NodeRuntime, client: Arc<Client>) -> Self {
        Self {
            node_runtime,
            client,
        }
    }

    pub async fn receivePrompt(&self, prompt: String) -> String {
        let script = format!(
            r#"
            const {{ orchestrator }} = require('dist/core/orchestrator');
            orchestrator.receivePrompt('{prompt}')
                .then(result => {{
                    console.log(result);
                    return result;
                }})
                .catch(err => {{
                    console.error(err);
                    return 'Error: ' + err.message;
                }});
            "#
        );

        let result = self.node_runtime.run_script(script, vec![]).await;
        match result {
            Ok(result) => result.to_string(),
            Err(err) => format!("Error: {}", err),
        }
    }
}
