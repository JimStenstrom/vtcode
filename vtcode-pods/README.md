# vtcode-pods

GPU pod management for VT Code. Handles pod lifecycle, health monitoring,
and SSH transport for remote development environments.

## Modules

| Module | Purpose |
|---|---|
| `catalog` | Pod catalog and model discovery |
| `manager` | Pod lifecycle management (start, stop, status) |
| `state` | Pod state tracking and transitions |
| `store` | Persistent pod state storage |
| `transport` | SSH and command transport layers |

## Public entrypoints

- `PodManager` – manage pod lifecycle
- `PodCatalog` – discover available pod models
- `PodGpu` – GPU resource information
- `PodHealth` / `PodState` – health and state enums
- `PodsStore` / `PodsState` – persistent state management
- `SshTransport` – SSH-based remote execution

## Usage

```rust
use vtcode_pods::{PodManager, PodCatalog};

let catalog = PodCatalog::load().expect("failed to load catalog");
let manager = PodManager::new(catalog);
```

## API reference

<https://docs.rs/vtcode-pods>
