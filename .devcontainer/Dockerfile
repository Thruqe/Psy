FROM mcr.microsoft.com/devcontainers/rust:latest

RUN curl -fsSL https://deb.nodesource.com/setup_lts.x | bash - && \
    apt-get install -y nodejs && \
    rm -rf /var/lib/apt/lists/*

RUN rustc --version && \
    node --version && \
    npm --version