use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use tower::ServiceExt;

use crate::router;

use super::support;

#[tokio::test]
async fn json_and_stream_surfaces_preserve_the_same_seeded_token_stream() {
    let request = format!(
        r#"{{"model":"{}","prompt":"test","max_tokens":2,"temperature":0.7,"top_p":0.9,"seed":42}}"#,
        support::reference().as_str()
    );

    let response = router(support::node())
        .oneshot(
            Request::post("/v1/generate")
                .header("content-type", "application/json")
                .body(Body::from(request.clone()))
                .expect("request should build"),
        )
        .await
        .expect("router should respond");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 16 * 1024)
        .await
        .expect("response body should be readable");
    let json = String::from_utf8(body.to_vec()).expect("response should be UTF-8");
    assert!(json.contains("hello world"));
    assert!(json.contains("\"id\":10"));
    assert!(json.contains("\"id\":11"));

    let response = router(support::node())
        .oneshot(
            Request::post("/v1/generate/stream")
                .header("content-type", "application/json")
                .body(Body::from(request))
                .expect("request should build"),
        )
        .await
        .expect("router should respond");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 16 * 1024)
        .await
        .expect("stream body should be readable");
    let stream = String::from_utf8(body.to_vec()).expect("stream should be UTF-8");
    assert!(stream.contains("event: token"));
    assert!(stream.contains(r#"data: {"id":10,"text":"hello"}"#));
    assert!(stream.contains(r#"data: {"id":11,"text":" world"}"#));
    assert!(stream.contains("event: complete"));
}

#[tokio::test]
async fn invalid_requests_return_a_safe_stable_error_code() {
    let response = router(support::node())
        .oneshot(
            Request::post("/v1/generate")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"model":"not-a-reference","prompt":"test","max_tokens":2,"temperature":0.7,"top_p":0.9,"seed":42}"#,
                ))
                .expect("request should build"),
        )
        .await
        .expect("router should respond");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), 16 * 1024)
        .await
        .expect("error body should be readable");
    assert!(String::from_utf8(body.to_vec())
        .expect("error body should be UTF-8")
        .contains("SYN-MODEL-001"));
}

#[tokio::test]
async fn requests_larger_than_the_local_api_limit_are_rejected() {
    let oversized_prompt = "x".repeat(16 * 1024);
    let request = format!(
        r#"{{"model":"{}","prompt":"{}","max_tokens":2,"temperature":0.7,"top_p":0.9,"seed":42}}"#,
        support::reference().as_str(),
        oversized_prompt
    );
    let response = router(support::node())
        .oneshot(
            Request::post("/v1/generate")
                .header("content-type", "application/json")
                .body(Body::from(request))
                .expect("request should build"),
        )
        .await
        .expect("router should respond");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), 16 * 1024)
        .await
        .expect("error body should be readable");
    assert!(String::from_utf8(body.to_vec())
        .expect("error body should be UTF-8")
        .contains("SYN-API-001"));
}
