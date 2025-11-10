use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use crate::core::router_utils::*;
use crate::route;
use anyhow::Result;
use http_body_util::combinators::BoxBody;
use hyper::{body::Bytes, body::Incoming, Request, Response, StatusCode};
use std::convert::Infallible;

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    route!(router, GET "/hello" => handle_hello);

    ctx.register_router("test", router).await;

    Ok(())
}

async fn handle_hello() -> Response<BoxBody<Bytes, Infallible>> {
    let response = serde_json::json!({
        "message": "Hello from test!"
    });

    json_response(&response)
}