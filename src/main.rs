use walkdir::WalkDir;
use simple_gallery::TransitionConfig;
use axum::{response::Html, routing::get, Router};
// use axum_extra::routing::SpaRouter;
use clap::Parser;
use rand::seq::SliceRandom;
use rand::thread_rng;
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

    // Dump HTML and exit
    if args.generate {
        let image_dir = args.directory;
        debug!("Generating HTML, finding images in {}", image_dir);
        let imgs = find_images(&image_dir, &args.file_extension, args.shuffle);
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
        // let spa = SpaRouter::new("/img", &image_dir);
        let imgs = find_images(&image_dir, &args.file_extension, args.shuffle);
        if imgs.is_empty() {
            warn!("image directory is empty");
        }
        let c = TransitionConfig::new(imgs, args.title);
        let html = Html(c.generate_html());

        let r = get_random_image(&image_dir, &args.file_extension);
        warn!("rando image is: {}", r);
        // Router::new().route_service("/foo", ServeFile::new("assets/index.html"))
        let app = Router::new()
            .nest_service("/img", ServeDir::new(&image_dir))
            .route_service("/random", ServeFile::new(r))
            .route_service("/static", ServeFile::new("img/prato.jpg"))
            .route("/", get(move || async { html }))
            .route("/hello", get(move || async { "what's up, dawg?" }));
        let bind_socket = format!("{}:{}", bind_address, serve_port);
        debug!("Starting webserver, binding to {}", bind_socket);
        let listener = tokio::net::TcpListener::bind(bind_socket).await.unwrap();
        axum::serve(listener, app.into_make_service())
            .await
            .unwrap();
    }
}

/// Walk filesystem directory and return a list of all files matching file extension.
/// Optionally can be shuffled for random order.
fn find_images(image_dir: &str, file_extension: &str, shuffle: bool) -> Vec<String> {
    let mut img_files: Vec<String> = Vec::new();
    for ent in WalkDir::new(image_dir).into_iter().flatten() {
        let path = ent.path();
        if path.display().to_string().ends_with(format!(".{}", file_extension).as_str()) {
            img_files.push(path.display().to_string());
        }
    }
    if shuffle {
        let mut rng = thread_rng();
        img_files.shuffle(&mut rng);
    }

    img_files
}

fn get_random_image(image_dir: &str, file_extension: &str) -> String {
    let imgs = find_images(image_dir, file_extension, true);
    imgs[0].to_string()
}
