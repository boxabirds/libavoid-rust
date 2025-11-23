# libavoid-rust JavaScript Tests

Test suite for verifying WASM/JS binding parity with libavoid-js.

## Quick Start

```bash
# Install dependencies
npm install

# Build WASM (from this directory)
npm run build:wasm

# Run all tests
npm test

# Run in watch mode
npm run test:watch

# Run with coverage
npm run test:coverage
```

## Test Structure

```
js-tests/
├── unit/                  # Unit tests for individual classes
│   ├── point.test.js
│   ├── router.test.js
│   └── connref.test.js
├── compatibility/         # API surface compatibility tests
│   └── api-surface.test.js
├── parity/               # Behavioral comparison with libavoid-js
│   └── compare-routes.test.js
├── integration/          # Full workflow tests
│   └── main-example.test.js
├── setup.js              # Test setup (loads both implementations)
├── vitest.config.js      # Vitest configuration
└── package.json
```

## Test Categories

### Unit Tests (`unit/`)
Test individual classes and methods work correctly.

### Compatibility Tests (`compatibility/`)
Verify our API surface matches libavoid-js:
- All required classes exist
- All required constants exist
- All required methods exist

### Parity Tests (`parity/`)
Run same operations on both implementations and compare results:
- Same inputs should produce same (or equivalent) outputs
- Requires both libavoid-rust and libavoid-js to be loaded

### Integration Tests (`integration/`)
Port actual libavoid-js examples to verify real-world usage patterns work.

## Prerequisites

1. Rust toolchain with wasm-pack:
   ```bash
   cargo install wasm-pack
   ```

2. Node.js 18+

3. The WASM build must succeed:
   ```bash
   npm run build:wasm
   ```

## Writing Tests

Tests use [Vitest](https://vitest.dev/) with globals enabled.

```javascript
import { describe, it, expect } from 'vitest';

describe('MyClass', () => {
  it.skipIf(!globalThis.Avoid)('does something', () => {
    // globalThis.Avoid - our implementation
    // globalThis.AvoidJS - reference implementation
    const obj = new globalThis.Avoid.MyClass();
    expect(obj).toBeDefined();
  });
});
```

Use `it.skipIf(!globalThis.Avoid)` to skip tests when WASM isn't loaded.

## CI Integration

Tests run automatically in CI. See `.github/workflows/test.yml`.

The CI workflow:
1. Builds the WASM module
2. Runs all tests
3. Reports coverage
