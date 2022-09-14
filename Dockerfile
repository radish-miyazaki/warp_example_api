FROM rust:latest AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update && apt-get upgrade -y
RUN apt-get install -y curl build-essential
RUN apt install -y gcc-x86-64-linux-gnu

WORKDIR /app

COPY ./ .

ENV RUSTFLAGS='-C linker=x86_64-linux-gnu-gcc'
ENV CC='gcc'
ENV CC_x86_64_unknown_linux_musl=x86_64-linux-gnu-gcc
ENV CC_x86_64-unknown-linux-musl=x86_64-linux-gnu-gcc

RUN cargo build --target x86_64-unknown-linux-musl --release

FROM scratch

WORKDIR /app

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/question_and_answer ./
COPY --from=builder /app/.env ./

CMD ["/app/question_and_answer"]

# FROM rust:latest-slim-buster

# ENV CARGO_TARGET_DIR=/tmp/target \
#     DEBIAN_FRONTEND=noninteractive \
#     LC_CTYPE=ja_JP.utf8 \
#     LANG=ja_JP.utf8

# RUN apt-get update \
#   && apt-get install -y -q ca-certificates locales apt-transport-https libssl-dev pkg-config curl build-essential
#   && echo "ja_JP UTF-8" > /etc/locale.gen \
#   && locale-gen \
#   \
#   && echo "install rust tools" \
#   && rustup component add rustfmt \
#   && cargo install cargo-watch cargo-make

# WORKDIR /app

# CMD ["cargo", "run"]