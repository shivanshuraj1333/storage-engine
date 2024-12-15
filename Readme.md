## Rust Storage Engine for Performance and Faster I/O

Components:
 - Engine Core
 - Config Loader
 - Listener Server
 - MetaData Extractor
 - Storage Adaptor
 - Event Log: Logs Entry

Description of components:
 - Listener Server:
   - listens proto messages over gRPC
   - uses msg.proto for messages
   - puts them in a queue inside Engine Core called raw data queue

 - Engine Core:
   - Central unit for all things related to message processing
   - Picks messages from raw data queue (RDQ)
   - Initiates message metadata extraction using parallel instances of MetaData Extractor component
   - Puts the modified messages coming from metadata extraction unit to another queue call called processed messages
   - Storage writer prepares modified entities called FMM entity, these entities are then compressed in batches and write them to corresponding storage
 
 - Storage Adaptor:
   - Provides interface to plugin different data soruces like GCP cloud storage and s3, everything is controlled via config loader

 - Event Log:
   - Common interface to provide logging using upstream libraries and provides all the information about the msg processing at every stage

 - Config Loader:
   - Loads config for all the components

 - MetaData Extractor:
   - Extracts metadata from all the messages

- Tonic: gRPC framework
- Tokio: Async runtime
- Tracing: Logging infrastructure
- Thiserror: Error handling
- Prost: Protocol buffers

- `client`: Controls client code compilation
- Enables separate builds for server and client components
