FROM rust:1.89-bookworm
RUN echo 'deb http://deb.debian.org/debian bookworm-backports main' >> /etc/apt/sources.list
RUN apt-get update && apt-get install -y git python3-matplotlib python3-pandas golang-1.21-go openjdk-17-jdk vim && \
    rm -rf /var/lib/apt/lists/*
ENV PATH="${PATH}:/usr/lib/go-1.21/bin"

## Get openfga
WORKDIR /
RUN git clone https://github.com/openfga/openfga -b main
# the openfga commit hash used in our experiments
RUN cd openfga && git checkout bbb4a07
COPY openfga-timing.patch openfga/
RUN cd openfga && git apply openfga-timing.patch
ENV GOPROXY="direct"
RUN cd openfga && go build -o ./openfga ./cmd/openfga
ENV PATH="${PATH}:${HOME}/openfga"
## run an openfga server for 100ms and then kill it. For some reason this reduces flakiness later on
RUN timeout 0.1 openfga run --playground-enabled=false --log-level=warn || true

## Clone cedar and cedar-spec
RUN git clone https://github.com/cedar-policy/cedar-spec cedar-benchmarks/cedar-spec -b main
RUN git clone https://github.com/cedar-policy/cedar cedar-benchmarks/cedar-spec/cedar -b main

## Install Cedar CLI so it is usable from command line
RUN cargo install cedar-policy-cli --path cedar-benchmarks/cedar-spec/cedar/cedar-policy-cli

## Get cedar-benchmarks (benchmark harness and example app code)
# unfortunately, `COPY src cedar-benchmarks/` doesn't do what we want, it copies
# the contents of `src` directly into `cedar-benchmarks` without creating a
# `src` directory.
# we could start with `COPY . cedar-benchmarks/`, but that pulls in some things
# we don't want in the image, including whatever target/ directory is currently
# on the host, which could be quite large. Seemed better to me to explicitly list
# the things we want copied in. And COPY followed by `RUN rm` doesn't help with
# image size due to how Docker works (unless we go multi-stage, which I don't
# think there's justification for in this case).
WORKDIR /
COPY Cargo.toml plot.py README.md cedar-benchmarks/
COPY generators cedar-benchmarks/generators/
COPY benches cedar-benchmarks/benches/
COPY src cedar-benchmarks/src/
COPY rego-harness cedar-benchmarks/rego-harness/
COPY validate_all_benchmarks.sh cedar-benchmarks

## Check that the Cedar policies validate with the provided schemas
RUN BENCHES=cedar-benchmarks/benches cedar-benchmarks/validate_all_benchmarks.sh

## Build Rego harness
RUN cd cedar-benchmarks/rego-harness && \
    go mod init rego-harness && \
    go get github.com/open-policy-agent/opa/rego && \
    mkdir -p build/bin && \
    go build -o build/bin/main cmd/harness/main.go

## Build benchmark harness so it's ready
WORKDIR /cedar-benchmarks
RUN cargo clean && cargo build --release

