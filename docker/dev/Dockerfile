FROM rust:1.83.0-slim-bullseye

RUN apt-get update && apt-get install -y \
  cmake \
  g++ \
  libopengl-dev \
  libx11-dev \
  libxinerama-dev \
  libxcursor-dev \
  libxi-dev \
  libglew-dev \
  libclang-dev \
  libxrandr-dev

CMD ["bash", "-c", "cd /code && cargo build"]
