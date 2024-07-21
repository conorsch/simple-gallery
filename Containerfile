FROM rust AS builder
LABEL maintainer=root@ruin.dev

# use musl target for static binaries,
# so we can use a scratch container.
RUN rustup target add x86_64-unknown-linux-musl

# configure container layer caching,
# by prebuilding dependencies in a shell project.
RUN USER=root cargo new --bin simple-gallery
WORKDIR /simple-gallery

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY .cargo/config.toml .cargo/config.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src
COPY ./files ./files

# build for release
RUN rm -f ./target/release/deps/simple_gallery*
RUN cargo build --release

# technically we could use a "scratch" image here, so only the binary is present.
# it's helpful to have a minimal OS, however, to debug e.g. volume mounts.
# FROM scratch
FROM debian:stable
COPY --from=builder /simple-gallery/target/x86_64-unknown-linux-musl/release/simple-gallery /usr/bin/simple-gallery
ENTRYPOINT ["/usr/bin/simple-gallery"]
