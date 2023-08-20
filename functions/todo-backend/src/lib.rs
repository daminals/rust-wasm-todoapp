use serde_json::json;
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

#[event(fetch, respond_with_errors)]
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
        .get_async("/api/todo/", |_req, ctx| async move {
            console_log!("Getting todos");
            let todos = get_kv_store!(ctx.kv("todos"));
            let keys_as_key_vectors = match todos.list().execute().await {
                Ok(keys_as_key_vectors) => keys_as_key_vectors.keys,
                Err(_) => {
                    console_log!("Error getting todos");
                    return Response::error("Error getting todos", 500);
                }
            };
            let keys_as_string_vectors = keys_as_key_vectors
                .into_iter()
                .map(|key| key.name)
                .collect::<Vec<_>>();
            let output = json!({ "keys": keys_as_string_vectors });
            console_log!("Got todos: {}", output);
            Response::from_json(&output)
        })
        .post_async("/api/todo/", |mut req, ctx| async move {
            let todos = get_kv_store!(ctx.kv("todos"));
            let body = get_body!(req.json::<serde_json::Value>().await);
            let text = get_text!(body.get("text"));
            // put the text key into the KV store
            match todos.put(text, "") {
                // execute put
                Ok(todo_put_check) => match todo_put_check.execute().await {
                    Ok(result_check) => result_check,
                    Err(_) => {
                        console_log!("Error creating todo: {}", text);
                        return Response::error("Error creating todo", 500);
                    }
                },
                Err(_) => {
                    console_log!("Error creating todo: {}", text);
                    return Response::error("Error creating todo", 500);
                }
            };
            console_log!("Created todo: {}", text);
            Response::ok("ok")
        })
        .delete_async("/api/todo/", |mut req, ctx| async move {
            let todos = get_kv_store!(ctx.kv("todos"));
            let body = get_body!(req.json::<serde_json::Value>().await);
            let text = get_text!(body.get("text"));
            // delete the text key into the KV store
            match todos.delete(text).await {
                Ok(result_check) => result_check,
                Err(_) => {
                    console_log!("Error deleting todo: {}", text);
                    return Response::error("Error deleting todo", 500);
                }
            };
            console_log!("Deleted todo: {}", text);
            Response::ok("ok")
        })
        .run(req, env)
        .await
}

// create a macro that gets a KV store to perform operations on
#[macro_export]
macro_rules! get_kv_store {
    ($expr:expr) => {
        match $expr {
            Ok(todos) => todos,
            Err(_) => {
                console_log!("Error getting todos");
                return Response::error("Error getting todos", 500);
            }
        }
    };
}

// create a macro that gets the body of a request
#[macro_export]
macro_rules! get_body {
    ($expr:expr) => {
        match $expr {
            Ok(body_value) => body_value,
            Err(_) => {
                console_log!("Error getting body");
                return Response::error("Error getting body", 500);
            }
        }
    };
}

// create a macro that gets text from the body
#[macro_export]
macro_rules! get_text {
    ($expr:expr) => {
        match $expr {
            Some(text) => match text.as_str() {
                Some(text) => text,
                None => {
                    console_log!("Error getting text: Nil");
                    return Response::error("Error getting text: Nil", 500);
                }
            },
            None => {
                console_log!("Error getting text");
                return Response::error("Error getting text", 500);
            }
        }
    };
}
