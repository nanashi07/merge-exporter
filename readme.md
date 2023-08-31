# Merge exporter

Merge exporter merges multiple sources from http response or file content.

## Configuration

Service read configuration from system variables

| Name               | Default / Example  | Description                                                |
|--------------------|--------------------|------------------------------------------------------------|
| `MERGER_ADDRESS`   | `0.0.0.0`          | Export http server binding address                         |
| `MERGER_PORT`      | `8989`             | Export http endpoint port                                  | 
| `MERGER_URLS`      | `http://localhost:8080 file:///root/metrics.txt,http://localhost:9090` | Merge data source. Source group is separated by whitespace, each source group is executed parallely. Source group can contains multiple source, service read them sequently and return the first success one. |
| `MERGER_LOG_LEVEL` | `INFO`             | Log level                                                  |

## Build

```rust
cargo build --release
```

## Docker image

```
docker build -t merge-exporter:$VERSION .
```