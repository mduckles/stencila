use crate::{
    config::CONFIG,
    documents::DOCUMENTS,
    jwt,
    projects::Projects,
    rpc::{self, Error, Request, Response},
    utils::urls,
};
use defaults::Defaults;
use events::{subscribe, Subscriber};
use eyre::{bail, eyre, Result};
use futures::{SinkExt, StreamExt};
use itertools::Itertools;
use jwt::JwtError;
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    env,
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};
use thiserror::private::PathAsDisplay;
use tokio::sync::{mpsc, RwLock};
use warp::{
    http::{
        header::{self, HeaderValue},
        StatusCode,
    },
    ws, Filter, Reply,
};

/// Run a server on this thread
///
/// # Arguments
///
/// - `url`: The URL to listen on
/// - `key`: A secret key for signing and verifying JSON Web Tokens (defaults to random)
/// - `home`: The root directory for files that are served (defaults to current working directory)
/// - `traversal`: Whether traversal out of the root directory is allowed
///
/// # Examples
///
/// Listen on http://0.0.0.0:1234,
///
/// ```no_run
/// # #![recursion_limit = "256"]
/// use stencila::serve::serve;
///
/// serve(Some("http://0.0.0.0:1234".to_string()), None, None, false);
/// ```
#[tracing::instrument]
pub async fn serve(
    url: Option<String>,
    key: Option<String>,
    home: Option<PathBuf>,
    traversal: bool,
) -> Result<()> {
    let (address, port) = match url {
        Some(url) => parse_url(&url)?,
        None => ("127.0.0.1".to_string(), pick_port(9000, 9011)?),
    };
    serve_on(address, port, key, home, traversal).await
}

/// Run a server on another thread
///
/// Arguments as for [`serve`]
#[tracing::instrument]
pub fn serve_background(
    url: Option<String>,
    key: Option<String>,
    home: Option<PathBuf>,
    traversal: bool,
) -> Result<()> {
    // Spawn a thread, start a runtime in it, and serve using that runtime.
    // Any errors within the thread are logged because we can't return a
    // `Result` from the thread to the caller of this function.
    std::thread::spawn(move || {
        let _span = tracing::trace_span!("serve_in_background");

        let runtime = match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime,
            Err(error) => {
                tracing::error!("{}", error.to_string());
                return;
            }
        };
        match runtime.block_on(async { serve(url, key, home, traversal).await }) {
            Ok(_) => {}
            Err(error) => tracing::error!("{}", error.to_string()),
        };
    });

    Ok(())
}

/// Parse a URL into address and port components
pub fn parse_url(url: &str) -> Result<(String, u16)> {
    let url = urls::parse(url)?;
    let address = url.host().unwrap().to_string();
    let port = url
        .port_or_known_default()
        .expect("Should be a default port for the protocol");
    Ok((address, port))
}

/// Pick the first available port from a range, falling back to a random port
/// if none of the ports in the range are available
pub fn pick_port(min: u16, max: u16) -> Result<u16> {
    for port in min..max {
        if portpicker::is_free(port) {
            return Ok(port);
        }
    }
    portpicker::pick_unused_port().ok_or_else(|| eyre!("There are no free ports"))
}

/// Static assets
///
/// During development, these are served from the `static` folder (which
/// has a symlink to `web/dist/browser` (and maybe in the future other folders).
/// At build time these are embedded in the binary. Use `include` and `exclude`
/// glob patterns to only include the assets that are required.
#[cfg(feature = "serve-http")]
#[derive(RustEmbed)]
#[folder = "static"]
#[exclude = "web/*.map"]
struct Static;

/// The version used in URL paths for static assets
/// Allows for caching control (see [`get_static`]).
const STATIC_VERSION: &str = if cfg!(debug_assertions) {
    "dev"
} else {
    env!("CARGO_PKG_VERSION")
};

struct Client {
    /// A list of subscription topics for this client
    subscriptions: HashSet<String>,

    /// The current sender for this client
    ///
    /// This is set / reset each time that the client opens
    /// a websocket connection
    sender: mpsc::UnboundedSender<ws::Message>,
}

impl Client {
    pub fn subscribe(&mut self, topic: &str) -> bool {
        self.subscriptions.insert(topic.to_string())
    }

    pub fn unsubscribe(&mut self, topic: &str) -> bool {
        self.subscriptions.remove(topic)
    }

    // Is a client subscribed to a particular topic, or set of topics?
    pub fn subscribed(&self, topic: &str) -> bool {
        for subscription in &self.subscriptions {
            if subscription == "*" || topic.starts_with(subscription) {
                return true;
            }
        }
        false
    }

    pub fn send(&self, message: impl Serialize) {
        match serde_json::to_string(&message) {
            Ok(json) => self.send_text(&json),
            Err(error) => tracing::error!("Error serializing to JSON `{}`", error),
        }
    }

    pub fn send_text(&self, text: &str) {
        if let Err(error) = self.sender.send(warp::ws::Message::text(text)) {
            tracing::error!("Client send error `{}`", error)
        }
    }
}

/// A store of clients
#[derive(Defaults)]
struct Clients {
    clients: Arc<RwLock<HashMap<String, Client>>>,
}

impl Clients {
    pub fn new() -> Self {
        let clients = Clients::default();

        let (sender, receiver) = mpsc::unbounded_channel::<events::Message>();
        subscribe("*", Subscriber::Sender(sender)).unwrap();
        tokio::spawn(Clients::publish(clients.clients.clone(), receiver));

        clients
    }

    pub async fn connected(&self, id: &str, sender: mpsc::UnboundedSender<ws::Message>) {
        let mut clients = self.clients.write().await;
        match clients.entry(id.to_string()) {
            Entry::Occupied(mut occupied) => {
                tracing::debug!("Re-connection for client `{}`", id);
                let client = occupied.get_mut();
                client.sender = sender;
            }
            Entry::Vacant(vacant) => {
                tracing::debug!("New connection for client `{}`", id);
                vacant.insert(Client {
                    subscriptions: HashSet::new(),
                    sender,
                });
            }
        };
    }

    pub async fn disconnected(&self, id: &str, gracefully: bool) {
        let mut clients = self.clients.write().await;
        clients.remove(id);

        if gracefully {
            tracing::debug!("Graceful disconnection by client `{}`", id)
        } else {
            tracing::warn!("Ungraceful disconnection by client `{}`", id)
        }
    }

    pub async fn send(&self, id: &str, message: impl Serialize) {
        let clients = self.clients.read().await;
        if let Some(client) = clients.get(id) {
            client.send(message);
        } else {
            tracing::error!("No such client `{}`", id);
        }
    }

    pub async fn subscribe(&self, id: &str, topic: &str) {
        let mut clients = self.clients.write().await;
        if let Some(client) = clients.get_mut(id) {
            tracing::debug!("Subscribing client `{}` to topic `{}`", id, topic);
            client.subscribe(topic);
        } else {
            tracing::error!("No such client `{}`", id);
        }
    }

    pub async fn unsubscribe(&self, id: &str, topic: &str) {
        let mut clients = self.clients.write().await;
        if let Some(client) = clients.get_mut(id) {
            tracing::debug!("Unsubscribing client `{}` from topic `{}`", id, topic);
            client.unsubscribe(topic);
        } else {
            tracing::error!("No such client `{}`", id);
        }
    }

    /// Publish events to clients
    ///
    /// The receiver will receive _all_ events that are published and relay them on to
    /// clients based in their subscriptions.
    async fn publish(
        clients: Arc<RwLock<HashMap<String, Client>>>,
        receiver: mpsc::UnboundedReceiver<events::Message>,
    ) {
        let mut receiver = tokio_stream::wrappers::UnboundedReceiverStream::new(receiver);
        while let Some((topic, event)) = receiver.next().await {
            // Get a list of clients that are subscribed to this topic
            let clients = clients.read().await;
            let clients = clients
                .values()
                .filter(|client| client.subscribed(&topic))
                .collect_vec();

            // Skip this event if no one is subscribed
            if clients.is_empty() {
                continue;
            }

            // Create a JSON-RPC notification for the event and serialize it
            // so that does not need to be repeated for each client
            let params = if event.is_object() {
                serde_json::from_value(event).unwrap()
            } else {
                let mut params = HashMap::new();
                params.insert("event".to_string(), event);
                params
            };
            let notification = rpc::Notification::new(&topic, params);
            let json = match serde_json::to_string(&notification) {
                Ok(json) => json,
                Err(error) => {
                    tracing::error!("Error serializing to JSON `{}`", error);
                    continue;
                }
            };

            // Send it!
            for client in clients {
                client.send_text(&json)
            }
        }
    }
}

/// The global clients store
static CLIENTS: Lazy<Clients> = Lazy::new(Clients::new);

/// Run a server
///
/// # Arguments
///
/// - `protocol`: The `Protocol` to serve on (defaults to Websocket)
/// - `address`: The address to listen to (defaults to `127.0.0.1`; only for HTTP and Websocket protocols)
/// - `port`: The port to listen on (defaults to `9000`, only for HTTP and Websocket protocols)
///
/// # Examples
///
/// Listen on both http://127.0.0.1:9000 and ws://127.0.0.1:9000,
///
/// ```no_run
/// # #![recursion_limit = "256"]
/// use stencila::serve::serve_on;
///
/// serve_on("127.0.0.1".to_string(), 9000, None, None, false);
/// ```
#[tracing::instrument]
pub async fn serve_on(
    address: String,
    port: u16,
    key: Option<String>,
    home: Option<PathBuf>,
    traversal: bool,
) -> Result<()> {
    if let Some(key) = key.as_ref() {
        if key.len() > 64 {
            bail!("Server key should be 64 bytes or less")
        }
    }

    let home = match home {
        Some(home) => home.canonicalize()?,
        None => Projects::current_path()?,
    };

    let mut url = format!("http://{}:{}", address, port);
    if let Some(key) = &key {
        // Provide the user with a long expiring token so they can access the server.
        let token = jwt::encode(key, Some(home.display().to_string()), Some(3600))?;
        url.push_str("?token=");
        url.push_str(&token);
    }
    tracing::info!("Serving {} at {}", home.display(), url);

    // Static files (assets embedded in binary for which authentication is not required)

    let statics = warp::get()
        .and(warp::path("~static"))
        .and(warp::path::tail())
        .and_then(get_static);

    // The following HTTP and WS endpoints all require authentication

    let authenticate = || authentication_filter(key.clone());

    let ws = warp::path("~ws")
        .and(warp::ws())
        .and(warp::query::<WsParams>())
        .and(authenticate())
        .map(ws_handshake);

    let get = warp::get()
        .and(warp::path::full())
        .and(warp::query::<GetParams>())
        .and(warp::any().map(move || (home.clone(), traversal)))
        .and(authenticate())
        .and_then(get_handler);

    let post = warp::post()
        .and(warp::path::end())
        .and(warp::body::json::<Request>())
        .and(authenticate())
        .and_then(post_handler);

    let post_wrap = warp::post()
        .and(warp::path::param())
        .and(warp::body::json::<serde_json::Value>())
        .and(authenticate())
        .and_then(post_wrap_handler);

    // Custom `server` header
    let server = warp::reply::with::default_header(
        "server",
        format!(
            "Stencila/{} ({})",
            env!("CARGO_PKG_VERSION"),
            env::consts::OS
        ),
    );

    // CORS headers to allow from any origin
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec![
            "Content-Type",
            "Referer", // Note that this is an intentional misspelling!
            "Origin",
            "Access-Control-Allow-Origin",
        ])
        .allow_methods(&[warp::http::Method::GET, warp::http::Method::POST])
        .max_age(24 * 60 * 60);

    let routes = statics
        .or(ws)
        .or(get)
        .or(post)
        .or(post_wrap)
        .with(server)
        .with(cors)
        .recover(rejection_handler);

    // Use `try_bind_ephemeral` here to avoid potential panic when using `run`
    let address: std::net::IpAddr = address.parse()?;
    let (_address, future) = warp::serve(routes).try_bind_ephemeral((address, port))?;
    future.await;

    Ok(())
}

/// Return an error response
///
/// Used to have a consistent structure to error responses in the
/// handler functions below.
#[allow(clippy::unnecessary_wraps)]
fn error_response(
    code: StatusCode,
    message: &str,
) -> Result<warp::reply::Response, std::convert::Infallible> {
    Ok(warp::reply::with_status(
        warp::reply::json(&serde_json::json!({ "message": message })),
        code,
    )
    .into_response())
}

/// Handle a HTTP `GET` request to the `/~static/` path
///
/// This path includes the current version number e.g. `/~static/0.127.0`. This
/// allows a `Cache-Control` header with long `max-age` and `immutable` (so that browsers do not
/// fetch / parse assets on each request) while also causing the browser cache to be busted for
/// each new version of Stencila. During development, the version is set to "dev" and the cache control
/// header is not set (for automatic reloading of re-built assets etc).
#[tracing::instrument]
async fn get_static(
    path: warp::path::Tail,
) -> Result<warp::reply::Response, std::convert::Infallible> {
    let path = path.as_str().to_string();
    tracing::debug!("GET ~static /{}", path);

    // Remove the version number with warnings if it is not present
    // or different to current version
    let parts = path.split('/').collect_vec();
    let path = if parts.len() < 2 {
        tracing::warn!("Expected path to have at least two parts");
        path
    } else {
        let version = parts[0];
        if version != STATIC_VERSION {
            tracing::warn!(
                "Requested static assets for a version `{}` not equal to current `{}`",
                version,
                STATIC_VERSION
            );
        }
        parts[1..].join("/")
    };

    let asset = match Static::get(&path) {
        Some(asset) => asset,
        None => return error_response(StatusCode::NOT_FOUND, "Requested path does not exist"),
    };

    let mut response = warp::reply::Response::new(asset.data.into());

    let mime = mime_guess::from_path(path).first_or_octet_stream();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(mime.as_ref()).unwrap(),
    );

    let cache_control = if STATIC_VERSION == "dev" {
        "no-cache"
    } else {
        "max-age=31536000, immutable"
    };
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_str(cache_control).unwrap(),
    );

    Ok(response)
}

/// Query parameters for `auth_filter`
#[derive(Deserialize)]
struct AuthParams {
    pub token: Option<String>,
}

/// A Warp filter that extracts any JSON Web Token from a `token` query parameter, `Authorization` header
/// or `token` cookie.
fn authentication_filter(
    key: Option<String>,
) -> impl Filter<Extract = ((jwt::Claims, Option<String>),), Error = warp::Rejection> + Clone {
    warp::query::<AuthParams>()
        .and(warp::header::optional::<String>("authorization"))
        .and(warp::cookie::optional("token"))
        .map(
            move |query: AuthParams, header: Option<String>, cookie: Option<String>| {
                (key.clone(), query.token, header, cookie)
            },
        )
        .and_then(
            |(key, param, header, cookie): (
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
            )| async move {
                if let Some(key) = key {
                    // Key present, so check for valid token as a query parameter, authorization header,
                    // or cookie (in that order of precedence).

                    let claims = if let Some(param) = param {
                        jwt::decode(&param, &key)
                    } else {
                        Err(JwtError::NoTokenSupplied)
                    };

                    let claims = if let (Err(..), Some(header)) = (&claims, header) {
                        jwt::from_auth_header(header).and_then(|token| jwt::decode(&token, &key))
                    } else {
                        claims
                    };

                    let (claims, from_cookie) = if let (Err(..), Some(cookie)) = (&claims, cookie) {
                        let claims = jwt::decode(&cookie, &key);
                        let ok = claims.is_ok();
                        (claims, ok)
                    } else {
                        (claims, false)
                    };

                    let claims = match claims {
                        Ok(claims) => claims,
                        Err(error) => return Err(warp::reject::custom(error)),
                    };

                    // Set a `token` cookie if the claims did not come from a cookie
                    let cookie = if !from_cookie {
                        const EXPIRY_SECONDS: i64 = 30 * 24 * 60 * 60;
                        let token =
                            jwt::encode(&key, claims.scope.clone(), Some(EXPIRY_SECONDS)).unwrap();
                        Some(format!(
                            "token={}; Max-Age={}; SameSite; HttpOnly",
                            token, EXPIRY_SECONDS
                        ))
                    } else {
                        None
                    };

                    Ok((claims, cookie))
                } else {
                    // No key, so in insecure mode. Return a permissive set of claims and
                    // no cookie.
                    Ok((
                        jwt::Claims {
                            exp: 0,
                            scope: None,
                        },
                        None,
                    ))
                }
            },
        )
}

/// Query parameters for `get_handler`
#[derive(Debug, Deserialize)]
struct GetParams {
    /// The mode "read", "view", "exec", or "edit"
    mode: Option<String>,

    /// The format to view or edit
    format: Option<String>,

    /// The theme (when format is `html`)
    theme: Option<String>,

    /// Should web components be loaded
    components: Option<String>,

    /// An authentication token
    ///
    /// Only used here to determine whether to redirect but used in `authentication_filter`
    /// for actual authentication.
    token: Option<String>,
}

/// Handle a HTTP `GET` request for a document
///
/// If the requested path starts with `/static` or is not one of the registered file types,
/// then returns the static asset with the `Content-Type` header set.
/// Otherwise, if the requested `Accept` header includes "text/html", viewer's index.html is
/// returned (which, in the background will request the document as JSON). Otherwise,
/// will attempt to determine the desired format from the `Accept` header and convert the
/// document to that.
#[tracing::instrument]
async fn get_handler(
    path: warp::path::FullPath,
    params: GetParams,
    (home, traversal): (PathBuf, bool),
    (claims, cookie): (jwt::Claims, Option<String>),
) -> Result<warp::reply::Response, std::convert::Infallible> {
    let path = path.as_str();
    tracing::debug!("GET {}", path);

    let filesystem_path = Path::new(path.strip_prefix('/').unwrap_or(path));
    let filesystem_path = match home.join(filesystem_path).canonicalize() {
        Ok(filesystem_path) => filesystem_path,
        Err(_) => return error_response(StatusCode::NOT_FOUND, "Requested path does not exist"),
    };

    if !traversal && filesystem_path.strip_prefix(&home).is_err() {
        return error_response(
            StatusCode::FORBIDDEN,
            "Traversal outside of server's home directory is not permitted",
        );
    }

    if let Some(scope) = claims.scope {
        if filesystem_path.strip_prefix(&scope).is_err() {
            return error_response(
                StatusCode::FORBIDDEN,
                "Insufficient permissions to access this directory or file",
            );
        }
    }

    let format = params.format.unwrap_or_else(|| "html".into());
    let mode = params.mode.unwrap_or_else(|| "view".into());
    let theme = params.theme.unwrap_or_else(|| "wilmore".into());
    let components = params.components.unwrap_or_else(|| "static".into());

    let (content, mime, redirect) = if params.token.is_some() {
        // A token is in the URL. For address bar aesthetics, and to avoid confusion (the token may
        // not be suitable for reuse), redirect to a token-less URL. Note that we set a token cookie
        // below to replace the URL-based token.
        (
            html_page_redirect(path).as_bytes().to_vec(),
            "text/html".to_string(),
            true,
        )
    } else if filesystem_path.is_dir() {
        // Request for a path that is a folder. Return a listing
        (
            html_directory_listing(&home, &filesystem_path)
                .as_bytes()
                .to_vec(),
            "text/html".to_string(),
            false,
        )
    } else if format == "raw" {
        // Request for raw content of the file (e.g. an image within the HTML encoding of a
        // Markdown document)
        let content = match fs::read(&filesystem_path) {
            Ok(content) => content,
            Err(error) => {
                return error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    &format!("When reading file `{}`", error),
                )
            }
        };

        let mime = mime_guess::from_path(filesystem_path).first_or_octet_stream();

        (content, mime.to_string(), false)
    } else {
        // Request for a document in some format (usually HTML)
        match DOCUMENTS.open(&filesystem_path, None).await {
            Ok(document) => {
                let document = DOCUMENTS.get(&document.id).await.unwrap();
                let document = document.lock().await;
                let content = match document.dump(Some(format.clone())).await {
                    Ok(content) => content,
                    Err(error) => {
                        return error_response(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            &format!("While converting document to {} `{}`", format, error),
                        )
                    }
                };

                let content = match format.as_str() {
                    "html" => html_rewrite(
                        &content,
                        &mode,
                        &theme,
                        &components,
                        &home,
                        &filesystem_path,
                    ),
                    _ => content,
                }
                .as_bytes()
                .to_vec();

                let mime = mime_guess::from_ext(&format).first_or_octet_stream();

                (content, mime.to_string(), false)
            }
            Err(error) => {
                return error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    &format!("While opening document `{}`", error),
                )
            }
        }
    };

    let mut response = warp::reply::Response::new(content.into());

    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_str(&mime).unwrap());

    if redirect {
        *response.status_mut() = StatusCode::MOVED_PERMANENTLY;
        response
            .headers_mut()
            .insert(header::LOCATION, HeaderValue::from_str(path).unwrap());
    }

    if let Some(cookie) = cookie {
        response
            .headers_mut()
            .insert(header::SET_COOKIE, HeaderValue::from_str(&cookie).unwrap());
    }

    Ok(response)
}

/// Generate HTML for a page redirect
///
/// Although the MOVED_PERMANENTLY status code should trigger the redirect, this
/// provides HTML / JavaScript fallbacks.
fn html_page_redirect(path: &str) -> String {
    format!(
        r#"<!DOCTYPE HTML>
<html lang="en-US">
<head>
    <title>Redirecting</title>
    <meta charset="UTF-8">
    <meta http-equiv="refresh" content="0; url={}">
    <script type="text/javascript">window.location.href = "{}"</script>
</head>
<body>If you are not redirected automatically, please follow this <a href="{}">link</a>.</body>
</html>"#,
        path, path, path
    )
}

/// Generate HTML for a directory listing
///
/// Note: If the `dir` is outside of `home` (i.e. traversal was allowed) then
/// no entries will be shown.
fn html_directory_listing(home: &Path, dir: &Path) -> String {
    let entries = match dir.read_dir() {
        Ok(entries) => entries,
        Err(error) => {
            // This should be an uncommon error but to avoid an unwrap...
            tracing::error!("{}", error);
            return "<p>Something went wrong</p>".to_string();
        }
    };
    entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();

            let href = match path.strip_prefix(home) {
                Ok(href) => href,
                Err(..) => return None,
            };

            let name = match path.strip_prefix(dir) {
                Ok(name) => name,
                Err(..) => return None,
            };

            Some(format!(
                "<p><a href=\"/{}\">{}</a></p>",
                href.display(),
                name.display()
            ))
        })
        .collect::<Vec<String>>()
        .concat()
}

/// Rewrite HTML to serve local files and wrap with desired theme etc.
///
/// Only local files somewhere withing the current working directory are
/// served.
pub fn html_rewrite(
    body: &str,
    mode: &str,
    theme: &str,
    components: &str,
    home: &Path,
    document: &Path,
) -> String {
    let static_root = ["/~static/", STATIC_VERSION].concat();

    // Head element for theme
    let themes = format!(
        r#"<link href="{static_root}/themes/themes/{theme}/styles.css" rel="stylesheet">"#,
        static_root = static_root,
        theme = theme
    );

    // Head elements for web client
    let web = format!(
        r#"
    <link href="{static_root}/web/{mode}.css" rel="stylesheet">
    <script src="{static_root}/web/{mode}.js"></script>
    <script>
        const startup = stencilaWebClient.main("{url}", "{client}", "{project}", "{snapshot}", "{document}");
        startup().catch((err) => console.error('Error during startup', err))
    </script>"#,
        static_root = static_root,
        mode = mode,
        // TODO: pass url from outside this function?
        url = "ws://127.0.0.1:9000/~ws",
        client = uuid_utils::generate("cl"),
        project = "current",
        snapshot = "current",
        document = document.as_display().to_string()
    );

    // Head elements for web components
    let components = match components {
        "none" => "".to_string(),
        _ => {
            let base = match components {
                "remote" => {
                    "https://unpkg.com/@stencila/components/dist/stencila-components".to_string()
                }
                _ => [&static_root, "/components"].concat(),
            };
            format!(
                r#"
                <script src="{}/stencila-components.esm.js" type="module"> </script>
                <script src="{}/stencila-components.js" type="text/javascript" nomodule=""> </script>
                "#,
                base, base
            )
        }
    };

    // Rewrite body content so that links to files work
    static REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#""file://(.*?)""#).expect("Unable to create regex"));

    let body = REGEX.replace_all(body, |captures: &Captures| {
        let path = captures
            .get(1)
            .expect("Should always have first capture")
            .as_str();
        let path = match Path::new(path).canonicalize() {
            Ok(path) => path,
            // Redact the path if it can not be canonicalized
            Err(_) => return "\"\"".to_string(),
        };
        match path.strip_prefix(home) {
            Ok(path) => ["\"/", &path.display().to_string(), "?format=raw\""].concat(),
            // Redact the path if it is outside of the current directory
            Err(_) => "\"\"".to_string(),
        }
    });

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        {themes}
        {web}
        {components}
    </head>
    <body>
        {body}
    </body>
</html>"#,
        themes = themes,
        web = web,
        components = components,
        body = body
    )
}

/// Handle a HTTP `POST /` request
async fn post_handler(
    request: Request,
    (_claims, _cookie): (jwt::Claims, Option<String>),
) -> Result<impl warp::Reply, std::convert::Infallible> {
    let (response, ..) = request.dispatch("http").await;
    Ok(warp::reply::json(&response))
}

/// Handle a HTTP `POST /<method>` request
async fn post_wrap_handler(
    method: String,
    params: serde_json::Value,
    (_claims, _cookie): (jwt::Claims, Option<String>),
) -> Result<impl warp::Reply, std::convert::Infallible> {
    use warp::reply;

    // Wrap the method and parameters into a request
    let request = serde_json::from_value::<Request>(serde_json::json!({
        "method": method,
        "params": params
    }));
    let request = match request {
        Ok(request) => request,
        Err(error) => {
            return Ok(reply::with_status(
                reply::json(&serde_json::json!({
                    "message": error.to_string()
                })),
                StatusCode::BAD_REQUEST,
            ))
        }
    };

    // Unwrap the response into results or error message
    let (Response { result, error, .. }, ..) = request.dispatch("http").await;
    let reply = match result {
        Some(result) => reply::with_status(reply::json(&result), StatusCode::OK),
        None => match error {
            Some(error) => reply::with_status(reply::json(&error), StatusCode::BAD_REQUEST),
            None => reply::with_status(
                reply::json(&serde_json::json!({
                    "message": "Response had neither a result nor an error"
                })),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        },
    };
    Ok(reply)
}

/// Parameters for the WebSocket handshake
#[derive(Debug, Deserialize)]
struct WsParams {
    client: String,
}

/// Perform a WebSocket handshake / upgrade
///
/// This function is called at the start of a WebSocket connection.
#[tracing::instrument]
fn ws_handshake(
    ws: warp::ws::Ws,
    params: WsParams,
    (_claims, _cookie): (jwt::Claims, Option<String>),
) -> impl warp::Reply {
    tracing::debug!("WebSocket handshake");
    ws.on_upgrade(|socket| ws_connected(socket, params.client))
}

/// Handle a WebSocket connection
///
/// This function is called after the handshake, when a WebSocket client
/// has successfully connected.
#[tracing::instrument(skip(socket))]
async fn ws_connected(socket: warp::ws::WebSocket, client: String) {
    tracing::debug!("WebSocket connected");

    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the client's websocket.
    let (client_sender, client_receiver) = mpsc::unbounded_channel();
    let mut client_receiver = tokio_stream::wrappers::UnboundedReceiverStream::new(client_receiver);

    let client_clone = client.clone();
    tokio::task::spawn(async move {
        while let Some(message) = client_receiver.next().await {
            if let Err(error) = ws_sender.send(message).await {
                let message = error.to_string();
                if message == "Connection closed normally" {
                    CLIENTS.disconnected(&client_clone, true).await
                } else {
                    tracing::error!("Websocket send error `{}`", error);
                }
            }
        }
    });

    // Save / update the client
    CLIENTS.connected(&client, client_sender).await;

    while let Some(result) = ws_receiver.next().await {
        // Get the message
        let message = match result {
            Ok(message) => message,
            Err(error) => {
                let message = error.to_string();
                if message == "WebSocket protocol error: Connection reset without closing handshake"
                {
                    CLIENTS.disconnected(&client, false).await
                } else {
                    tracing::error!("Websocket receive error `{}`", error);
                }
                continue;
            }
        };

        // Parse the message as a string, skipping non-text messages
        let json = if let Ok(string) = message.to_str() {
            string
        } else {
            continue;
        };

        // Parse the message, returning an error to the client if that fails
        let request = match serde_json::from_str::<rpc::Request>(json) {
            Ok(request) => request,
            Err(error) => {
                let error = rpc::Error::parse_error(&error.to_string());
                let response = rpc::Response::new(None, None, Some(error));
                CLIENTS.send(&client, response).await;
                continue;
            }
        };

        // Dispatch the request and send back the response and update subscriptions
        let (response, subscription) = request.dispatch(&client).await;
        CLIENTS.send(&client, response).await;
        match subscription {
            rpc::Subscription::Subscribe(topic) => {
                CLIENTS.subscribe(&client, &topic).await;
            }
            rpc::Subscription::Unsubscribe(topic) => {
                CLIENTS.unsubscribe(&client, &topic).await;
            }
            rpc::Subscription::None => (),
        }
    }

    // Record that the client has diconnected gracefully
    CLIENTS.disconnected(&client, true).await
}

/// Handle a rejection by converting into a JSON-RPC response
///
/// The above handlers can not handle all errors, in particular, they do not
/// handle JSON parsing errors (which are rejected by the `warp::body::json` filter).
/// This therefore ensures that any request expecting a JSON-RPC response, will get
/// a JSON-RPC response (in these cases containing and error code and message).
#[tracing::instrument]
async fn rejection_handler(
    rejection: warp::Rejection,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    let error = if let Some(error) = rejection.find::<jwt::JwtError>() {
        Error::invalid_request_error(&format!("{}", error))
    } else if let Some(error) = rejection.find::<warp::filters::body::BodyDeserializeError>() {
        Error::invalid_request_error(&format!("{}", error))
    } else if rejection.find::<warp::reject::MethodNotAllowed>().is_some() {
        Error::invalid_request_error("Invalid HTTP method and/or path")
    } else {
        Error::server_error("Unknown error")
    };

    tracing::error!("{:?}", error);

    Ok(warp::reply::with_status(
        warp::reply::json(&Response {
            error: Some(error),
            ..Default::default()
        }),
        StatusCode::BAD_REQUEST,
    ))
}

pub mod config {
    use defaults::Defaults;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};
    use serde_with::skip_serializing_none;
    use validator::Validate;

    /// Server
    ///
    /// Configuration settings for running as a server
    #[skip_serializing_none]
    #[derive(Debug, Defaults, PartialEq, Clone, JsonSchema, Deserialize, Serialize, Validate)]
    #[serde(default)]
    #[schemars(deny_unknown_fields)]
    pub struct ServeConfig {
        /// The URL to serve on
        #[validate(url(message = "Not a valid URL"))]
        pub url: Option<String>,

        /// Secret key to use for signing and verifying JSON Web Tokens
        pub key: Option<String>,

        /// Do not require a JSON Web Token to access the server
        #[def = "false"]
        pub insecure: bool,
    }
}

#[cfg(feature = "cli")]
pub mod commands {
    use std::path::PathBuf;

    use super::*;
    use async_trait::async_trait;
    use cli_utils::{result, Result, Run};
    use structopt::StructOpt;

    /// Serve over HTTP and WebSockets
    ///
    /// ## Ports and addresses
    ///
    /// Use the <url> argument to change the port and/or address that the server
    /// listens on. This argument can be a partial, or complete, URL.
    ///
    /// For example, to serve on port 8000 instead of the default port,
    ///
    ///    stencila serve :8000
    ///
    /// To serve on all IPv4 addresses on the machine, instead of only `127.0.0.1`,
    ///
    ///    stencila serve 0.0.0.0
    ///
    /// Or if you prefer, use a complete URL including the scheme e.g.
    ///
    ///   stencila serve http://127.0.0.1:9000
    ///
    /// ## Security
    ///
    /// By default, the server requires authentication using JSON Web Token. A token is
    /// printed as part of the server's URL at startup. To turn authorization off, for example
    /// if you are using some other authentication layer in front of the server, use the `--insecure`
    /// flag.
    ///
    /// By default, this command will NOT run as a root (Linux/Mac OS/Unix) or administrator (Windows) user.
    /// Use the `--root` option, with extreme caution, to allow to be run as root.
    ///
    /// Most of these options can be set in the Stencila configuration file. See `stencila config get serve`
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Command {
        /// The home directory for the server to serve from
        ///
        /// Defaults to the current directory or an ancestor project directory (if the current directory
        /// is within a project).
        home: Option<PathBuf>,

        /// The URL to serve on
        ///
        /// Defaults to the `STENCILA_URL` environment variable, the value set in config
        /// or otherwise `http://127.0.0.1:9000`.
        #[structopt(short, long, env = "STENCILA_URL")]
        url: Option<String>,

        /// Secret key to use for signing and verifying JSON Web Tokens
        ///
        /// Defaults to the `STENCILA_KEY` environment variable, the value set in config
        /// or otherwise a randomly generated value.
        #[structopt(short, long, env = "STENCILA_KEY")]
        key: Option<String>,

        /// Serve in a background thread (when in interactive mode)
        #[structopt(short, long)]
        background: bool,

        /// Do not require a JSON Web Token to access the server
        ///
        /// For security reasons (any client can access files and execute code) this should be avoided.
        #[structopt(long)]
        insecure: bool,

        /// Allow traversal out of the server's home directory
        ///
        /// For security reasons (clients can access any file on the filesystem) this should be avoided.
        #[structopt(long)]
        traversal: bool,

        /// Allow root (Linux/Mac OS/Unix) or administrator (Windows) user to serve
        ///
        /// For security reasons (clients may be able to execute code as root) this should be avoided.
        #[structopt(long)]
        root: bool,
    }
    #[async_trait]
    impl Run for Command {
        async fn run(&self) -> Result {
            let config = &CONFIG.lock().await.serve;

            let url = match &self.url {
                Some(url) => Some(url.clone()),
                None => config.url.clone(),
            };

            // Get key configured on command line or config file
            let key = match &self.key {
                Some(key) => {
                    tracing::warn!("Server key set on command line can be sniffed by malicious processes; prefer to set it in config file.");
                    Some(key.clone())
                }
                None => config.key.clone(),
            };

            // Check that user is explicitly allowing no key to be used
            let insecure = self.insecure || config.insecure;
            if insecure {
                tracing::warn!("Serving in insecure mode is dangerous and discouraged.")
            }

            // Generate key if necessary
            let key = if key.is_none() {
                match insecure {
                    true => None,
                    false => Some(key_utils::generate()),
                }
            } else {
                key
            };

            // Warn about use of traversal option
            if self.traversal {
                tracing::warn!("Allowing traversal out of server home directory.")
            }

            // Check for root usage
            #[cfg(any(target_os = "linux", target_os = "macos"))]
            if let sudo::RunningAs::Root = sudo::check() {
                if self.root {
                    tracing::warn!("Serving as root/administrator is dangerous and discouraged.")
                } else {
                    bail!("Serving as root/administrator is not permitted by default, use the `--root` option to bypass this safety measure.")
                }
            }

            // If stdout is not a TTY then print the login URL to stdout so that it can be used
            // by, for example, the parent process.
            // TODO: Consider re-enabling this when/id `cli` modules are moved to the `cli` crate
            // where the `atty` crate is available. Until then skip to avoid noise on stdout.
            // println!("{}", login_url(port, key.clone(), Some(300), None)?);

            let home = self.home.clone();
            if self.background {
                super::serve_background(url, key, home, self.traversal)?;
            } else {
                super::serve(url, key, home, self.traversal).await?;
            }

            result::nothing()
        }
    }
}
