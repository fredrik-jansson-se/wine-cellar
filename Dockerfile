####################################################################################################
## Build
####################################################################################################
ARG APP=wine-cellar

FROM lukemathwalker/cargo-chef:latest-rust-1-alpine AS chef
WORKDIR /app-dir

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ARG APP

RUN apk update && \
    apk upgrade --no-cache && \
    apk add --no-cache openssl libressl-dev

COPY --from=planner /app-dir/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin $APP


####################################################################################################
## This stage is used to get the correct files into the final image
####################################################################################################
FROM alpine@sha256:4b7ce07002c69e8f3d704a9c5d6fd3053be500b7f1c69fc0d80990c2ad8dd412 AS files

# mailcap is used for content type (MIME type) detection
# tzdata is used for timezones info
RUN apk update && \
    apk upgrade --no-cache && \
    apk add --no-cache ca-certificates tzdata

RUN update-ca-certificates

ENV USER=the-user
ENV UID=1000
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"


####################################################################################################
## Final image
####################################################################################################
FROM scratch
ARG APP

COPY --from=files \
    /etc/passwd \
    /etc/group \
    /etc/nsswitch.conf \
    /etc/

COPY --from=files /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=files /usr/share/zoneinfo /usr/share/zoneinfo

# Copy our build
COPY --from=builder /app-dir/target/release/$APP /bin/$APP

# Use an unprivileged user.
# USER the-user:the-user

# the scratch image doesn't have a /tmp folder, you may need it
WORKDIR /tmp

WORKDIR /app-dir
EXPOSE 20000

ENV RUST_LOG=info
ENV DATABASE_URL=sqlite:/app-dir/data/data.db
ENTRYPOINT ["/bin/wine-cellar"]
