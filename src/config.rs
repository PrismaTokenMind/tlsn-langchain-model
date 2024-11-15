#![allow(dead_code)]

use std::sync::LazyLock;

static SETUP_PROMPT: LazyLock<&str> =
    LazyLock::new(|| "Model Prompt: YOU ARE GOING TO BE ACTING AS A HELPFUL ASSISTANT");

/// Configuration for API settings, including server endpoints and the API key
#[derive(Debug, Default)]
pub struct ModelApiSettings {
    pub server_domain: &'static str,
    pub inference_route: &'static str,
    pub model_list_route: &'static str,
    pub api_key: String,
}

impl ModelApiSettings {
    pub(crate) fn new(api_key: String) -> Self {
        Self {
            server_domain: "api.red-pill.ai",
            inference_route: "/v1/chat/completions",
            model_list_route: "/v1/models",
            api_key,
        }
    }
}

#[derive(Debug)]
pub struct NotarySettings {
    pub dummy_notary: bool, // TODO - improve this struct to be more effective
    pub host: &'static str,
    pub port: u16,
    pub path: &'static str,
}

/// Configuration for Notary settings, defining host, port, and path
impl Default for NotarySettings {
    fn default() -> Self {
        NotarySettings {
            dummy_notary: true,
            host: "notary.pse.dev", // TODO - figure out why this is not working
            port: 443,
            path: "v0.1.0-alpha.6",
        }
    }
}

/// Privacy settings including topics to censor in requests and responses
#[derive(Debug)]
pub struct PrivacySettings {
    pub request_topics_to_censor: &'static [&'static str],
    pub response_topics_to_censor: &'static [&'static str],
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            request_topics_to_censor: &["authorization"],
            response_topics_to_censor: &[
                "anthropic-ratelimit-requests-reset",
                "anthropic-ratelimit-tokens-reset",
                "request-id",
                "x-kong-request-id",
                "cf-ray",
                "server-timing",
                "report-to",
            ],
        }
    }
}

/// Model settings including API settings, model ID, and setup prompt
#[derive(Debug)]
pub struct ModelSettings {
    pub api_settings: ModelApiSettings,
    pub id: String,
    pub setup_prompt: &'static str,
}

impl ModelSettings {
    fn new(model_id: String, api_settings: ModelApiSettings) -> Self {
        Self {
            api_settings,
            id: model_id,
            setup_prompt: *SETUP_PROMPT,
        }
    }
}

/// Complete application configuration including model, privacy, and notary settings
#[derive(Debug)]
pub struct Config {
    pub model_settings: ModelSettings,
    pub privacy_settings: PrivacySettings,
    pub notary_settings: NotarySettings,
}

impl Config {
    fn new(model_settings: ModelSettings) -> Self {
        Self {
            model_settings,
            privacy_settings: PrivacySettings::default(),
            notary_settings: NotarySettings::default(),
        }
    }
}
