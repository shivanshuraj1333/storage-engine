# OpenTelemetry Storage Engine

A high-performance asynchronous trace storage engine built with Rust, featuring OpenTelemetry protocol support, S3-compatible storage, and health monitoring.

## Features

- **OpenTelemetry Support**
  - OTLP protocol implementation
  - Trace collection and storage
  - Span batching and processing

- **Storage**
  - S3-compatible backend
  - Configurable batching
  - Efficient data organization

- **Monitoring**
  - Health check endpoints
  - Performance metrics
  - Error tracking

- **API**
  - gRPC for trace collection
  - REST for querying spans
  - Health status endpoints

## Quick Start

1. **Prerequisites**
```bash
# Required tools
- Rust 1.70+
- Docker
- AWS CLI
```

2. **Setup LocalStack**
```bash
# Start LocalStack
docker run --rm -it -p 4566:4566 localstack/localstack

# Create test bucket
aws --endpoint-url=http://localhost:4566 s3 mb s3://my-test-bucket
```

3. **Run the Server**
```bash
# Build and run with logging
RUST_LOG=info cargo run
```

4. **Test with Example Client**
```bash
# Run the test client
RUST_LOG=info cargo run --example grpc_client --features client
```

## Architecture

```mermaid
graph TD
    %% Client Layer
    subgraph Clients
        OTLP[OpenTelemetry Client]
        HTTP[HTTP Client]
    end

    %% Server Layer
    subgraph Server["Server Layer (src/server.rs)"]
        GS[gRPC Server]
        HS[HTTP Server]
        LS[ListenerServer]
        Router[Axum Router]
    end

    %% Processing Layer
    subgraph Core["Processing Layer (src/core.rs)"]
        EC[EngineCore]
        Queue[Message Queue]
        Batch[Batch Processor]
        Conv[Span Converter]
    end

    %% Health Monitoring
    subgraph Health["Health Monitoring (src/health.rs)"]
        HM[Health Monitor]
        Metrics[Health Metrics]
        Status[Health Status]
    end

    %% Storage Layer
    subgraph Storage["Storage Layer (src/storage/mod.rs)"]
        SW[StorageWriter Trait]
        S3W[S3StorageWriter]
        Reader[SpanReader]
    end

    %% Config Layer
    subgraph Config["Configuration (src/config.rs)"]
        Env[Environment]
        YAML[YAML Config]
        Defaults[Default Values]
    end

    %% Error Handling
    subgraph Errors["Error Handling (src/error.rs)"]
        PE[ProcessingError]
        SE[StorageError]
        CE[ConfigError]
    end

    %% Data Flow
    OTLP -->|OTLP Protocol| GS
    HTTP -->|REST| HS
    GS --> LS
    HS --> Router
    Router --> Reader
    LS -->|Channel| Queue
    Queue --> EC
    EC --> Batch
    Batch --> Conv
    Conv --> SW
    SW --> S3W
    S3W -->|Write| S3[(S3 Storage)]
    Reader -->|Read| S3W

    %% Monitoring Flow
    EC -.->|Report| HM
    S3W -.->|Report| HM
    HM -->|Update| Metrics
    Metrics -->|Expose| Status
    Router -->|Query| Status

    %% Configuration Flow
    Env -->|Load| Config
    YAML -->|Parse| Config
    Defaults -->|Fallback| Config
    Config -->|Configure| EC
    Config -->|Configure| S3W

    %% Error Flow
    EC -.->|Emit| PE
    S3W -.->|Emit| SE
    Config -.->|Emit| CE

    %% Styling
    classDef primary fill:#f9f,stroke:#333,stroke-width:2px
    classDef secondary fill:#bbf,stroke:#333,stroke-width:2px
    classDef storage fill:#bfb,stroke:#333,stroke-width:2px
    classDef monitoring fill:#fbb,stroke:#333,stroke-width:2px
    classDef config fill:#ffb,stroke:#333,stroke-width:2px
    classDef error fill:#fdd,stroke:#333,stroke-width:2px

    class EC,GS,LS,S3W primary
    class Queue,Batch,Conv,Router secondary
    class SW,Reader,S3 storage
    class HM,Metrics,Status monitoring
    class Config,Env,YAML,Defaults config
    class PE,SE,CE error
```

### Component Details

1. **Server Layer**
   - `ListenerServer`: Handles gRPC trace collection
   - `Router`: Manages HTTP endpoints for querying
   - Supports both OTLP and REST protocols

2. **Processing Layer**
   - `EngineCore`: Central processing unit
   - Message queuing and batching
   - Span conversion and validation

3. **Health Monitoring**
   - Real-time health metrics
   - Queue size monitoring
   - Error rate tracking
   - Performance statistics

4. **Storage Layer**
   - `StorageWriter` trait for storage abstraction
   - S3-compatible implementation
   - Efficient span organization
   - Query capabilities

5. **Configuration**
   - Environment variables
   - YAML configuration
   - Sensible defaults
   - Runtime validation

6. **Error Handling**
   - Structured error types
   - Error propagation
   - Graceful failure handling

## API Reference

### gRPC Endpoints
- `/opentelemetry.proto.collector.trace.v1.TraceService/Export`
  - Accepts OTLP trace data
  - Batches and stores spans

### HTTP Endpoints
- `GET /spans`
  - Query recent spans
  - Optional limit parameter
- `GET /health`
  - System health status
  - Performance metrics

## Configuration

Configuration can be provided via:
1. Environment variables
2. YAML configuration file
3. Default values

### Environment Variables
```bash
SERVER_HOST=0.0.0.0
SERVER_PORT=50051
STORAGE_BUCKET=my-test-bucket
RUST_LOG=info
```

### YAML Configuration
```yaml
server:
  host: "0.0.0.0"
  port: 50051
storage:
  bucket: "my-test-bucket"
  prefix: "traces"
processing:
  batch_size: 100
  batch_timeout_ms: 5000
```

## Development

### Build Commands
```bash
make setup-proto  # Setup OpenTelemetry protos
make build       # Build the project
make test        # Run tests
make lint        # Run lints
make run         # Run server
make run-client  # Run test client
```

### Project Structure
```
.
├── proto/              # Protocol definitions
├── src/
│   ├── config/        # Configuration
│   ├── core/          # Processing engine
│   ├── health/        # Health monitoring
│   ├── proto/         # Generated code
│   ├── server/        # gRPC server
│   └── storage/       # Storage backend
└── examples/          # Usage examples
```

### Testing
```bash
# Unit tests
cargo test

# Integration tests
cargo test --test '*'

# With logging
RUST_LOG=debug cargo test
```

## Monitoring

### Health Metrics
- Queue size
- Processing latency
- Error rates
- Storage operations

### Logging
```bash
# Debug logging
RUST_LOG=debug cargo run

# Trace logging
RUST_LOG=trace cargo run
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and lints
5. Submit a pull request

## License

[MIT License](LICENSE)