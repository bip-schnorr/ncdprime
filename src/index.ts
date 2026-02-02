// Public JS entrypoint for library consumers.
//
// This module is safe to import even when the native backend isn't built.

export { ncdAuto, matrixAuto, joinFrame64 } from "./api.js";
export { ncd, ncdFromSizes, gzipCompressor } from "./ncd.js";
export { computeMatrix, formatMatrix } from "./matrix.js";
export type { Compressor, Bytes } from "./ncd.js";
export type { Matrix } from "./matrix.js";
