export interface ObjectIDSeedSharePayload {
  type: string;
  version: number;
  did: string;
  network: 'testnet' | 'mainnet';
  address?: string;
  encryptedSeed: string;
  encryption?: {
    alg?: string;
    format?: string;
  };
}

function norm(value: unknown): string {
  return String(value ?? '').trim();
}

function strip0x(value: string): string {
  return value.replace(/^0x/i, '');
}

function isHex(value: string): boolean {
  return /^[0-9a-fA-F]+$/.test(value);
}

function b64urlToB64(value: string): string {
  const normalized = value.replace(/-/g, '+').replace(/_/g, '/');
  const pad = (4 - (normalized.length % 4)) % 4;
  return normalized + '='.repeat(pad);
}

function base64ToBytes(value: string): Uint8Array {
  const bin = atob(value);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i += 1) out[i] = bin.charCodeAt(i);
  return out;
}

function base64UrlDecodeToBytes(value: string): Uint8Array {
  return base64ToBytes(b64urlToB64(norm(value)));
}

function base64UrlDecodeToJson(value: string): unknown {
  const bytes = base64UrlDecodeToBytes(value);
  return JSON.parse(new TextDecoder().decode(bytes));
}

function u8ToArrayBuffer(value: Uint8Array): ArrayBuffer {
  return value.buffer.slice(value.byteOffset, value.byteOffset + value.byteLength) as ArrayBuffer;
}

async function deriveAesKey(password: string, salt: Uint8Array, iterations: number): Promise<CryptoKey> {
  const passwordKey = await crypto.subtle.importKey('raw', new TextEncoder().encode(password), 'PBKDF2', false, [
    'deriveKey',
  ]);

  return await crypto.subtle.deriveKey(
    { name: 'PBKDF2', salt: u8ToArrayBuffer(salt), iterations, hash: 'SHA-256' },
    passwordKey,
    { name: 'AES-GCM', length: 256 },
    false,
    ['decrypt'],
  );
}

export function parseObjectIDSeedShareQr(value: string): ObjectIDSeedSharePayload | null {
  const raw = norm(value);
  if (!raw) return null;

  let encodedPayload: string;
  try {
    const url = new URL(raw);
    if (url.protocol !== 'objectid-wallet-seed-share:') return null;
    encodedPayload = norm(url.searchParams.get('payload'));
  } catch {
    try {
      const parsed = JSON.parse(raw);
      if (parsed?.type !== 'objectid.wallet-seed-share') return null;
      return normalizePayload(parsed);
    } catch {
      return null;
    }
  }

  if (!encodedPayload) throw new Error('ObjectID wallet QR is missing the payload.');
  return normalizePayload(base64UrlDecodeToJson(encodedPayload));
}

function normalizePayload(value: unknown): ObjectIDSeedSharePayload {
  const payload = value as Record<string, unknown>;
  if (payload?.type !== 'objectid.wallet-seed-share') {
    throw new Error('This QR is not an ObjectID wallet configuration QR.');
  }

  const did = norm(payload.did);
  const encryptedSeed = norm(payload.encryptedSeed);
  const network = norm(payload.network).toLowerCase() === 'mainnet' ? 'mainnet' : 'testnet';
  if (!did) throw new Error('ObjectID wallet QR is missing the DID.');
  if (!encryptedSeed) throw new Error('ObjectID wallet QR is missing the encrypted seed.');

  return {
    type: 'objectid.wallet-seed-share',
    version: Number(payload.version ?? 1),
    did,
    network,
    address: norm(payload.address) || undefined,
    encryptedSeed,
    encryption: payload.encryption as ObjectIDSeedSharePayload['encryption'],
  };
}

export async function decryptObjectIDSeedShare(payload: ObjectIDSeedSharePayload, password: string): Promise<string> {
  const jwt = norm(payload.encryptedSeed);
  if (!jwt) throw new Error('Missing encrypted seed token.');
  if (!password) throw new Error('Password is required.');

  const parts = jwt.split('.');
  if (parts.length < 2) throw new Error('Invalid encrypted seed token.');

  let header: Record<string, unknown> | null;
  try {
    header = base64UrlDecodeToJson(parts[0]) as Record<string, unknown>;
  } catch {
    header = null;
  }
  const alg = norm(header?.alg);
  if (alg && alg !== 'none') throw new Error('Unsupported encrypted seed token algorithm.');

  const tokenPayload = base64UrlDecodeToJson(parts[1]) as Record<string, unknown>;
  const version = Number(tokenPayload.v ?? 1);
  if (version !== 1) throw new Error('Unsupported encrypted seed token version.');

  const iterations = Number(tokenPayload.it ?? 0);
  if (!Number.isFinite(iterations) || iterations < 10_000) {
    throw new Error('Invalid encrypted seed token parameters.');
  }

  const salt = base64UrlDecodeToBytes(String(tokenPayload.s ?? ''));
  const iv = base64UrlDecodeToBytes(String(tokenPayload.iv ?? ''));
  const ciphertext = base64UrlDecodeToBytes(String(tokenPayload.ct ?? ''));
  if (salt.length !== 16) throw new Error('Invalid encrypted seed token salt.');
  if (iv.length !== 12) throw new Error('Invalid encrypted seed token IV.');
  if (!ciphertext.length) throw new Error('Invalid encrypted seed token ciphertext.');

  const key = await deriveAesKey(password, salt, iterations);

  let plaintext: ArrayBuffer;
  try {
    plaintext = await crypto.subtle.decrypt(
      { name: 'AES-GCM', iv: u8ToArrayBuffer(iv) },
      key,
      u8ToArrayBuffer(ciphertext),
    );
  } catch {
    throw new Error('Wrong password or corrupted QR.');
  }

  const seed = strip0x(new TextDecoder().decode(new Uint8Array(plaintext)).trim()).toLowerCase();
  if (!seed || !matchesSeed(seed)) throw new Error('Decrypted seed is invalid.');
  return seed;
}

function matchesSeed(seed: string): boolean {
  return (seed.length === 64 || seed.length === 128) && isHex(seed);
}
