/** A buffer with the position to read from */
export interface BufferPosition {
  buf: Uint8Array;
  pos: number;
}

/**
 * @param low - the low 32 bits of the number
 * @param high - the high 32 bits of the number
 * @returns - the decoded number
 */
function toNum(low: number, high: number): number {
  return (high >>> 0) * 0x100000000 + (low >>> 0);
}

/**
 * @param bufPos - the buffer with it's position
 * @returns - the decoded number
 */
export function readVarint(bufPos: BufferPosition): number {
  const buf = bufPos.buf;
  let b = buf[bufPos.pos++];
  let val = b & 0x7f;
  if (b < 0x80) return val;
  b = buf[bufPos.pos++];
  val |= (b & 0x7f) << 7;
  if (b < 0x80) return val;
  b = buf[bufPos.pos++];
  val |= (b & 0x7f) << 14;
  if (b < 0x80) return val;
  b = buf[bufPos.pos++];
  val |= (b & 0x7f) << 21;
  if (b < 0x80) return val;
  b = buf[bufPos.pos];
  val |= (b & 0x0f) << 28;

  return readVarintRemainder(val, bufPos);
}

/**
 * @param low - the low 32 bits of the number
 * @param bufPos - the buffer with it's position
 * @returns - the decoded remainder
 */
export function readVarintRemainder(low: number, bufPos: BufferPosition): number {
  const buf = bufPos.buf;
  let b = buf[bufPos.pos++];
  let h = (b & 0x70) >> 4;
  if (b < 0x80) return toNum(low, h);
  b = buf[bufPos.pos++];
  h |= (b & 0x7f) << 3;
  if (b < 0x80) return toNum(low, h);
  b = buf[bufPos.pos++];
  h |= (b & 0x7f) << 10;
  if (b < 0x80) return toNum(low, h);
  b = buf[bufPos.pos++];
  h |= (b & 0x7f) << 17;
  if (b < 0x80) return toNum(low, h);
  b = buf[bufPos.pos++];
  h |= (b & 0x7f) << 24;
  if (b < 0x80) return toNum(low, h);
  b = buf[bufPos.pos++];
  h |= (b & 0x01) << 31;
  if (b < 0x80) return toNum(low, h);
  throw new Error('Expected varint not more than 10 bytes');
}

/**
 * Write a varint. Can be max 64-bits. Numbers are coerced to an unsigned
 * while number before using this function.
 * @param val - any whole unsigned number.
 * @param bufPos - the buffer with it's position to write at
 */
export function writeVarint(val: number, bufPos: BufferPosition): void {
  if (val > 0xfffffff || val < 0) {
    writeBigVarint(val, bufPos);
    return;
  }

  realloc(bufPos, 4);

  bufPos.buf[bufPos.pos++] = (val & 0x7f) | (val > 0x7f ? 0x80 : 0);
  if (val <= 0x7f) return;
  bufPos.buf[bufPos.pos++] = ((val >>>= 7) & 0x7f) | (val > 0x7f ? 0x80 : 0);
  if (val <= 0x7f) return;
  bufPos.buf[bufPos.pos++] = ((val >>>= 7) & 0x7f) | (val > 0x7f ? 0x80 : 0);
  if (val <= 0x7f) return;
  bufPos.buf[bufPos.pos++] = (val >>> 7) & 0x7f;
}

/**
 * @param val - the number
 * @param bufPos - the buffer with it's position to write at
 */
export function writeBigVarint(val: number, bufPos: BufferPosition): void {
  let low = val % 0x100000000 | 0;
  let high = (val / 0x100000000) | 0;

  if (val < 0) {
    low = ~(-val % 0x100000000);
    high = ~(-val / 0x100000000);

    if ((low ^ 0xffffffff) !== 0) {
      low = (low + 1) | 0;
    } else {
      low = 0;
      high = (high + 1) | 0;
    }
  }

  if (val >= 0x10000000000000000n || val < -0x10000000000000000n) {
    throw new Error("Given varint doesn't fit into 10 bytes");
  }

  realloc(bufPos, 10);

  writeBigVarintLow(low, high, bufPos);
  writeBigVarintHigh(high, bufPos);
}

/**
 * @param low - lower 32 bits
 * @param _high - unused "high" bits
 * @param bufPos - the buffer with it's position to write at
 */
export function writeBigVarintLow(low: number, _high: number, bufPos: BufferPosition): void {
  bufPos.buf[bufPos.pos++] = (low & 0x7f) | 0x80;
  low >>>= 7;
  bufPos.buf[bufPos.pos++] = (low & 0x7f) | 0x80;
  low >>>= 7;
  bufPos.buf[bufPos.pos++] = (low & 0x7f) | 0x80;
  low >>>= 7;
  bufPos.buf[bufPos.pos++] = (low & 0x7f) | 0x80;
  low >>>= 7;
  bufPos.buf[bufPos.pos] = low & 0x7f;
}

/**
 * @param high - the high 32 bits
 * @param bufPos - the buffer with it's position to write at
 */
export function writeBigVarintHigh(high: number, bufPos: BufferPosition): void {
  const lsb = (high & 0x07) << 4;

  bufPos.buf[bufPos.pos++] |= lsb | ((high >>>= 3) !== 0 ? 0x80 : 0);
  if (high === 0) return;
  bufPos.buf[bufPos.pos++] = (high & 0x7f) | ((high >>>= 7) !== 0 ? 0x80 : 0);
  if (high === 0) return;
  bufPos.buf[bufPos.pos++] = (high & 0x7f) | ((high >>>= 7) !== 0 ? 0x80 : 0);
  if (high === 0) return;
  bufPos.buf[bufPos.pos++] = (high & 0x7f) | ((high >>>= 7) !== 0 ? 0x80 : 0);
  if (high === 0) return;
  bufPos.buf[bufPos.pos++] = (high & 0x7f) | ((high >>>= 7) !== 0 ? 0x80 : 0);
  if (high === 0) return;
  bufPos.buf[bufPos.pos++] = high & 0x7f;
}

/**
 * Allocate more space in the buffer
 * @param bufPos - the buffer with it's position
 * @param min - the minimum number of bytes to allocate
 */
function realloc(bufPos: BufferPosition, min: number): void {
  let length = bufPos.buf.length > 0 ? bufPos.buf.length : 16;

  while (length < bufPos.pos + min) length *= 2;

  if (length !== bufPos.buf.length) {
    const buf = new Uint8Array(length);
    buf.set(bufPos.buf);
    bufPos.buf = buf;
  }
}
