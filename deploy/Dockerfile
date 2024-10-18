FROM debian:sid
ENV RUST_LOG=info
WORKDIR /work
COPY subparsvc/target/release/subparweb /work/subparweb

RUN apt-get update && \
    apt-get upgrade -y && \
    apt-get install \
        libssl-dev \
        pkg-config \
        ca-certificates \
        -y

EXPOSE 3000
RUN chmod +x /work/subparweb
ENTRYPOINT [ "/work/subparweb" ]
