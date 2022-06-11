use worker::*;

mod utils;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or("unknown region".into())
    );
}

const none = Url::parse("https://www.google.com/")?;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    // Optionally, use the Router to handle matching endpoints, use ":name" placeholders, or "*name"
    // catch-alls to match on specific patterns. Alternatively, use `Router::with_data(D)` to
    // provide arbitrary data that will be accessible in each route via the `ctx.data()` method.
    let router = Router::new();

    // Add as many routes as your Worker needs! Each route will get a `Request` for handling HTTP
    // functionality and a `RouteContext` which you can use to  and get route parameters and
    // Environment bindings like KV Stores, Durable Objects, Secrets, and Variables.
    router
        .get("", |_, _| {
            Response::redirect_with_status(Url::parse("https://www.google.com/")?, 301)
        })
        .get("/", |_, _| {
            Response::redirect_with_status(Url::parse("https://www.google.com/")?, 301)
        })
        .get_async("/:key", |_, ctx| async move {
            let key = match ctx.param("key") {
                Some(key) => key,
                None => "",
            };

            let kv = match ctx.kv("SHORT_URL") {
                Ok(kv) => kv,
                Err(e) => return Response::error(e.to_string(), 500),
            };

            let value = match kv.get(key).text().await {
                Ok(value) => value,
                Err(_) => {
                    return Response::redirect_with_status(
                        Url::parse("https://www.google.com/")?,
                        301,
                    )
                }
            };

            match value {
                Some(url) => return Response::redirect_with_status(Url::parse(&url)?, 301),
                None => {
                    return Response::redirect_with_status(
                        Url::parse("https://www.google.com/")?,
                        301,
                    )
                }
            }
        })
        .run(req, env)
        .await
}
