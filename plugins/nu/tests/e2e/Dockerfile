FROM rust:1.94-bookworm

ENV DEBIAN_FRONTEND=noninteractive
ENV PATH="/root/.proto/bin:/usr/local/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"

SHELL ["/bin/bash", "-lc"]

RUN apt-get update && apt-get install -y --no-install-recommends \
    bash \
    bubblewrap \
    build-essential \
    ca-certificates \
    curl \
    git \
    unzip \
    xz-utils \
 && rm -rf /var/lib/apt/lists/*

RUN /usr/local/cargo/bin/rustup target add wasm32-wasip1
RUN bash <(curl -fsSL https://moonrepo.dev/install/proto.sh) 0.55.3 --yes

WORKDIR /workspace
COPY . .
