# Development Guidelines for dive-computer-gfx

This document provides guidelines for developing and testing the dive-computer-gfx project.

## Build/Configuration Instructions

### Prerequisites

The project requires the following dependencies:
- Rust toolchain (stable)
- Local dependencies:
  - embedded-plots
  - embassy-sync
  - dive-computer-deco
  - dive-computer-proto

### Building the Project

1. **Standard Build**:
   ```bash
   cargo build
   ```

2. **Release Build**:
   ```bash
   cargo build --release
   ```

3. **Build with Features**:
   - Simulator (for development and testing):
     ```bash
     cargo build --features "simulator"
     ```
   - Standard library support (for testing):
     ```bash
     cargo build --features "std"
     ```
   - Both simulator and std (recommended for testing):
     ```bash
     cargo build --features "std simulator"
     ```

### Running Examples

The project includes examples that demonstrate its functionality:

```bash
cargo run --example main --features "std simulator"
```

## Testing Information

### Running Tests

The project uses Rust's built-in testing framework. Tests can be run with:

```bash
cargo test --features "std simulator"
```

The `std` and `simulator` features are required for testing because:
- `std` enables standard library features needed for testing
- `simulator` enables the embedded-graphics-simulator for visual testing

### Adding New Tests

#### Integration Tests

1. Create a new file in the `tests` directory with a descriptive name (e.g., `component_test.rs`).
2. Import the necessary modules from the library.
3. Write test functions with the `#[test]` attribute.

Example:

```rust
// tests/state_test.rs
use stm_graphics::State;

#[test]
fn test_state_new() {
    let state = State::new();
    
    // Test default values
    assert_eq!(state.pressure, 20.0);
    assert_eq!(state.temperature, 20.0);
    assert_eq!(state.datetime.date.day, 1);
    assert_eq!(state.datetime.date.month, 1);
    assert_eq!(state.datetime.date.year, 2025);
    assert_eq!(state.datetime.time.hours, 22);
    assert_eq!(state.datetime.time.minutes, 35);
    assert_eq!(state.battery, 0xff);
}
```

#### Unit Tests

For unit tests within the library code:

1. Add a `#[cfg(test)]` module at the bottom of the file you want to test.
2. Import the necessary modules.
3. Write test functions with the `#[test]` attribute.

Example:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_functionality() {
        // Test code here
    }
}
```

## Additional Development Information

### Project Structure

- `src/common`: Common traits and structures used throughout the project
- `src/components`: UI components like buttons, labels, and plots
- `src/scene`: Scene management functionality
- `src/scenes`: Different scenes/screens for the dive computer
- `src/status`: Status bar and related components
- `src/utils`: Utility functions and components

### Code Style

- The project follows Rust's standard naming conventions:
  - `snake_case` for functions, methods, and variables
  - `CamelCase` for types and traits
  - `SCREAMING_SNAKE_CASE` for constants

### No-std Environment

This project is designed to run on embedded systems without the standard library (`no_std`). When developing:

- Avoid using standard library features unless the `std` feature is enabled
- Use `core` instead of `std` where possible
- Use `heapless` collections instead of standard collections
- Be mindful of memory usage and avoid dynamic allocation

### Simulator Mode

The simulator mode (enabled with the `simulator` feature) allows for development and testing on a desktop environment using the `embedded-graphics-simulator` crate. This is useful for:

- Rapid UI development and testing
- Debugging without deploying to hardware
- Running automated tests

### Debugging Tips

- Use the simulator mode for visual debugging
- Enable the `std` feature for more comprehensive error messages during development
- Use `defmt` for logging in the embedded environment