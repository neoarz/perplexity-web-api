/// Cookie names and credentials used to authenticate against Perplexity's web app.
pub const SESSION_TOKEN_COOKIE: &str = "__Secure-next-auth.session-token";

pub const CSRF_TOKEN_COOKIE: &str = "next-auth.csrf-token";

/// Browser cookies needed to reuse an existing Perplexity session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthCookies {
    session_token: String,
    csrf_token: String,
}

impl AuthCookies {
    /// Builds a cookie pair from the values copied out of your browser.
    pub fn new(session_token: impl Into<String>, csrf_token: impl Into<String>) -> Self {
        Self {
            session_token: session_token.into(),
            csrf_token: csrf_token.into(),
        }
    }

    /// Returns the raw session token value.
    pub fn session_token(&self) -> &str {
        &self.session_token
    }

    /// Returns the raw CSRF token value.
    pub fn csrf_token(&self) -> &str {
        &self.csrf_token
    }

    pub(crate) fn as_pairs(&self) -> [(&str, &str); 2] {
        [
            (SESSION_TOKEN_COOKIE, self.session_token()),
            (CSRF_TOKEN_COOKIE, self.csrf_token()),
        ]
    }
}
