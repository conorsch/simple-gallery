use axum::{response::Html, routing::get, Router};
use clap::Parser;
use simple_gallery::ImageDir;
use simple_gallery::TransitionConfig;
use std::net::SocketAddr;
use std::path::PathBuf;
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
    directory: PathBuf,

    /// Local TCP socket to bind to.
    #[clap(short, long, value_parser, default_value = "127.0.0.1:3000")]
    bind_address: SocketAddr,

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
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let env = Env::default().filter_or("RUST_LOG", "debug,hyper=info");
    env_logger::init_from_env(env);

    let i = ImageDir {
        path: args.directory.clone(),
        file_extension: args.file_extension,
    };
    let imgs = i.find_images();
    let c = TransitionConfig::new(imgs, args.title);
    let html = c.generate_html()?;

    // Dump HTML and exit
    if args.generate {
        debug!(
            "Generating HTML, finding images in {}",
            &args.directory.display()
        );
        println!("{}", html);

    // Otherwise, spin up webserver
    } else {
        // TODO: Reread the directory periodically, so we can find new files
        // without an application restart.
        let app = Router::new()
            // Homepage, auto slideshow from generated html
            .route("/", get(move || async { Html(html) }))
            // Static file server, so images can be loaded from directory
            .nest_service(
                format!("/{}", c.static_route).as_str(),
                ServeDir::new(&args.directory),
            )
            // Direct file loading of a random image from the directory
            .route_service("/random", RandomFileServer::new(i));
        debug!("Starting webserver, binding to {}", args.bind_address);
        let listener = tokio::net::TcpListener::bind(args.bind_address).await?;
        axum::serve(listener, app.into_make_service()).await?;
    }
    Ok(())
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
        debug!("looked up fresh random image {} (in call)", r.display());
        let mut file_server = ServeFile::new(r);
        file_server.call(req)
    }
}
