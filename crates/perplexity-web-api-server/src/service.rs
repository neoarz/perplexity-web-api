use futures_util::StreamExt;
use futures_util::future::BoxFuture;
use futures_util::stream::BoxStream;
use perplexity_web_client::{
    Client, Error as ClientError, SearchEvent, SearchRequest, SearchResponse,
};

pub type ClientResult<T> = Result<T, ClientError>;

pub trait PerplexityService: Send + Sync {
    fn search(&self, request: SearchRequest) -> BoxFuture<'_, ClientResult<SearchResponse>>;

    fn search_stream(
        &self,
        request: SearchRequest,
    ) -> BoxFuture<'_, ClientResult<BoxStream<'static, ClientResult<SearchEvent>>>>;
}

impl PerplexityService for Client {
    fn search(&self, request: SearchRequest) -> BoxFuture<'_, ClientResult<SearchResponse>> {
        Box::pin(async move { Client::search(self, request).await })
    }

    fn search_stream(
        &self,
        request: SearchRequest,
    ) -> BoxFuture<'_, ClientResult<BoxStream<'static, ClientResult<SearchEvent>>>> {
        Box::pin(async move {
            let stream = Client::search_stream(self, request).await?;
            Ok(stream.boxed())
        })
    }
}
