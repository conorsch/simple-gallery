simple-gallery
==============

A portable webserver for creating extremely simple photo galleries,
with **only vanilla JS.** It can serve directly, or just generate
the raw HTML so you can serve up a static site elsewhere.

Why?
----

Sharing photos is hard. I don't want to create accounts on third-party
websites riddled with ads. I'd like to self-host a few photos from an event,
but wrangling open source Javascript frameworks is hard. The experience I want is:
dump a bunch of images into a directory, run a binary, and have a website.
A simple script to generate static assets is enough for a decent-looking,
barebones photo gallery website. You can see an example at https://jawn.best/.

How it works
------------
All you need is a directory of images, preferably in PNG format. Then `simple-gallery`
will:

1. Scan the directory `--directory=img` for PNG files (specify e.g. `--file-extension jpg` to override)
2. Otherwise, spin up a webserver on `127.0.0.1:3000` (specify `--bind-address`, `--port` to override)

Installation
------------

First, make sure you have [Rust installed](https://rustup.rs/). Then:

```
cargo install --force simple-gallery
```

Requirements
------------

1. Some images to serve.
2. That's it.

Usage
-----

```
simple-gallery 0.1.0
Generates a single-page static web application, with no JS, serving a simple photogallery

USAGE:
    simple-gallery [OPTIONS]

OPTIONS:
    -b, --bind-address <BIND_ADDRESS>
            Local IP address to bind to [default: 127.0.0.1]

    -d, --directory <DIRECTORY>
            On-disk path for directory of images to serve [default: img]

    -g, --generate
            Build static HTML and print to stdout, then exit

    -h, --help
            Print help information

    -p, --port <PORT>
            TCP port to listen on [default: 3000]

    -s, --shuffle
            Randomize order of images

    -t, --title <TITLE>
            Title for HTML page, e.g. "example.com" [default: simple-gallery]

    -V, --version
            Print version information
```

Creating a static site
----------------------

With a directory structure like:

```
cool-pictures/
├── tree.png
├── dog.png
├── horse.png
```

Run:

```
simple-gallery --generate --directory ./cool-pictures > index.html
```

You can now serve that directory, e.g.

```
python3 -m http.server --port 3000 --directory .
```

More advanced features (FAQ)
----------------------------
Many features are intentionally left out. There's no navigation functionality:
the images will transition indefinitely. If you want to retrieve a specific
image, peek at the source, and GET the `<img src="">` URL.

References
----------
The logic for computing the CSS animation duration values were taken
from this [invaluable blog post](https://www.devtwins.com/blog/css-cross-fading-images).
As ever, [MDN CSS docs](https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_Animations/Using_CSS_animations) were helpful.
The vanilla JS examples for transitions were adapted from [this helpful post](https://daily-dev-tips.com/posts/fading-images-using-javascript/).
Finally, while it's not used, the crate [arse](https://crates.io/crates/arse) may be helpful.

License
----
AGPLv3
