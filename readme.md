# Merge exporter

Merge exporter merges multiple sources from http response or file content.

## Configuration

Service read configuration from system variables

| Name             | Default   | Description                                |
|------------------|-----------|--------------------------------------------|
| `MERGER_ADDRESS` | `0.0.0.0` | Export http server binding address         |
| `MERGER_PORT`    | `8989`    | Export http endpoint port                  | 
| `MERGER_URLS`    |           | Merge data source, separated by whitespace |
