use crate::auth::AuthCookies;
use crate::config::API_BASE_URL;
use crate::error::{Error, Result};
use rquest::Client as HttpClient;
use rquest::cookie::Jar;
use rquest_util::Emulation;
use std::sync::Arc;
use std::time::Duration;

pub(crate) fn build_http_client(cookies: Option<&AuthCookies>) -> Result<HttpClient> {
    let jar = Arc::new(Jar::default());
    let url: rquest::Url = API_BASE_URL
        .parse()
        .expect("BUG: API_BASE_URL is a constant and must be a valid URL");

    if let Some(auth) = cookies {
        for (name, value) in auth.as_pairs() {
            let cookie = format!("{name}={value}; Domain=www.perplexity.ai; Path=/");
            jar.add_cookie_str(&cookie, &url);
        }
    }

    HttpClient::builder()
        .emulation(Emulation::Chrome131)
        .cookie_provider(jar)
        .build()
        .map_err(Error::HttpClientInit)
}

pub(crate) async fn warmup(http: &HttpClient, timeout: Duration) -> Result<()> {
    use crate::config::ENDPOINT_AUTH_SESSION;

    let fut = http
        .get(format!("{API_BASE_URL}{ENDPOINT_AUTH_SESSION}"))
        .send();

    tokio::time::timeout(timeout, fut)
        .await
        .map_err(|_| Error::Timeout(timeout))?
        .map_err(Error::SessionWarmup)?;

    Ok(())
}
