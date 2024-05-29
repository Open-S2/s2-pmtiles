import { describe, expect, test } from 'bun:test';
import { readVarint, writeVarint } from '../src/varint';

describe('varint', () => {
  const buffer = { buf: new Uint8Array(0), pos: 0 };
  writeVarint(0, buffer);
  writeVarint(1, buffer);
  writeVarint(127, buffer);
  writeVarint(128, buffer);
  writeVarint(16383, buffer);
  writeVarint(16384, buffer);
  writeVarint(839483929049384, buffer);

  test('writeVarint', () => {
    expect(buffer).toEqual({
      buf: new Uint8Array([
        0, 1, 127, 128, 1, 255, 127, 128, 128, 1, 168, 242, 138, 171, 153, 240, 190, 1, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
      ]),
      pos: 18,
    });
  });

  const resBuffer = { buf: new Uint8Array(buffer.buf.buffer, 0, buffer.pos), pos: 0 };

  test('readVarint', () => {
    expect(readVarint(resBuffer)).toEqual(0);
    expect(readVarint(resBuffer)).toEqual(1);
    expect(readVarint(resBuffer)).toEqual(127);
    expect(readVarint(resBuffer)).toEqual(128);
    expect(readVarint(resBuffer)).toEqual(16383);
    expect(readVarint(resBuffer)).toEqual(16384);
    expect(readVarint(resBuffer)).toEqual(839483929049384);
  });
});
