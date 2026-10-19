# ==============================================================================
# Build Stage
# ==============================================================================
FROM golang:1.24-alpine AS builder

WORKDIR /app

# Install ca-certificates and build tools
RUN apk add --no-cache ca-certificates git

# Cache dependencies
COPY go.mod ./
RUN go mod download

# Copy source code and build statically linked binary
COPY . .
RUN CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build \
    -trimpath \
    -ldflags="-w -s" \
    -o /app/bin/childprotect-server \
    ./cmd/server/main.go

# ==============================================================================
# Final Minimal Distroless Stage
# ==============================================================================
FROM gcr.io/distroless/static-debian12:nonroot

WORKDIR /

# Copy trusted CA roots and compiled binary
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /app/bin/childprotect-server /childprotect-server

USER nonroot:nonroot

EXPOSE 8080

ENTRYPOINT ["/childprotect-server"]
