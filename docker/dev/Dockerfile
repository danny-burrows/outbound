FROM rust:1.97.1-slim-trixie

RUN apt-get update && apt-get install -y --no-install-recommends \
  cmake \
  g++ \
  libasound2-dev \
  libclang-dev \
  libgl-dev \
  libglu1-mesa-dev \
  libwayland-bin \
  libwayland-dev \
  libx11-dev \
  libxcursor-dev \
  libxi-dev \
  libxinerama-dev \
  libxkbcommon-dev \
  libxrandr-dev \
  make \
  pkg-config \
  wayland-protocols \
  && rm -rf /var/lib/apt/lists/*

RUN rustup component add clippy rustfmt

CMD ["bash", "-c", "cd /code && cargo build"]
