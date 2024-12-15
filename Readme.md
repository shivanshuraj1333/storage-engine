# Rust Storage Engine

A high-performance asynchronous message processing and storage engine built with Rust, featuring gRPC communication and
modular architecture.

## Architecture Overview

```mermaid
graph TD
    subgraph Client Layer
        Client[gRPC Client]
    end

    subgraph Server Layer
        LS[Listener Server]
    end

    subgraph Processing Layer
        subgraph Core["Engine Core"]
            RQ[Raw Queue]
            PQ[Processed Queue]
        end
        ME[MetaData Extractor]
    end

    subgraph Storage Layer
        SW[Storage Writer]
        SA[Storage Adaptor]
        S3[AWS S3]
        GCS[Google Cloud Storage]
    end

    subgraph Infrastructure
        CL[Config Loader]
        EL[Event Logger]
    end

    %% Main Data Flow
    Client -->|gRPC Messages| LS
    LS -->|Queue Message| RQ
    RQ -->|Extract Metadata| ME
    ME -->|Processed Message| PQ
    PQ -->|Batch Messages| SW
    SW -->|Write| SA
    SA -->|Store| S3
    SA -->|Store| GCS

    %% Infrastructure Connections
    CL -.->|Configure|LS
    CL -.->|Configure|Core
    CL -.->|Configure|SA
    EL -.->|Log|LS
    EL -.->|Log|Core
    EL -.->|Log|ME
    EL -.->|Log|SW

    %% Styling
    classDef implemented fill:#90EE90,stroke:#333,stroke-width:2px
    classDef inProgress fill:#FFB6C1,stroke:#333,stroke-width:2px
    classDef planned fill:#87CEEB,stroke:#333,stroke-width:2px
    classDef queue fill:#FFF,stroke:#333,stroke-width:2px
    classDef infrastructure fill:#F0E68C,stroke:#333,stroke-width:2px

    %% Apply styles
    class Client,LS implemented
    class RQ,PQ queue
    class ME,SW inProgress
    class S3,GCS planned
    class CL,EL infrastructure
    class Core implemented
```

## Components

### Core Components:

- **Engine Core**: Central message processing unit
- **Config Loader**: Configuration management
- **Listener Server**: gRPC interface
- **MetaData Extractor**: Message analysis and metadata extraction
- **Storage Adaptor**: Storage interface layer
- **Event Log**: System-wide logging

### Component Details:

- **Listener Server**:
    - Listens for proto messages over gRPC
    - Uses msg.proto for message definitions
    - Queues messages in Engine Core's raw data queue (RDQ)

- **Engine Core**:
    - Central unit for message processing
    - Manages raw data queue (RDQ)
    - Coordinates parallel metadata extraction
    - Handles processed message queue
    - Manages FMM entity preparation and batch compression

- **Storage Adaptor**:
    - Plugin interface for different storage backends
    - Supports GCP cloud storage and S3
    - Config-driven storage selection

- **Event Log**:
    - Unified logging interface
    - Comprehensive message processing tracking
    - Integration with upstream logging systems

## Technical Stack

- **Async Runtime**: Tokio
- **gRPC Framework**: Tonic
- **Serialization**: Prost (Protocol Buffers)
- **Logging**: Tracing
- **Error Handling**: thiserror
- **Configuration**: Serde

## Development

### Prerequisites

- Rust 1.70+
- Protocol Buffers compiler
- Make (for build scripts)

### Build Commands

```bash
make build       # Build the project
make test        # Run tests
make lint        # Run clippy and format checks
make run         # Run the server
make run-client  # Run the test client
make doc         # Generate documentation
```

### Feature Flags

- `client`: Enables client code compilation
- Default features include server-side functionality

## Project Structure

```
.
├── src/
│   ├── config.rs     # Configuration structures
│   ├── core.rs       # Engine Core implementation
│   ├── error.rs      # Error types
│   ├── proto/        # Protocol buffer implementations
│   ├── server.rs     # gRPC server implementation
│   └── storage/      # Storage adaptor interfaces
├── proto/
│   └── msg.proto     # Protocol buffer definitions
└── examples/
    └── grpc_client.rs # Test client implementation
```

## Roadmap

### Q2 2024

- [ ] Complete MetaData Extractor implementation
- [ ] Implement basic Storage Writer functionality
- [ ] Add message compression

### Q3 2024

- [ ] Implement AWS S3 storage adaptor
- [ ] Add Google Cloud Storage support
- [ ] Optimize batch processing

### Q4 2024

- [ ] Add monitoring and metrics
- [ ] Implement advanced error recovery
- [ ] Add support for custom storage backends

## Contributing

Contributions are welcome! Please check our contributing guidelines for more information.

## License

[MIT License]