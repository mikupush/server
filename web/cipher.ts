const ENCRYPTION_BLOCK_SIZE = 8 * 1024 * 1024;
const NONCE_SIZE = 12;
const TAG_SIZE = 16;
const ENCRYPTED_BLOCK_SIZE = ENCRYPTION_BLOCK_SIZE + NONCE_SIZE + TAG_SIZE;

export function decodeEncryptionKey(fragment: string): string | undefined {
  const encoded = fragment.startsWith('#') ? fragment.slice(1) : fragment;

  if (!encoded) {
    return undefined;
  }

  const hexKey = atob(encoded.replace(/-/g, '+').replace(/_/g, '/'));

  if (!/^[a-f0-9]{64}$/i.test(hexKey)) {
    throw new Error('Invalid encryption key');
  }

  return hexKey;
}

export async function createDecryptionStream(hexKey: string): Promise<TransformStream<Uint8Array, Uint8Array>> {
  if (!/^[a-f0-9]{64}$/i.test(hexKey)) {
    throw new Error('Invalid encryption key');
  }

  const keyBytes = new Uint8Array(32);

  for (let index = 0; index < keyBytes.length; index += 1) {
    keyBytes[index] = Number.parseInt(hexKey.slice(index * 2, index * 2 + 2), 16);
  }

  const key = await crypto.subtle.importKey('raw', keyBytes, 'AES-GCM', false, ['decrypt']);
  const buffer = new Uint8Array(ENCRYPTED_BLOCK_SIZE);
  let buffered = 0;

  async function decryptBlock(block: Uint8Array<ArrayBuffer>): Promise<Uint8Array> {
    if (block.byteLength <= NONCE_SIZE + TAG_SIZE) {
      throw new Error('Invalid encrypted block size');
    }

    const plaintext = await crypto.subtle.decrypt(
      { name: 'AES-GCM', iv: block.subarray(0, NONCE_SIZE), tagLength: TAG_SIZE * 8 },
      key,
      block.subarray(NONCE_SIZE)
    );

    return new Uint8Array(plaintext);
  }

  return new TransformStream<Uint8Array, Uint8Array>({
    transform: async (chunk: Uint8Array, controller: TransformStreamDefaultController<Uint8Array>) => {
      let offset = 0;

      while (offset < chunk.byteLength) {
        const length = Math.min(buffer.byteLength - buffered, chunk.byteLength - offset);
        buffer.set(chunk.subarray(offset, offset + length), buffered);
        buffered += length;
        offset += length;

        if (buffered === buffer.byteLength) {
          controller.enqueue(await decryptBlock(buffer));
          buffered = 0;
        }
      }
    },
    flush: async (controller: TransformStreamDefaultController<Uint8Array>) => {
      if (buffered > 0) {
        controller.enqueue(await decryptBlock(buffer.subarray(0, buffered)));
      }
    }
  });
}
