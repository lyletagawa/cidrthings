use axum::{
    extract::Query,
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use cidrthings::{minimal_supernet, Cidr};
use serde::Deserialize;

#[derive(Deserialize)]
struct SupernetQuery {
    /// Comma-separated CIDR blocks, e.g. ?cidrs=10.1.0.0/24,10.2.0.0/24
    cidrs: Option<String>,
}

fn text(status: StatusCode, body: String) -> Response {
    (
        status,
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        body,
    )
        .into_response()
}

fn parse_and_summarize(input: &str) -> Response {
    let mut blocks: Vec<Cidr> = Vec::new();
    for s in input
        .split(['\n', ','])
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        match s.parse::<Cidr>() {
            Ok(c) => blocks.push(c),
            Err(e) => return text(StatusCode::BAD_REQUEST, format!("error parsing {s:?}: {e}")),
        }
    }
    match minimal_supernet(&blocks) {
        Ok(supernet) => text(StatusCode::OK, supernet.to_string()),
        Err(e) => text(StatusCode::BAD_REQUEST, e.to_string()),
    }
}

async fn get_handler(Query(q): Query<SupernetQuery>) -> Response {
    match q.cidrs {
        Some(s) => parse_and_summarize(&s),
        None => Html(INDEX_HTML).into_response(),
    }
}

async fn post_handler(body: String) -> Response {
    parse_and_summarize(&body)
}

const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>cidrthings — supernet calculator</title>
<style>
  body { font-family: monospace; max-width: 640px; margin: 4rem auto; padding: 0 1rem; }
  h1 { font-size: 1.2rem; }
  label { display: block; margin: 1rem 0 0.25rem; }
  textarea { width: 100%; height: 8rem; font-family: monospace; font-size: 1rem; }
  button { margin-top: 0.75rem; padding: 0.4rem 1.2rem; font-size: 1rem; cursor: pointer; }
  #result { margin-top: 1.5rem; font-size: 1.4rem; font-weight: bold; }
  #error { margin-top: 1rem; color: #c00; }
</style>
</head>
<body>
<h1>cidrthings — minimal supernet</h1>
<p>Enter CIDR blocks (one per line or comma-separated) to find their minimal enclosing supernet.</p>
<label for="cidrs">CIDR blocks:</label>
<textarea id="cidrs" placeholder="10.1.0.0/24&#10;10.2.0.0/24"></textarea>
<br>
<button onclick="compute()">Compute supernet</button>
<div id="result"></div>
<div id="error"></div>
<script>
async function compute() {
  const raw = document.getElementById('cidrs').value.trim();
  document.getElementById('result').textContent = '';
  document.getElementById('error').textContent = '';
  if (!raw) { document.getElementById('error').textContent = 'Enter at least one CIDR block.'; return; }
  const res = await fetch('/', { method: 'POST', body: raw });
  const text = await res.text();
  if (res.ok) {
    document.getElementById('result').textContent = text;
  } else {
    document.getElementById('error').textContent = text;
  }
}
</script>
</body>
</html>
"#;

fn router() -> Router {
    Router::new().route("/", get(get_handler).post(post_handler))
}

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .unwrap();
    println!("listening on http://0.0.0.0:{port}");
    axum::serve(listener, router()).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn body_text(b: axum::body::Body) -> String {
        let bytes = b.collect().await.unwrap().to_bytes();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    async fn get(uri: &str) -> (StatusCode, String) {
        let resp = router()
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = resp.status();
        (status, body_text(resp.into_body()).await)
    }

    async fn post(body: &'static str) -> (StatusCode, String) {
        let resp = router()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = resp.status();
        (status, body_text(resp.into_body()).await)
    }

    #[tokio::test]
    async fn get_query_returns_supernet() {
        let (status, body) = get("/?cidrs=10.1.0.0/24,10.2.0.0/24").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "10.0.0.0/14");
    }

    #[tokio::test]
    async fn get_no_query_returns_html() {
        let (status, body) = get("/").await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.starts_with("<!DOCTYPE html>"));
    }

    #[tokio::test]
    async fn post_newline_delimited() {
        let (status, body) = post("10.1.0.0/24\n10.2.0.0/24").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "10.0.0.0/14");
    }

    #[tokio::test]
    async fn post_comma_delimited() {
        let (status, body) = post("10.1.0.0/24,10.2.0.0/24").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "10.0.0.0/14");
    }

    #[tokio::test]
    async fn post_bare_ip() {
        let (status, body) = post("10.0.0.1\n10.0.0.2").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "10.0.0.0/30");
    }

    #[tokio::test]
    async fn post_invalid_cidr_returns_400() {
        let (status, _) = post("not-a-cidr").await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn post_empty_body_returns_400() {
        let (status, _) = post("").await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn post_mixed_families_returns_400() {
        let (status, body) = post("10.0.0.0/8\n2001:db8::/32").await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(body.contains("IPv4") || body.contains("IPv6"));
    }

    #[tokio::test]
    async fn text_responses_have_content_type() {
        let resp = router()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/")
                    .body(Body::from("10.0.0.0/8"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/plain; charset=utf-8"
        );
    }
}
