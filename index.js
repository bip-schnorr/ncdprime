// JS wrapper for the native N-API module.
// After `napi build`, the binary will be placed under `npm/` directories.
// For now we load from the build output when present.

let binding;
try {
  // napi-rs layout when using `napi build`.
  binding = require('./npm/linux-x64-gnu/ncdprime.node');
} catch {
  // Fallback for dev setups; user can adjust.
  binding = require('./npm/linux-x64/ncdprime.node');
}

module.exports = binding;
