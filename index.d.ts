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

export function ncdAuto(
  x: Buffer | Uint8Array,
  y: Buffer | Uint8Array,
  opts?: { gzipLevel?: number; gzipMtime?: number },
): number;

export function matrixAuto(
  a: Array<Buffer | Uint8Array>,
  b: Array<Buffer | Uint8Array>,
  opts?: { gzipLevel?: number; gzipMtime?: number },
): Promise<number[][]>;

export function joinFrame64(a: Uint8Array, b: Uint8Array): Uint8Array;
