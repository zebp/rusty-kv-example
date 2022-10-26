use serde::{Deserialize, Serialize};
use worker::{*, kv::KvError};

mod utils;

// This is the name of the KV store binding that we specified in our wrangler.toml file.
const KV_BINDING_NAME: &str = "KV_STORE";

/// Let's pretend we have some important metadata we want to store along side our keys, so we'll
/// just use the amazing [serde](https://docs.rs/serde) library add serialization support for
/// our metadata struct.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExampleMetadata {
    // For our metadata, let's store the content-type the user specified when putting a key.
    content_type: String,
}

async fn list(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // Load the KV store binding by the name we specified above.
    let store = ctx.kv(KV_BINDING_NAME)?;

    // Read any options we'd like to do to configure our list.
    let url = req.url()?;
    let limit = utils::param_from(&url, "limit")
        .and_then(|limit_str| limit_str.parse().ok())
        .unwrap_or(100);
    let prefix = utils::param_from(&url, "prefix")
        .map(String::from)
        .unwrap_or_default();

    let list = store.list().limit(limit).prefix(prefix).execute().await?;
    Response::from_json(&list)
}

async fn put(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let store = ctx.kv(KV_BINDING_NAME)?;
    let key = ctx.param("key").unwrap();
    let body = req.bytes().await?;

    // Let's store the content-type in our metadata, defaulting to data/binary if none was sent.
    let content_type = req
        .headers()
        .get("content-type")?
        .unwrap_or_else(|| "data/binary".into());

    store
        .put_bytes(key, &body)?
        .metadata(ExampleMetadata { content_type })?
        .execute()
        .await?;

    Response::ok("inserted")
}

async fn get(_: Request, ctx: RouteContext<()>) -> Result<Response> {
    let store = ctx.kv(KV_BINDING_NAME)?;
    let key = ctx.param("key").unwrap();

    // In our store we might have the key and that key might have metadata, so we need to check.
    let (maybe_value, maybe_metadata) = store
        .get(key)
        .bytes_with_metadata::<ExampleMetadata>()
        .await?;

    let (value, metadata) = match (maybe_value, maybe_metadata) {
        (Some(value), Some(metadata)) => (value, metadata),
        // Our KV store might have that key, but no metadata associated. So we'll just return a 500
        // as we should never get into this state unless the store is manipulated manually.
        (Some(_), None) => return Response::error("no metadata found", 500),
        _ => return Response::error("key not found", 404),
    };

    // Let's return a body containing the bytes in the KV store with a content-type header from our
    // metadata.
    Ok(Response::from_bytes(value)?.with_headers({
        let mut headers = Headers::default();
        headers.append("content-type", &metadata.content_type)?;
        headers
    }))
}

async fn delete(_: Request, ctx: RouteContext<()>) -> Result<Response> {
    let store = ctx.kv(KV_BINDING_NAME)?;
    let key = ctx.param("key").unwrap();

    store.delete(key).await?;

    Response::ok("deleted")
}

#[derive(Debug, Serialize, Deserialize)]
struct StructuredValue {
    foo: String,
    bar: i32,
}

/// We might not always want to deal with having vague data types in our KV store we can use serde
/// to write any serializable type to a key. So let's just only put the request body in the store
/// if it matches the schema for [StructuredValue].
async fn structured_put(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let store = ctx.kv(KV_BINDING_NAME)?;
    let key = ctx.param("key").unwrap();

    let url = req.url()?;
    let body: StructuredValue = match req.json().await {
        Ok(body) => body,
        // Reject all requests that don't follow our body schema.
        Err(_) => return Response::error("invalid body", 400),
    };

    // Let's add a expiration ttl if the user specifies one.
    if let Some(ttl_str) = utils::param_from(&url, "ttl") {
        let ttl = match ttl_str.parse() {
            Ok(ttl) => ttl,
            Err(_) => return Response::error("invalid ttl", 400),
        };

        store.put(key, &body)?.expiration_ttl(ttl).execute().await?
    } else {
        store.put(key, &body)?.execute().await?
    }

    Response::ok("inserted")
}

async fn structured_get(_: Request, ctx: RouteContext<()>) -> Result<Response> {
    let store = ctx.kv(KV_BINDING_NAME)?;
    let key = ctx.param("key").unwrap();

    match store.get(key).json::<StructuredValue>().await {
        Ok(Some(value)) => Response::from_json(&value),
        Ok(None) => Response::error("key not found", 404),
        // The key might have already been inserted with out non-structured put endpoint, so let's
        // pretend it doesn't exist if it's invalid.
        Err(KvError::Serialization(_)) => Response::error("key not found", 404),
        Err(_) => Response::error("internal server error", 500),
    }
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    utils::log_request(&req);

    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    // We can use a Router to route our incoming requests to our handlers, using `:param` syntax to
    // add URL patterns or `*name` for catch-alls.
    Router::new()
        .get_async("/list", list)
        .put_async("/:key", put)
        .get_async("/:key", get)
        .delete_async("/:key", delete)
        .put_async("/structured/:key", structured_put)
        .get_async("/structured/:key", structured_get)
        .run(req, env)
        .await
}
