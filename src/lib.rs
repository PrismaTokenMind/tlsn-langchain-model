mod model_interactions;
mod setup_notary;
mod config;
mod tlsn_operations;

use crate::config::{Config, ModelSettings};
use crate::model_interactions::single_interaction_round;
use crate::setup_notary::setup_connections;
use crate::tlsn_operations::{build_proof, notarise_session};
use anyhow::{Context, Result};
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::PyModule;
use pyo3::{pyfunction, pymodule, wrap_pyfunction, PyAny, PyErr, PyResult, Python};
use tracing::debug;

#[pymodule]
fn tlsn_langchain(_: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(exec, m)?)?;
    Ok(())
}

#[pyfunction]
pub fn exec(py: Python, model: String, api_key: String, messages: Vec<String>) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        generate_conversation_attribution(model, api_key, messages).await.map_err(|e| {
            PyErr::new::<PyTypeError, _>(e.to_string())
        })
    })
}

pub async fn generate_conversation_attribution(model: String, api_key: String, messages: Vec<String>) -> Result<(String, String)> {
    let config = Config {
        model_settings: ModelSettings {
            id: model,
            api_settings: config::ModelApiSettings::new(api_key),
            setup_prompt: "Model Prompt: YOU ARE GOING TO BE ACTING AS A HELPFUL ASSISTANT",
        },
        privacy_settings: config::PrivacySettings::default(),
        notary_settings: config::NotarySettings::default(),
    };

    debug!("The system is being setup...");

    let (_, prover_task, mut request_sender) = setup_connections(&config)
        .await
        .context("Error setting up connections")?;

    debug!("Initialising the message conversation...");
    let parsed_messages = messages
        .iter()
        .map(|m| serde_json::from_str(m))
        .collect::<Result<Vec<serde_json::Value>, _>>()
        .context("Error parsing messages")?;

    let mut recv_private_data = vec![];
    let mut sent_private_data = vec![];

    let response = single_interaction_round(
        &mut request_sender,
        &config,
        parsed_messages,
        &mut recv_private_data,
        &mut sent_private_data,
    )
        .await?;

    debug!("Shutting down the connection with the API...");

    // Notarize the session
    debug!("Notarizing the session...");
    let notarised_session = notarise_session(prover_task, &recv_private_data, &sent_private_data)
        .await
        .context("Error notarizing the session")?;

    // Build the proof
    debug!("Building the proof...");
    let proof = build_proof(notarised_session);

    Ok((response, serde_json::to_string_pretty(&proof)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_generate_conversation_attribution() -> Result<()> {
        dotenv::dotenv().ok();
        let api_key = env::var("REDPILL_API_KEY")?;

        let model = "gpt-4o".to_string();
        let messages = vec![serde_json::json!(
        {
            "role": "user",
            "content": "Hello, I am John, how are you doing?"
        }
        ).to_string()];

        println!("Model: {}", messages[0]);

        let (response, proof) = generate_conversation_attribution(model, api_key, messages).await?;
        println!("Response: {}", response);
        println!("Proof: {}", proof);

        Ok(())
    }

    #[test]
    fn test_parsing() {
        let messages = vec!["        {
            \"role\": \"user\",
            \"content\": \"Hello, I am John, how are you doing?\"
        }"];

        let parsed_messages = messages
            .iter()
            .map(|m| serde_json::from_str(m))
            .collect::<Result<Vec<serde_json::Value>, _>>().unwrap();

        println!("{:?}", parsed_messages);
        assert!(parsed_messages.len() == 1);
    }
}

