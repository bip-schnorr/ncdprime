// CommonJS entrypoint for the native N-API module.
// This is used by CJS consumers and by our ESM wrapper via createRequire.

function loadBinding() {
  try {
    return require('./npm/linux-x64-gnu/ncdprime.node')
  } catch {
    return require('./npm/linux-x64/ncdprime.node')
  }
}

module.exports = loadBinding()
