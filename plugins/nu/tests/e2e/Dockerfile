FROM debian:bookworm-slim

ENV DEBIAN_FRONTEND=noninteractive
ENV PATH="/root/.proto/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"

SHELL ["/bin/bash", "-lc"]

RUN apt-get update && apt-get install -y --no-install-recommends \
    bash \
    ca-certificates \
    curl \
    git \
    gzip \
    unzip \
    xz-utils \
 && rm -rf /var/lib/apt/lists/*

RUN bash <(curl -fsSL https://moonrepo.dev/install/proto.sh) 0.63.1 --yes

WORKDIR /workspace
COPY . .
