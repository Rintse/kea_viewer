FROM rust:latest as builder
WORKDIR /app
ADD . /app
RUN cargo build --release

from gcr.io/distroless/cc
COPY --from=builder /app/target/release/kea_leases_viewer /
CMD ["./kea_leases_viewer"]

