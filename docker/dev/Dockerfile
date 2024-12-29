FROM rust:1.83.0-slim-bullseye

RUN apt-get update && apt-get install -y \
  cmake \
  libasound2-dev \
  libgl1-mesa-dev \
  libglu1-mesa-dev \
  libx11-dev \
  libxcursor-dev \
  libxi-dev \
  libxinerama-dev \
  libxkbcommon-dev \
  libxrandr-dev

RUN rustup component add clippy

CMD ["bash", "-c", "cd /code && cargo build"]
