export function ncd(
  x: Buffer | Uint8Array,
  y: Buffer | Uint8Array,
  gzipLevel?: number,
  gzipMtime?: number,
): number;

export function matrix(
  a: Array<Buffer | Uint8Array>,
  b: Array<Buffer | Uint8Array>,
  gzipLevel?: number,
  gzipMtime?: number,
): number[][];
