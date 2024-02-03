FROM rust:1.75-alpine as build
RUN apk add musl-dev protobuf-dev
WORKDIR /app
COPY . .
RUN cargo build --release

FROM alpine:3.19 as app
COPY --from=build /app/target/release/RusticPoker /
EXPOSE 55100
CMD ["/RusticPoker"]
