FROM debian:stretch as builder

RUN apt-get update && \
    apt-get install --no-install-recommends -y \
    bash \
    ca-certificates \
    openssh-client \
    rsync \
    curl file \
    build-essential \
    libssl-dev \
    pkg-config \
    libpq-dev \
    postgresql-client \
    git \
    pkg-config cmake zlib1g-dev \
    autoconf automake autotools-dev libtool xutils-dev && \
    rm -rf /var/lib/apt/lists/*

RUN update-ca-certificates

RUN mkdir -p /rust/cargo /rust/rustup

ENV RUSTUP_HOME=/rust/rustup \
    CARGO_HOME=/rust/cargo


ARG RUST_TOOLCHAIN=1.54.0

RUN curl https://sh.rustup.rs -sSf | sh -s -- \
  --default-toolchain $RUST_TOOLCHAIN --no-modify-path -y

ENV CARGO_TARGET_DIR=/out \
    PATH=$CARGO_HOME/bin:$PATH

ADD . /code
WORKDIR /code

RUN cargo build

# Stage 2: Add to a small deploy container
FROM debian:stretch

RUN apt-get update \
    && apt-get install --no-install-recommends -y \
    libpq5 \
    libssl1.1 \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /out/debug/actix-tracing-sentry-repro /

ENTRYPOINT ["/actix-tracing-sentry-repro"]

EXPOSE 7878
ENV RUST_BACKTRACE=1
