use webarcade_api::prelude::*;

pub async fn register_routes(ctx: &Context) -> Result<()> {
    let mut router = Router::new();

    route!(router, GET "/hello" => handle_hello);

    ctx.register_router("demo", router).await;

    Ok(())
}

async fn handle_hello() -> HttpResponse {
    let response = json!({
        "message": "Hello from demo!"
    });

    json_response(&response)
}
