# See here for image contents: https://github.com/microsoft/vscode-dev-containers/tree/v0.238.0/containers/rust/.devcontainer/base.Dockerfile

# [Choice] Debian OS version (use bullseye on local arm64/Apple Silicon): buster, bullseye
ARG VARIANT="buster"
FROM mcr.microsoft.com/vscode/devcontainers/rust:0-${VARIANT}

# [Optional] Uncomment this section to install additional packages.
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends fish

USER vscode

# RUN rustup default nightly \
#     && cargo install cargo-expand \
#     && rustup component add rustfmt \
#     && rustup component add clippy

RUN cargo install cargo-expand \
    && rustup component add rustfmt \
    && rustup component add clippy
