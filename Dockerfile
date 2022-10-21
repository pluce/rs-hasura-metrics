
FROM rust:1.64.0 as build-env
WORKDIR /app
COPY . /app
# Env variable used to get through Out of memory error
ENV CARGO_NET_GIT_FETCH_WITH_CLI true
RUN cargo build --release

FROM gcr.io/distroless/cc
COPY --from=build-env /app/target/release/hasura-metrics /
CMD ["./hasura-metrics"]