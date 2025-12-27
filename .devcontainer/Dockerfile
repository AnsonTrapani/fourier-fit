FROM mcr.microsoft.com/devcontainers/rust:trixie

# Install apt dependencies
RUN set -eux; \
    apt update && apt dist-upgrade -y; \
    apt install -y build-essential clang;

# Create workspace
WORKDIR /fourier_fit_dev

# PATH items
ENV PATH=/home/${USERNAME}/.local/bin:~/.local/bin:${PATH}
