# rs-hasura-metrics

This project provides with a Prometheus metrics exporter for Hasura CE. Hasura is a wonderful tool but its Community Edition comes without much visibility about what's going on inside. This projects access directly to Hasura metadata database to expose metrics.

## Available metrics

- `count_events`: Counter of triggered events (only count table-based triggers), labels are `event` (string, contains trigger name) and `delivered` (boolean, `true` if event has been delivered, `false` otherwiser)
- `count_invocations`: Counter of event invocations (for table-based triggers), labels are `event` (string, contains trigger name) and `status` (integer, HTTP status returned by the call, -1 if a network error occured)

## Environment variables

| Variable | Semantic | Required | Default |
|-:|:-|:-:|:-:|
| `LISTEN` | Host to listen to | False | `127.0.0.1` |
| `PORT` | Port to listen to | False | `9185` |
| `POSTGRES_DB_USER` | Hasura Metadata DB user | False | `postgres` |
| `POSTGRES_DB_PASSWORD` | Hasura Metadata DB password | False | empty string |
| `POSTGRES_DB_HOST` | Hasura Metadata DB hostname | False | `localhost` |
| `POSTGRES_DB_PORT` | Hasura Metadata DB port | False | `5432` |
| `POSTGRES_DB_METADATA` | Hasura Metadata DB name | False | `postgres` |

# Setup

1. Clone this repository
2. `cargo build --release`
3. `./target/release/hasura-metrics`

