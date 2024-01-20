use axum::{routing::get, Router};
use clap::Parser;
use simple_gallery::ImageDir;
use simple_gallery::TransitionConfig;
use tower_http::services::ServeDir;
use tower_http::services::ServeFile;

#[macro_use]
extern crate log;
use env_logger::Env;

/// Generates a single-page application, with no JS, serving a simple photogallery
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// On-disk path for directory of images to serve
    #[clap(short, long, value_parser, default_value = "img")]
    directory: String,

    /// TCP port to listen on
    #[clap(short, long, value_parser, default_value_t = 3000)]
    port: u16,

    /// Local IP address to bind to
    #[clap(short, long, value_parser, default_value = "127.0.0.1")]
    bind_address: String,

    /// Title for HTML page, e.g. "example.com"
    #[clap(short, long, value_parser, default_value = "simple-gallery")]
    title: String,

    /// Build static HTML and print to stdout, then exit
    #[clap(short, long, value_parser, default_value_t = false)]
    generate: bool,

    /// Randomize order of images
    #[clap(short, long, value_parser, default_value_t = true)]
    shuffle: bool,

    /// File extension of images, used to filter results.
    #[clap(short, long, default_value = "png")]
    file_extension: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let env = Env::default().filter_or("RUST_LOG", "debug,hyper=info");
    env_logger::init_from_env(env);

    let i = ImageDir {
        path: args.directory.parse().unwrap(),
        file_extension: args.file_extension,
    };
    let imgs = i.find_images();
    let c = TransitionConfig::new(imgs, args.title);
    let html = c.generate_html();

    // Dump HTML and exit
    if args.generate {
        debug!("Generating HTML, finding images in {}", &args.directory);
        println!("{}", html);

    // Otherwise, spin up webserver
    } else {
        // TODO: Reread the directory periodically, so we can find new files
        // without an application restart.
        let image_dir = args.directory;
        let serve_port = args.port;
        let bind_address = args.bind_address;

        let app = Router::new()
            // Homepage, auto slideshow from generated html
            .route("/", get(move || async { html }))
            // Static file server, so images can be loaded from directory
            .nest_service("/img", ServeDir::new(&image_dir))
            // Direct file loading of a random image from the directory
            .route_service("/random", RandomFileServer::new(i));
        let bind_socket = format!("{}:{}", bind_address, serve_port);
        debug!("Starting webserver, binding to {}", bind_socket);
        let listener = tokio::net::TcpListener::bind(bind_socket).await.unwrap();
        axum::serve(listener, app.into_make_service())
            .await
            .unwrap();
    }
}

#[derive(Clone)]
pub struct RandomFileServer(ImageDir);

impl RandomFileServer {
    pub fn new(image_dir: ImageDir) -> Self {
        Self(image_dir)
    }
}

use http::Request;
use std::task::{Context, Poll};
use tower_service::Service;
impl<ReqBody> Service<Request<ReqBody>> for RandomFileServer
where
    ReqBody: Send + 'static,
{
    type Error = <ServeDir as Service<Request<ReqBody>>>::Error;
    type Response = <ServeDir as Service<Request<ReqBody>>>::Response;
    type Future = <ServeDir as Service<Request<ReqBody>>>::Future;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        // let r = get_random_image("img", "jpg");
        let r = self.0.get_random_image();
        debug!("looked up fresh random image {} (in call)", r);
        let mut file_server = ServeFile::new(r);
        file_server.call(req)
    }
}
