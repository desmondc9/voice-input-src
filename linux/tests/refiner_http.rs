use voice_input::refiner::LlmRefiner;
use wiremock::matchers::{body_partial_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn happy_path_returns_refined_text() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(header("authorization", "Bearer sk-test"))
        .and(body_partial_json(serde_json::json!({
            "model": "gpt-4o-mini",
            "temperature": 0.3,
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "choices": [
                {"message": {"role": "assistant", "content": "Python and JSON"}}
            ]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let refiner = LlmRefiner::for_test(server.uri(), "sk-test", "gpt-4o-mini");
    let out = refiner.try_refine("配森 和 杰森", false).await.unwrap();
    assert_eq!(out, "Python and JSON");
}

#[tokio::test]
async fn trailing_slash_in_base_url_does_not_double_up() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "choices": [{"message": {"content": "ok"}}]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let url_with_slash = format!("{}/", server.uri());
    let refiner = LlmRefiner::for_test(url_with_slash, "sk-test", "gpt-4o-mini");
    let out = refiner.try_refine("hi", false).await.unwrap();
    assert_eq!(out, "ok");
}

#[tokio::test]
async fn response_content_is_whitespace_trimmed() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "choices": [{"message": {"content": "  Python and JSON  \n"}}]
        })))
        .mount(&server)
        .await;

    let refiner = LlmRefiner::for_test(server.uri(), "sk-test", "gpt-4o-mini");
    let out = refiner.try_refine("配森", false).await.unwrap();
    assert_eq!(out, "Python and JSON");
}
