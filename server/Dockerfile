FROM rust:1-alpine
RUN apk add --no-cache build-base 

COPY . .

RUN cargo build --release

# COPY assets /app/assets

CMD ["cargo", "run", "--release"]