#!/bin/bash

# Create proto directory
mkdir -p proto

# Download OpenTelemetry proto files
curl -o proto/common.proto https://raw.githubusercontent.com/open-telemetry/opentelemetry-proto/main/opentelemetry/proto/common/v1/common.proto
curl -o proto/trace.proto https://raw.githubusercontent.com/open-telemetry/opentelemetry-proto/main/opentelemetry/proto/trace/v1/trace.proto
curl -o proto/resource.proto https://raw.githubusercontent.com/open-telemetry/opentelemetry-proto/main/opentelemetry/proto/resource/v1/resource.proto

# Create our service definition
cat > proto/service.proto << 'EOF'
syntax = "proto3";

package opentelemetry.proto.collector.trace.v1;

import "trace.proto";
import "common.proto";
import "resource.proto";

// TraceService defines how to receive spans batches
service TraceService {
    // Export spans to the collector
    rpc Export(ExportTraceServiceRequest) returns (ExportTraceServiceResponse) {}
}

// Request for sending a batch of spans to the collector
message ExportTraceServiceRequest {
    // ResourceSpans is the top-level struct that contains resource and spans that belong to it
    repeated opentelemetry.proto.trace.v1.ResourceSpans resource_spans = 1;
}

// Response for sending spans to the collector
message ExportTraceServiceResponse {
    // Empty for now
}
EOF

# Fix imports in all proto files
for file in proto/*.proto; do
    # Replace all long import paths with local ones
    sed -i '' 's|opentelemetry/proto/common/v1/common.proto|common.proto|g' "$file"
    sed -i '' 's|opentelemetry/proto/resource/v1/resource.proto|resource.proto|g' "$file"
    sed -i '' 's|opentelemetry/proto/trace/v1/trace.proto|trace.proto|g' "$file"
    
    # Remove any existing go_package options
    sed -i '' '/option go_package/d' "$file"
done
