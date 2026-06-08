const DID_PREFIX = 'did:iota:';

export const isObjectIDDID = (value: string) => value.trim().startsWith(DID_PREFIX);

export const createObjectIDDIDSharePayload = (did: string) =>
  JSON.stringify({
    type: 'objectid.did-share',
    did,
  });

export const parseObjectIDDIDShareQr = (value: string) => {
  const raw = value.trim();
  if (!raw) return '';
  if (isObjectIDDID(raw)) return raw;

  try {
    const url = new URL(raw);
    const did = url.searchParams.get('did')?.trim() ?? '';
    if (isObjectIDDID(did)) return did;
  } catch {
    // Continue with JSON parsing.
  }

  try {
    const parsed = JSON.parse(raw) as { type?: unknown; did?: unknown };
    const did = String(parsed.did ?? '').trim();
    if ((parsed.type === 'objectid.did-share' || parsed.type === 'objectid.identity') && isObjectIDDID(did)) {
      return did;
    }
  } catch {
    // The QR content is not an ObjectID DID share.
  }

  return '';
};
