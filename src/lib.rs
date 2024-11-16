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

#[allow(unused_variables)]
#[pyfunction]
pub fn exec(py: Python, model: String, api_key: String, messages: Vec<String>, tools: Vec<String>, top_p: f64, temperature: f64, stream: bool) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        notarised_model_request(model, api_key, messages, tools, top_p, temperature).await.map_err(|e| {
            PyErr::new::<PyTypeError, _>(e.to_string())
        })
    })
}

pub async fn notarised_model_request(model: String, api_key: String, messages: Vec<String>, tools: Vec<String>, top_p: f64, temperature: f64) -> Result<(String, String)> {
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

    // TODO - explore how to do it stateful to avoid redoing the attestation every time
    let (_, prover_task, mut request_sender) = setup_connections(&config)
        .await
        .context("Error setting up connections")?;

    debug!("Initialising the message conversation...");
    let parsed_messages = messages
        .iter()
        .map(|m| serde_json::from_str(m))
        .collect::<Result<Vec<serde_json::Value>, _>>()
        .context("Error parsing messages")?;

    let parsed_tools = tools
        .iter()
        .map(|m| serde_json::from_str(m))
        .collect::<Result<Vec<serde_json::Value>, _>>()
        .context("Error parsing tools")?;

    let mut recv_private_data = vec![];
    let mut sent_private_data = vec![];

    let response = single_interaction_round(
        &mut request_sender,
        &config,
        parsed_messages,
        parsed_tools,
        top_p,
        temperature,
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

        let messages = vec![
            "{
                \"role\": \"user\",
                \"content\": \"hi im bob! and i live in sf\"
            }",
            "{
                \"role\": \"assistant\",
                \"content\": \"Hi Bob! It's great to meet you. How can I assist you today?\"
            }",
            "{
                \"role\": \"user\",
                \"content\": \"whats the weather where I live?\"
            }"
        ].iter().map(|s| s.to_string()).collect::<Vec<String>>();

        let tools: Vec<String> = vec!["
            {
                \"type\": \"function\",
                \"function\": {
                    \"name\": \"tavily_search_results_json\",
                    \"description\": \"A search engine optimized for comprehensive, accurate, and trusted results. Useful for when you need to answer questions about current events. Input should be a search query.\",
                    \"parameters\": {
                        \"properties\": {
                            \"query\": {
                                \"description\": \"search query to look up\",
                                \"type\": \"string\"
                            }
                        },
                        \"required\": [\"query\"],
                        \"type\": \"object\"
                    }
                }
            }"
        ].iter().map(|s| s.to_string()).collect::<Vec<String>>();

        let top_p = 0.85;
        let temperature = 0.3;

        let (response, proof) = notarised_model_request(model, api_key, messages, tools, top_p, temperature).await?;
        println!("Response: {}", response);
        println!("Proof: {}", proof.replace("\n", "").replace(" ", ""));

        Ok(())
    }

    #[test]
    fn test_parsing() -> Result<()> {
        let messages = vec![
            "{
                \"role\": \"user\",
                \"content\": \"hi im bob! and i live in sf\"
            }",
            "{
                \"role\": \"assistant\",
                \"content\": \"Hi Bob! It's great to meet you. How can I assist you today?\"
            }",
            "{
                \"role\": \"user\",
                \"content\": \"whats the weather where I live?\"
            }"
        ].iter().map(|s| s.to_string()).collect::<Vec<String>>();

        let tools: Vec<String> = vec!["
            {
                \"type\": \"function\",
                \"function\": {
                    \"name\": \"tavily_search_results_json\",
                    \"description\": \"A search engine optimized for comprehensive, accurate, and trusted results. Useful for when you need to answer questions about current events. Input should be a search query.\",
                    \"parameters\": {
                        \"properties\": {
                            \"query\": {
                                \"description\": \"search query to look up\",
                                \"type\": \"string\"
                            }
                        },
                        \"required\": [\"query\"],
                        \"type\": \"object\"
                    }
                }
            }
        "].iter().map(|s| s.to_string()).collect::<Vec<String>>();

        let parsed_messages: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| serde_json::from_str(m))
            .collect::<Result<Vec<serde_json::Value>, _>>()?;
        let parsed_tools: Vec<serde_json::Value> = tools
            .iter()
            .map(|m| serde_json::from_str(m))
            .collect::<Result<Vec<serde_json::Value>, _>>()?;

        assert_eq!(parsed_messages.len(), 3);
        assert_eq!(parsed_tools.len(), 1);

        assert_eq!(
            parsed_messages[1]
                .get("content")
                .and_then(|name| name.as_str()), // Convert to &str if it's a JSON value
            Some("Hi Bob! It's great to meet you. How can I assist you today?")
        );
        assert_eq!(
            parsed_tools[0]
                .get("function")
                .and_then(|func| func.get("name"))
                .and_then(|name| name.as_str()), // Convert to &str if it's a JSON value
            Some("tavily_search_results_json")
        );
        Ok(())
    }
}

