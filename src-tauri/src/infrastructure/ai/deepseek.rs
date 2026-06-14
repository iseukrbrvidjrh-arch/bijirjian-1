use std::time::Duration;

use reqwest::{blocking::Client, StatusCode};
use serde::{Deserialize, Serialize};

use crate::{domain::ProviderModel, error::AppError};

const DEEPSEEK_BASE_URL: &str = "https://api.deepseek.com";

pub struct DeepSeekAdapter {
    client: Client,
    base_url: String,
}

impl DeepSeekAdapter {
    pub fn new() -> Result<Self, AppError> {
        Self::with_base_url(DEEPSEEK_BASE_URL)
    }

    fn with_base_url(base_url: &str) -> Result<Self, AppError> {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(60))
            .user_agent("Second-Brain-OS/0.1")
            .build()
            .map_err(|_| {
                AppError::AiProvider("could not initialize the DeepSeek HTTP client".to_owned())
            })?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_owned(),
        })
    }

    pub fn test_connection(&self, api_key: &str) -> Result<(), AppError> {
        let response = self
            .client
            .get(format!("{}/models", self.base_url))
            .bearer_auth(api_key)
            .send()
            .map_err(connection_error)?;

        ensure_success_status(response.status())?;

        let models = response.json::<ModelsResponse>().map_err(|_| {
            AppError::AiProvider("DeepSeek returned an invalid /models response".to_owned())
        })?;

        if models.object != "list"
            || models.data.is_empty()
            || models.data.iter().any(|model| model.id.trim().is_empty())
        {
            return Err(AppError::AiProvider(
                "DeepSeek returned an invalid /models response".to_owned(),
            ));
        }

        Ok(())
    }

    pub fn generate_text(
        &self,
        model: ProviderModel,
        api_key: &str,
        system_prompt: &str,
        user_content: &str,
    ) -> Result<String, AppError> {
        let request = ChatCompletionRequest {
            model: model.as_str(),
            messages: [
                ChatMessage {
                    role: "system",
                    content: system_prompt,
                },
                ChatMessage {
                    role: "user",
                    content: user_content,
                },
            ],
        };
        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(api_key)
            .json(&request)
            .send()
            .map_err(connection_error)?;

        ensure_success_status(response.status())?;

        let completion = response.json::<ChatCompletionResponse>().map_err(|_| {
            AppError::AiProvider(
                "DeepSeek returned an invalid chat completions response".to_owned(),
            )
        })?;
        let content = completion
            .choices
            .into_iter()
            .next()
            .and_then(|choice| choice.message.content)
            .map(|content| content.trim().to_owned())
            .filter(|content| !content.is_empty())
            .ok_or_else(|| {
                AppError::AiProvider(
                    "DeepSeek returned an empty chat completions response".to_owned(),
                )
            })?;

        Ok(content)
    }
}

fn connection_error(error: reqwest::Error) -> AppError {
    if error.is_timeout() {
        AppError::AiProvider("DeepSeek connection timed out".to_owned())
    } else {
        AppError::AiProvider("could not connect to DeepSeek".to_owned())
    }
}

fn ensure_success_status(status: StatusCode) -> Result<(), AppError> {
    match status {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(AppError::AiProvider(
            "DeepSeek authentication failed; check the saved API key".to_owned(),
        )),
        StatusCode::TOO_MANY_REQUESTS => Err(AppError::AiProvider(
            "DeepSeek rate limit was reached; try again later".to_owned(),
        )),
        status if !status.is_success() => Err(AppError::AiProvider(format!(
            "DeepSeek returned HTTP status {status}"
        ))),
        _ => Ok(()),
    }
}

#[derive(Serialize)]
struct ChatCompletionRequest<'request> {
    model: &'request str,
    messages: [ChatMessage<'request>; 2],
}

#[derive(Serialize)]
struct ChatMessage<'message> {
    role: &'static str,
    content: &'message str,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: AssistantMessage,
}

#[derive(Deserialize)]
struct AssistantMessage {
    content: Option<String>,
}

#[derive(Deserialize)]
struct ModelsResponse {
    object: String,
    data: Vec<ModelSummary>,
}

#[derive(Deserialize)]
struct ModelSummary {
    id: String,
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        net::TcpListener,
        sync::mpsc,
        thread,
        time::Duration,
    };

    use super::DeepSeekAdapter;
    use crate::{domain::ProviderModel, error::AppError};

    #[test]
    fn accepts_a_valid_models_response() {
        let (base_url, request_rx, handle) =
            mock_server(200, r#"{"object":"list","data":[{"id":"deepseek-chat"}]}"#);
        let adapter = DeepSeekAdapter::with_base_url(&base_url).expect("create test adapter");

        adapter
            .test_connection("test-api-key")
            .expect("valid models response should succeed");

        let request = request_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("mock server should receive a request")
            .to_ascii_lowercase();
        assert!(request.starts_with("get /models "));
        assert!(request.contains("authorization: bearer test-api-key"));
        handle.join().expect("mock server should finish");
    }

    #[test]
    fn maps_unauthorized_and_forbidden_responses_to_authentication_errors() {
        for status in [401, 403] {
            let (base_url, _, handle) = mock_server(status, "{}");
            let adapter = DeepSeekAdapter::with_base_url(&base_url).expect("create test adapter");

            let error = adapter
                .test_connection("invalid-key")
                .expect_err("authentication response should fail");

            assert_ai_provider_error_contains(error, "authentication failed");
            handle.join().expect("mock server should finish");
        }
    }

    #[test]
    fn rejects_non_success_responses() {
        let (base_url, _, handle) = mock_server(500, "{}");
        let adapter = DeepSeekAdapter::with_base_url(&base_url).expect("create test adapter");

        let error = adapter
            .test_connection("test-api-key")
            .expect_err("server error should fail");

        assert_ai_provider_error_contains(error, "500");
        handle.join().expect("mock server should finish");
    }

    #[test]
    fn rejects_invalid_models_responses() {
        let (base_url, _, handle) = mock_server(200, r#"{"object":"list","data":[]}"#);
        let adapter = DeepSeekAdapter::with_base_url(&base_url).expect("create test adapter");

        let error = adapter
            .test_connection("test-api-key")
            .expect_err("invalid response should fail");

        assert_ai_provider_error_contains(error, "invalid /models response");
        handle.join().expect("mock server should finish");
    }

    #[test]
    fn sends_a_minimal_chat_completion_request_and_returns_content() {
        let (base_url, request_rx, handle) = mock_server(
            200,
            r#"{"choices":[{"message":{"role":"assistant","content":"  Concise summary.  "}}]}"#,
        );
        let adapter = DeepSeekAdapter::with_base_url(&base_url).expect("create test adapter");

        let summary = adapter
            .generate_text(
                ProviderModel::DeepSeekV4Flash,
                "test-api-key",
                "System prompt",
                "Wrapped source",
            )
            .expect("valid chat response should succeed");

        assert_eq!(summary, "Concise summary.");
        let request = request_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("mock server should receive a request");
        assert!(request.starts_with("POST /chat/completions "));
        assert!(request
            .to_ascii_lowercase()
            .contains("authorization: bearer test-api-key"));

        let body = request
            .split_once("\r\n\r\n")
            .map(|(_, body)| body)
            .expect("request should contain a body");
        let json: serde_json::Value = serde_json::from_str(body).expect("parse request JSON");
        assert_eq!(json["model"], "deepseek-v4-flash");
        assert_eq!(json["messages"][0]["role"], "system");
        assert_eq!(json["messages"][0]["content"], "System prompt");
        assert_eq!(json["messages"][1]["role"], "user");
        assert_eq!(json["messages"][1]["content"], "Wrapped source");
        assert_eq!(json.as_object().expect("request object").len(), 2);
        handle.join().expect("mock server should finish");
    }

    #[test]
    fn maps_chat_authentication_and_rate_limit_errors() {
        for (status, expected) in [
            (401, "authentication failed"),
            (403, "authentication failed"),
            (429, "rate limit"),
        ] {
            let (base_url, _, handle) = mock_server(status, "{}");
            let adapter = DeepSeekAdapter::with_base_url(&base_url).expect("create test adapter");

            let error = adapter
                .generate_text(
                    ProviderModel::DeepSeekV4Flash,
                    "test-api-key",
                    "System prompt",
                    "Source",
                )
                .expect_err("error status should fail");

            assert_ai_provider_error_contains(error, expected);
            handle.join().expect("mock server should finish");
        }
    }

    #[test]
    fn rejects_chat_non_success_statuses() {
        let (base_url, _, handle) = mock_server(500, "{}");
        let adapter = DeepSeekAdapter::with_base_url(&base_url).expect("create test adapter");

        let error = adapter
            .generate_text(
                ProviderModel::DeepSeekV4Pro,
                "test-api-key",
                "System prompt",
                "Source",
            )
            .expect_err("server error should fail");

        assert_ai_provider_error_contains(error, "500");
        handle.join().expect("mock server should finish");
    }

    #[test]
    fn rejects_invalid_or_empty_chat_responses() {
        for body in [
            "not-json",
            r#"{"choices":[]}"#,
            r#"{"choices":[{"message":{"content":null}}]}"#,
            r#"{"choices":[{"message":{"content":"   "}}]}"#,
        ] {
            let (base_url, _, handle) = mock_server(200, body);
            let adapter = DeepSeekAdapter::with_base_url(&base_url).expect("create test adapter");

            let error = adapter
                .generate_text(
                    ProviderModel::DeepSeekV4Flash,
                    "test-api-key",
                    "System prompt",
                    "Source",
                )
                .expect_err("invalid response should fail");

            assert!(matches!(error, AppError::AiProvider(_)));
            handle.join().expect("mock server should finish");
        }
    }

    fn mock_server(
        status: u16,
        body: &'static str,
    ) -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind local mock server");
        let address = listener.local_addr().expect("read mock server address");
        let (request_tx, request_rx) = mpsc::channel();

        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept mock request");
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .expect("set mock read timeout");
            let mut request = Vec::new();
            let mut buffer = [0_u8; 1024];

            let header_end = loop {
                let bytes_read = stream.read(&mut buffer).expect("read mock request");
                if bytes_read == 0 {
                    break request.len();
                }
                request.extend_from_slice(&buffer[..bytes_read]);
                if let Some(position) = request.windows(4).position(|window| window == b"\r\n\r\n")
                {
                    break position + 4;
                }
            };
            let headers = String::from_utf8_lossy(&request[..header_end]);
            let content_length = headers
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().ok())
                        .flatten()
                })
                .unwrap_or_default();

            while request.len() < header_end + content_length {
                let bytes_read = stream.read(&mut buffer).expect("read mock request body");
                if bytes_read == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..bytes_read]);
            }

            let _ = request_tx.send(String::from_utf8_lossy(&request).into_owned());

            let reason = match status {
                200 => "OK",
                401 => "Unauthorized",
                403 => "Forbidden",
                429 => "Too Many Requests",
                500 => "Internal Server Error",
                _ => "Test Response",
            };
            let response = format!(
                "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream
                .write_all(response.as_bytes())
                .expect("write mock response");
        });

        (format!("http://{address}"), request_rx, handle)
    }

    fn assert_ai_provider_error_contains(error: AppError, expected: &str) {
        match error {
            AppError::AiProvider(message) => assert!(
                message.contains(expected),
                "expected '{message}' to contain '{expected}'"
            ),
            other => panic!("expected AI provider error, got {other}"),
        }
    }
}
