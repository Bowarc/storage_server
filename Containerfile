##########
#  BASE  #
##########
FROM rust:1.85 AS base

RUN rustup target add wasm32-unknown-unknown
RUN cargo install --locked wasm-bindgen-cli --version 0.2.100
# RUN cargo install sccache
RUN cargo install --git https://github.com/bowarc/cargo-chef
RUN cargo install --git https://github.com/wilsonzlin/minify-html minhtml --rev 2301223773dadce30a33b4c407d8b2524adeb5e2

##########
# PANNER #
##########
FROM base AS planner

WORKDIR /app

# Move the essentials
COPY ./Rocket.toml ./Cargo.toml ./Cargo.lock .
COPY ./back ./back
COPY ./front ./front

# Prepare all dependencies
RUN cargo chef prepare --recipe-path recipe.json

###########
# BUILDER #
###########
FROM base AS builder

WORKDIR /app

# Take the recipe only from tyhe planner
COPY --from=planner /app/recipe.json recipe.json

# Set up the project's build artefacts
RUN cargo chef cook --release --recipe-path recipe.json
RUN cargo chef cook -p front --release --target=wasm32-unknown-unknown --recipe-path recipe.json

# Pull the projects code
COPY ./scripts/build.sh ./scripts/build_back.sh ./scripts/build_front.sh ./scripts/
COPY ./Rocket.toml ./Cargo.toml ./Cargo.lock .
COPY ./back ./back
COPY ./front ./front

# Build it
RUN sh ./scripts/build.sh release

############
# MINIFIER #
############
FROM base AS minifier

WORKDIR /app

COPY ./static ./static

COPY --from=builder /app/target/wasm-bindgen/release/* ./static/

RUN minhtml --minify-css ./static/css/*.css
# RUN minhtml --minify-js ./static/lib/**/*.js # Broken
# RUN minhtml --minify-js ./static/front.js --output ./static/front.js # Broken
RUN minhtml --minify-doctype ./static/index.html --output ./static/index.html


##########
# RUNNER #
##########
FROM ubuntu:22.04 AS runner

WORKDIR /app

# Here we take the rocket config from builder because it has been used to build the front end, to elimiate all TOCTOU / desync issues, we use the same one
COPY --from=builder /app/target/release/server /app/Rocket.toml .
COPY --from=minifier /app/static ./static

RUN mkdir ./log
RUN mkdir ./cache

EXPOSE 42070

CMD ["./server"]
