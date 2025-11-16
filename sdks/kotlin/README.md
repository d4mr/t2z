# PCZT Kotlin SDK

Kotlin bindings for working with Partially Constructed Zcash Transactions (PCZT).

## Installation

Add to your `build.gradle.kts`:

```kotlin
dependencies {
    implementation("io.github.d4mr:pczt-kotlin:0.1.0")
}
```

## Building

1. First, build the Rust wrapper with UniFFI bindings:

```bash
cd ../../wrapper
make build-uniffi
```

2. The Kotlin bindings will be generated in `sdks/kotlin/generated/`

3. Build the Kotlin module:

```bash
./gradlew build
```

## Usage

```kotlin
import io.github.d4mr.pczt.*

fun main() {
    // Example usage
    // TODO: Add example after bindings are generated
}
```

## Status

🚧 Under development - UniFFI bindings need to be generated.

