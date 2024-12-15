/*
    Protocol buffer module organization
    Provides type definitions and service interfaces
    Separates client and server concerns
*/

// Include the generated proto code
pub mod opentelemetry {
    pub mod proto {
        pub mod collector {
            pub mod trace {
                pub mod v1 {
                    include!("opentelemetry.proto.collector.trace.v1.rs");
                }
            }
        }
        pub mod common {
            pub mod v1 {
                include!("opentelemetry.proto.common.v1.rs");
            }
        }
        pub mod trace {
            pub mod v1 {
                include!("opentelemetry.proto.trace.v1.rs");
            }
        }
        pub mod resource {
            pub mod v1 {
                include!("opentelemetry.proto.resource.v1.rs");
            }
        }
    }
}

// Re-export commonly used types
pub use opentelemetry::proto::collector::trace::v1::{
    ExportTraceServiceRequest,
    ExportTraceServiceResponse,
};

// Re-export server types
pub use opentelemetry::proto::collector::trace::v1::trace_service_server::{
    TraceService,
    TraceServiceServer,
};

// Re-export client types when client feature is enabled
#[cfg(feature = "client")]
pub use opentelemetry::proto::collector::trace::v1::trace_service_client::TraceServiceClient;

// Re-export trace types
pub use opentelemetry::proto::trace::v1::{
    ResourceSpans,
    ScopeSpans,
    Span,
};
