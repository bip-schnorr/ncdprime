// CommonJS entrypoint for the native N-API module.
// This is used by CJS consumers and by our ESM wrapper via createRequire.

function loadBinding() {
  // Preferred (napi build --output-dir npm)
  try {
    return require('./npm/index.node')
  } catch {}

  // Older/manual layouts
  try {
    return require('./npm/linux-x64-gnu/ncdprime.node')
  } catch {}
  return require('./npm/linux-x64/ncdprime.node')
}

module.exports = loadBinding()
