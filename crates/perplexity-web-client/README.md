# perplexity-web-client

Typed async Rust client for Perplexity's web app endpoints.

This crate is the low-level integration layer used by `perplexity-api-server`. It handles session warm-up, request building, SSE parsing, and normalized search responses.

## Example

```rust
use perplexity_web_client::{
    AuthCookies, Client, ReasonModel, SearchMode, SearchRequest,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    let cookies = AuthCookies::new(
        std::env::var("PERPLEXITY_SESSION_TOKEN")?,
        std::env::var("PERPLEXITY_CSRF_TOKEN")?,
    );

    let client = Client::builder()
        .cookies(cookies)
        .build()
        .await?;

    let request = SearchRequest::new("What is Rust?")
        .mode(SearchMode::Reasoning)
        .model(ReasonModel::Claude46SonnetThinking);

    let response = client.search(request).await?;

    if let Some(answer) = response.answer {
        println!("{answer}");
    }

    Ok(())
}
```

## Public API

| Item | Description |
| --- | --- |
| `Client` | async entry point for search and streaming requests |
| `ClientBuilder` | configures cookies and request timeout before building a `Client` |
| `AuthCookies` | typed wrapper around the required Perplexity session cookies |
| `SearchRequest` | request builder for one Perplexity query |
| `SearchMode` | high-level request mode: search, reasoning, or deep research |
| `SearchModel` | supported model names for `search` mode |
| `ReasonModel` | supported model names for `reason` mode |
| `Source` | source filter enum used by `SearchRequest` |
| `SearchEvent` | one parsed event from the SSE stream |
| `SearchResponse` | final normalized response returned by `Client::search()` |

## Notes

- This crate uses Perplexity browser session cookies, not the official Perplexity API.
- `Client::search()` returns the final normalized response.
- `Client::search_stream()` returns progressive snapshots from the SSE stream.
