use walkdir::WalkDir;
mod lib;
use crate::lib::TransitionConfig;
use axum::{response::Html, routing::get, Router};
use axum_extra::routing::SpaRouter;
use clap::Parser;
use rand::seq::SliceRandom;
use rand::thread_rng;


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
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let env = Env::default().filter_or("RUST_LOG", "debug,hyper=info");
    env_logger::init_from_env(env);

    // Dump HTML and exit
    if args.generate {
        let image_dir = args.directory;
        debug!("Generating HTML, finding images in {}", image_dir);
        let imgs = find_images(&image_dir, args.shuffle);
        let c = TransitionConfig::new(imgs, args.title);
        let html = c.generate_html();
        println!("{}", html);

    // Otherwise, spin up webserver
    } else {
        // TODO: Reread the directory periodically, so we can find new files
        // without an application restart.
        let image_dir = args.directory;
        let serve_port = args.port;
        let bind_address = args.bind_address;
        // Create a single-page application (SPA) router for serving static images.
        let spa = SpaRouter::new("/img", &image_dir);
        let imgs = find_images(&image_dir, args.shuffle);
        let c = TransitionConfig::new(imgs, args.title);
        let html = Html(c.generate_html());

        let app = Router::new()
            .merge(spa)
            .route("/", get(move || async { html }));
        let bind_socket = format!("{}:{}", bind_address, serve_port);
        debug!("Starting webserver, binding to {}", bind_socket);
        axum::Server::bind(&bind_socket.parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap();
    }
}

fn find_images(image_dir: &str, shuffle: bool) -> Vec<String> {
    let mut img_files: Vec<String> = Vec::new();
    for ent in WalkDir::new(image_dir).into_iter().flatten() {
        let path = ent.path();
        if path.display().to_string().ends_with(".png") {
            img_files.push(path.display().to_string());
        }
    }
    if shuffle {
        let mut rng = thread_rng();
        img_files.shuffle(&mut rng);
    }

    img_files
}
