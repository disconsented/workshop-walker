use salvo::prelude::endpoint;
#[tracing::instrument(level = "trace", skip())]
#[endpoint]
pub async fn vote() {}
#[tracing::instrument(level = "trace", skip())]
#[endpoint]
pub async fn remove() {}
#[tracing::instrument(level = "trace", skip())]
#[endpoint]
pub async fn new() {}
