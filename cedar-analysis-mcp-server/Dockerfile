FROM amazonlinux:2023 

RUN yum update -y \
  && yum install -y \
  curl-minimal clang tar zip unzip python3 git xz \
  make wget gcc gcc-c++ \
  && yum clean all

# Install Node.js and npm
RUN curl -fsSL https://rpm.nodesource.com/setup_18.x | bash - \
  && yum install -y nodejs \
  && node -v && npm -v

# Setup Rust toolchain
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > /tmp/rustup.sh \
  && chmod +x /tmp/rustup.sh \
  && /tmp/rustup.sh -y \
  && . ~/.profile; rustup component add llvm-tools-preview rust-src

# Install cargo-fuzz
RUN . ~/.profile; cargo install cargo-fuzz

# Install Lean
RUN wget https://raw.githubusercontent.com/leanprover/elan/master/elan-init.sh && sh elan-init.sh -y --default-toolchain none

# Install protoc
RUN wget https://github.com/protocolbuffers/protobuf/releases/download/v29.0/protoc-29.0-linux-x86_64.zip && unzip protoc-29.0-linux-x86_64.zip && rm protoc-29.0-linux-x86_64.zip

ENV PATH="/root/.cargo/bin:${PATH}"

ENV CEDAR_SPEC_ROOT=/opt/src/cedar-deps

# Clone `cedar` repositories
WORKDIR $CEDAR_SPEC_ROOT
RUN git clone --depth 1 https://github.com/cedar-policy/cedar-spec

# Clone `cedar` repositories
WORKDIR $CEDAR_SPEC_ROOT/cedar-spec/
RUN git clone --depth 1 https://github.com/cedar-policy/cedar

# Build CLI and Lean library
WORKDIR $CEDAR_SPEC_ROOT/cedar-spec/
RUN source /root/.profile \
  && cd cedar-lean-ffi \
  && source ./set_env_vars.sh \
  && ./build_lean_lib.sh \
  && cd ../cedar-lean-cli \
  && cargo build \
  && cargo install --path .

# Create a setup script that will be sourced in .bashrc
RUN echo '#!/bin/bash' > /root/setup_env.sh \
  && echo 'source /root/.profile' >> /root/setup_env.sh \
  && echo "cd $CEDAR_SPEC_ROOT/cedar-spec/cedar-lean-ffi" >> /root/setup_env.sh \
  && echo 'source ./set_env_vars.sh' >> /root/setup_env.sh \
  && chmod +x /root/setup_env.sh

# Add the setup script to .bashrc so it runs automatically
RUN echo 'source /root/setup_env.sh' >> /root/.bashrc

# Download cvc5 based on architecture
RUN arch=$(uname -m) && \
    if [ "$arch" = "aarch64" ] || [ "$arch" = "arm64" ]; then \
        curl -LO https://github.com/cvc5/cvc5/releases/download/cvc5-1.2.1/cvc5-Linux-arm64-static.zip && \
        unzip cvc5-Linux-arm64-static.zip && \
        export CVC5_DIR="cvc5-Linux-arm64-static"; \
    else \
        curl -LO https://github.com/cvc5/cvc5/releases/download/cvc5-1.2.1/cvc5-Linux-x86_64-static.zip && \
        unzip cvc5-Linux-x86_64-static.zip && \
        export CVC5_DIR="cvc5-Linux-x86_64-static"; \
    fi && \
    echo "export CVC5=$CEDAR_SPEC_ROOT/cedar-spec/$CVC5_DIR/bin/cvc5" >> /root/setup_env.sh

ENV MCP_ROOT=/opt/src/mcp
COPY mcp/. $MCP_ROOT

WORKDIR $MCP_ROOT
RUN npm install && npm run build

# Set the working directory to mcp
WORKDIR $MCP_ROOT

# Use a simple entrypoint that keeps the container running
ENV NODE_ENV=production
ENTRYPOINT ["/bin/bash", "-c", "source /root/setup_env.sh && exec node /opt/src/mcp/dist/server.js"]
