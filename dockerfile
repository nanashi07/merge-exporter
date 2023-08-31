FROM rust:1.72-alpine3.17 as BUILDER

ENV RUST_BACKTRACE=full

WORKDIR /workspace
COPY ./src /workspace/src
COPY ./Cargo.* /workspace/

# install build tools
RUN apk add alpine-sdk
RUN cargo build --release


# Target image
FROM alpine:3.17

WORKDIR /app
COPY --from=BUILDER /workspace/target/release/merge-exporter /app/
CMD ["./merge-exporter"]