use crate::config::{Config, ModelSettings};
use crate::tlsn_operations::extract_private_data;
use anyhow::{Context, Result};
use http_body_util::BodyExt;
use hyper::client::conn::http1::SendRequest;
use hyper::header::{AUTHORIZATION, CONNECTION, CONTENT_TYPE, HOST};
use hyper::{Method, StatusCode};
use tlsn_prover::tls::ProverControl;
use tracing::{debug, warn};

pub(super) async fn single_interaction_round(
    prover_ctrl: ProverControl,
    request_sender: &mut SendRequest<String>,
    config: &Config,
    messages: Vec<serde_json::Value>,
    recv_private_data: &mut Vec<Vec<u8>>,
    sent_private_data: &mut Vec<Vec<u8>>,
) -> Result<String> {

    // Prepare the Request to send to the model's API
    let request = generate_request(messages, &config.model_settings)
        .context("Error generating request")?;

    // Collect the private data transmitted in the request
    extract_private_data(
        sent_private_data,
        request.headers(),
        config.privacy_settings.request_topics_to_censor,
    );

    debug!("Request: {:?}", request);

    debug!("Sending request to Model...");

    prover_ctrl.defer_decryption().await.unwrap();


    let response = request_sender
        .send_request(request)
        .await
        .context("Error sending request to Model")?;

    debug!("Response: {:?}", response);

    if response.status() != StatusCode::OK {
        // TODO - do a graceful shutdown
        panic!(
            "Request failed with status code: {}",
            response.status()
        );
    }

    // Collect the received private data
    extract_private_data(
        recv_private_data,
        response.headers(),
        config.privacy_settings.response_topics_to_censor,
    );

    // // Collect the body
    // let payload = response
    //     .into_body()
    //     .collect()
    //     .await
    //     .context("Error reading response body")?
    //     .to_bytes();
    //
    // let parsed = serde_json::from_str::<serde_json::Value>(&String::from_utf8_lossy(&payload))
    //     .context("Error parsing the response")?;
    //
    // // Pretty printing the response
    // debug!(
    //     "Response: {}",
    //     serde_json::to_string_pretty(&parsed).context("Error pretty printing the response")?
    // );
    //
    // debug!("Extracting the assistant's response...");
    //
    // let received_assistant_message = serde_json::json!({"role": "assistant", "content": parsed["choices"][0]["message"]["content"]});

    // Ok(received_assistant_message.to_string())
    Ok("".to_string())
}

fn generate_request(
    messages: Vec<serde_json::Value>,
    model_settings: &ModelSettings,
) -> Result<hyper::Request<String>> {
    let messages = serde_json::to_value(messages).context("Error serializing messages")?;
    let mut json_body = serde_json::Map::new();
    json_body.insert("model".to_string(), serde_json::json!(model_settings.id));
    json_body.insert("messages".to_string(), messages);
    let json_body = serde_json::Value::Object(json_body);

    // Build the HTTP request to send the prompt to Model's API
    hyper::Request::builder()
        .method(Method::POST)
        .uri(model_settings.api_settings.inference_route)
        .header(HOST, model_settings.api_settings.server_domain)
        .header("Accept-Encoding", "identity")
        .header(CONNECTION, "close")
        .header(CONTENT_TYPE, "application/json")
        .header(
            AUTHORIZATION,
            format!("Bearer {}", model_settings.api_settings.api_key),
        )
        .body(json_body.to_string())
        .context("Error building the request")
}

pub(super) async fn shutdown_connection(
    prover_ctrl: ProverControl,
    request_sender: &mut SendRequest<String>,
    recv_private_data: &mut Vec<Vec<u8>>,
    config: &Config,
) {
    debug!("Conversation ended, sending final request to Model's API to shut down the session...");

    // Prepare final request to close the session
    let close_connection_request = hyper::Request::builder()
        .header(HOST, config.model_settings.api_settings.server_domain)
        .uri(config.model_settings.api_settings.inference_route)
        .header("Accept-Encoding", "identity")
        .header(CONNECTION, "close") // This will instruct the server to close the connection
        .body(String::new())
        .unwrap();

    debug!("Sending final request to Model's API...");

    // As this is the last request, we can defer decryption until the end.
    prover_ctrl.defer_decryption().await.unwrap();

    let response = request_sender
        .send_request(close_connection_request)
        .await
        .unwrap();

    // Collect the received private data
    extract_private_data(
        recv_private_data,
        response.headers(),
        config.privacy_settings.response_topics_to_censor,
    );

    // Collect the body
    let payload = response.into_body().collect().await.unwrap().to_bytes();

    let parsed = serde_json::from_str::<serde_json::Value>(&String::from_utf8_lossy(&payload))
        .unwrap_or_else(|_| {
            warn!("Error parsing the response");
            serde_json::json!({
                "error": "Error parsing the response"
            })
        });

    // Pretty printing the response
    debug!(
        "Shutdown response (error response is expected ): {}",
        serde_json::to_string_pretty(&parsed).unwrap()
    );
}