//! Placeholder endpoints. The router does not mount them yet.
use salvo::prelude::endpoint;

#[allow(
    clippy::unused_async_trait_impl,
    reason = "the body comes with the feature"
)]
#[endpoint]
pub async fn vote() {}

#[allow(
    clippy::unused_async_trait_impl,
    reason = "the body comes with the feature"
)]
#[endpoint]
pub async fn remove() {}

#[allow(
    clippy::unused_async_trait_impl,
    reason = "the body comes with the feature"
)]
#[endpoint]
pub async fn new() {}
