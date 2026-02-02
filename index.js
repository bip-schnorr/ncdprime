// ESM entrypoint for the native N-API module.
// Works when package.json has `type: "module"`.

import { createRequire } from 'node:module'

const require = createRequire(import.meta.url)

let binding
try {
  binding = require('./npm/linux-x64-gnu/ncdprime.node')
} catch {
  binding = require('./npm/linux-x64/ncdprime.node')
}

export const ncd = binding.ncd
export const matrix = binding.matrix
export default binding
