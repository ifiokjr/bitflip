# Build from the repository root with:
# docker build -f bitflip_server/bitflip_server_server/Dockerfile .

# Build stage
FROM dart:3.13.3@sha256:7e57e61d97813dc57dd0801656ff4d5c5efafa47bae563a681571543ad30f199 AS build

# Create a minimal workspace that preserves locked server dependency versions.
WORKDIR /app/bitflip_server
COPY bitflip_server/pubspec.yaml .
COPY bitflip_server/pubspec.lock .
COPY bitflip_server/bitflip_server_client bitflip_server_client
COPY bitflip_server/bitflip_server_server bitflip_server_server
COPY bitflip_program/clients/dart /app/bitflip_program/clients/dart
RUN dart pub get --enforce-lockfile

# Compile the server executable.
WORKDIR /app/bitflip_server/bitflip_server_server
RUN dart build cli --target bin/main.dart --output build
RUN mv build/bundle/bin/main build/bundle/bin/server

# Add a fallback for the copy of possibly missing directories.
RUN mkdir -p config web migrations

# Final stage
FROM alpine:3.24.1@sha256:28bd5fe8b56d1bd048e5babf5b10710ebe0bae67db86916198a6eec434943f8b
WORKDIR /app
RUN addgroup -S -g 10001 bitflip \
  && adduser -S -D -H -u 10001 -G bitflip bitflip

# Environment variables
ENV runmode=production
ENV serverid=default
ENV logging=normal
ENV role=monolith

# Copy runtime dependencies
COPY --from=build /runtime/ /

# Copy compiled server executable and native libraries
COPY --from=build /app/bitflip_server/bitflip_server_server/build/bundle/ ./

# Copy configuration files and resources
COPY --from=build /app/bitflip_server/bitflip_server_server/config/ config/
COPY --from=build /app/bitflip_server/bitflip_server_server/web/ web/
COPY --from=build /app/bitflip_server/bitflip_server_server/migrations/ migrations/

# This file is required to enable the endpoint log filter in Insights.
COPY --from=build /app/bitflip_server/bitflip_server_server/lib/src/generated/protocol.yaml lib/src/generated/protocol.yaml

USER bitflip

# Expose ports
EXPOSE 8080
EXPOSE 8082

HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 \
  CMD wget --quiet --output-document=- http://127.0.0.1:8080/livez >/dev/null || exit 1

# Define the entrypoint command
ENTRYPOINT ["sh", "-c", "exec ./bin/server --mode=\"$runmode\" --server-id=\"$serverid\" --logging=\"$logging\" --role=\"$role\""]
