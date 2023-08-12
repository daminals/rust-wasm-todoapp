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
          let todo_result = ctx.kv("todos");
          // check for err
          if todo_result.is_err() {
            console_log!("Error getting todos");
            return Response::error("Error getting todos", 500);
          }
          let todos = todo_result.unwrap();
          let keys_as_key_vectors = todos.list().execute().await;
          // check for err
          if keys_as_key_vectors.is_err() {
            console_log!("Error getting todos");
            return Response::error("Error getting todos", 500);
          }
          let keys_as_key_vectors = keys_as_key_vectors.unwrap().keys;
          let keys_as_string_vectors = keys_as_key_vectors.into_iter().map(|key| key.name).collect::<Vec<_>>();
          let output = json!({ "keys": keys_as_string_vectors });
          console_log!("Got todos: {}", output);
          Response::from_json(&output)
        })
        .post_async("/api/todo/",|mut req, ctx| async move {
          let todo_result = ctx.kv("todos");
          // check for err
          if todo_result.is_err() {
            console_log!("Error getting todos");
            return Response::error("Error getting todos", 500);
          }
          let todos = todo_result.unwrap();
          let body_result = req.json::<serde_json::Value>().await;
          // check for errs
          if body_result.is_err() {
            console_log!("Error getting body");
            return Response::error("Error getting body", 500);
          }
          let body = body_result.unwrap();
          // get the text key from the body
          let text = body.get("text").unwrap().as_str().unwrap();
          // put the text key into the KV store
          let todo_put_check = todos.put(text, "");
          // check for err
          if todo_put_check.is_err() {
            console_log!("Error creating todo: {}", text);
            return Response::error("Error creating todo", 500);
          }
          let result_check = todo_put_check.unwrap().execute().await;
          if result_check.is_err() {
            console_log!("Error creating todo: {}", text);
            return Response::error("Error creating todo", 500);
          }
          console_log!("Created todo: {}", text);
          Response::ok("ok")
        })
        .delete_async("/api/todo/", |mut req, ctx| async move {
          let todo_result = ctx.kv("todos");
          // check for err
          if todo_result.is_err() {
            console_log!("Error getting todos");
            return Response::error("Error getting todos", 500);
          }
          let todos = todo_result.unwrap();
          let body_result = req.json::<serde_json::Value>().await;
          // check for errs
          if body_result.is_err() {
            console_log!("Error getting body");
            return Response::error("Error getting body", 500);
          }
          let body = body_result.unwrap();
          // get the text key from the body
          let text = body.get("text").unwrap().as_str().unwrap();
          // put the text key into the KV store
          let result_check = todos.delete(text).await;
          // check for err
          if result_check.is_err() {
            console_log!("Error deleting todo: {}", text);
            return Response::error("Error deleting todo", 500);
          }
          console_log!("Deleted todo: {}", text);
          Response::ok("ok")
        })
        .run(req, env)
        .await
}
