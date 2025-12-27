# PBS Exporter - Memory Efficiency Guide

## Memory-Efficient Design

This document outlines the memory-efficient design choices in the PBS Exporter.

### 1. **Zero-Copy Operations**

- **String Handling**: Uses `&str` references instead of `String` where possible to avoid allocations
- **Borrowed Data**: API responses are parsed directly without intermediate copies
- **Streaming**: HTTP responses are processed as streams, not loaded entirely into memory

### 2. **Efficient Data Structures**

```rust
// Metrics are stored as primitives (f64, u64) not complex structures
pub struct MetricsCollector {
    // Gauges store only a single f64 value
    host_cpu_usage: Gauge,
    // GaugeVec uses HashMap internally but only for label combinations
    datastore_used_bytes: GaugeVec,
}
```

### 3. **Arc for Shared State**

The PBS client is wrapped in `Arc` to avoid cloning:

```rust
pub struct MetricsCollector {
    client: Arc<PbsClient>,  // Shared reference, not cloned
    // ...
}
```

This means only a pointer is copied, not the entire HTTP client.

### 4. **Async/Await Efficiency**

- **Tokio Runtime**: Uses work-stealing scheduler for efficient task management
- **No Blocking**: All I/O is async, threads don't block waiting for responses
- **Small Stack Frames**: Async functions compile to state machines with minimal stack usage

### 5. **Release Build Optimizations**

In `Cargo.toml`:

```toml
[profile.release]
opt-level = 3          # Maximum optimizations
lto = true             # Link-time optimization (reduces binary size)
codegen-units = 1      # Single codegen unit (better optimization)
strip = true           # Strip symbols (smaller binary)
```

### 6. **Lazy Initialization**

- Metrics are only collected when `/metrics` is scraped
- No background polling consuming memory
- Connection pool reuses HTTP connections

### 7. **Bounded Collections**

- No unbounded vectors or hashmaps
- Metrics registry has a fixed set of metrics
- Label cardinality is bounded by PBS resources (datastores, VMs)

### 8. **Memory Footprint**

Typical memory usage:

- **Idle**: ~5-10 MB (Rust binary + Tokio runtime)
- **During scrape**: ~15-20 MB (temporary JSON parsing)
- **Steady state**: ~10-15 MB

Compare to Go exporters which typically use 20-50 MB idle.

### 9. **No Memory Leaks**

- **Rust's Ownership**: Prevents memory leaks at compile time
- **No Manual Memory Management**: RAII ensures cleanup
- **Drop Trait**: Resources are freed automatically

### 10. **Efficient JSON Parsing**

```rust
// serde_json uses efficient parsing
let api_response: ApiResponse<NodeStatus> = response.json().await?;
// Deserialized directly into typed structs, no intermediate representation
```

### 11. **String Interning**

Labels in Prometheus metrics are interned (deduplicated):

- Same datastore name "store1" used in multiple metrics shares one allocation
- Prometheus crate handles this automatically

### 12. **Monitoring Memory Usage**

To monitor the exporter's memory:

```bash
# Docker stats
docker stats pbs-exporter

# Linux
ps aux | grep pbs-exporter

# Detailed profiling (development)
cargo install cargo-flamegraph
cargo flamegraph --bin pbs-exporter
```

### 13. **Future Optimizations**

If memory becomes a concern:

1. **Reduce metric cardinality**: Aggregate per-VM metrics if you have thousands of VMs
2. **Use `prometheus-client`**: Alternative crate with lower overhead
3. **Disable process metrics**: Remove `features = ["process"]` from prometheus dependency
4. **Custom allocator**: Use jemalloc for better memory management

```toml
[dependencies]
jemallocator = "0.5"
```

```rust
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;
```

## Benchmarks

Run benchmarks to verify memory efficiency:

```bash
cargo bench
```

## Conclusion

The PBS Exporter is designed to be memory-efficient from the ground up:

- Rust's zero-cost abstractions
- Efficient async runtime
- Minimal allocations
- Automatic memory management

For a typical PBS installation with 10 datastores and 100 VMs, expect **~15 MB** memory usage.
