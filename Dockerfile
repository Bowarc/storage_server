###########
## BUILD ##
###########
FROM rust:1.85 AS builder

# Install build dependencies
RUN rustup target add wasm32-unknown-unknown
RUN cargo install --locked wasm-bindgen-cli

# Setup
WORKDIR /app

# Get build scripts
RUN mkdir ./scripts
COPY ./scripts/build.sh ./scripts/build_back.sh ./scripts/build_front.sh ./scripts

# Copy project
COPY Cargo.toml Cargo.lock Rocket.toml .

RUN mkdir ./back ./back/src && echo "fn main() {}" > ./back/src/main.rs
COPY ./back/Cargo.toml ./back

RUN mkdir ./front ./front/src && echo "fn main() {}" > ./front/src/main.rs
COPY ./front/Cargo.toml ./front
COPY ./front/build.rs ./front

# Compile dependencies
RUN cargo b -p back --release
RUN cargo b -p front --target wasm32-unknown-unknown

# Copy the project's code (this is to make sure a layer with compiled dependencies is created)
RUN rm -rf ./back/src ./front/src
COPY ./back/src ./back/src
COPY ./front/src ./front/src

# Build the whole project
RUN sh ./scripts/build.sh release

#########
## RUN ##
#########
FROM ubuntu:22.04 AS runner
# FROM scratch Causes issues with musl libc ? something like that
# check this for more info https://dev.to/mattdark/rust-docker-image-optimization-with-multi-stage-builds-4b6c
 
WORKDIR /app

COPY --from=builder /app/target/release/storage_server .
COPY --from=builder /app/Rocket.toml .
COPY ./static ./static
COPY --from=builder /app/target/wasm-bindgen/release/* ./static/

RUN mkdir log cache

EXPOSE 42069

CMD ["./storage_server"]
