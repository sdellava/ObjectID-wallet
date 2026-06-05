import { createOid } from '@objectid/oid-provider/oid';

export type ObjectIDMoveEdge = {
  node: {
    address: string;
    asMoveObject?: {
      contents?: {
        type?: {
          repr?: string;
        };
        data?: {
          Struct?: Array<{
            name: string;
            value: unknown;
          }>;
        };
      };
    };
  };
};

export type ObjectIDProduct = {
  id: string;
  network: string;
  typeRepr: string;
  title: string;
  description: string;
  imageUrl: string;
  fields: Record<string, unknown>;
};

const oid = createOid();

const normalizeDid = (did: unknown) =>
  String(did ?? '')
    .trim()
    .toLowerCase()
    .replace(/[#?].*$/, '')
    .replace(/\/+$/, '');

const didContains = (value: unknown, did: string) => {
  const normalizedValue = normalizeDid(value);
  const normalizedDid = normalizeDid(did);
  return !!normalizedValue && !!normalizedDid && normalizedValue.includes(normalizedDid);
};

const extractMoveValue = (value: unknown): unknown => {
  if (value === null || value === undefined) return '';
  if (typeof value === 'string' || typeof value === 'number' || typeof value === 'boolean') return value;

  if (Array.isArray(value)) return value.map(extractMoveValue);

  if (typeof value === 'object') {
    const record = value as Record<string, unknown>;
    if ('String' in record) return record.String;
    if ('Number' in record) return record.Number;
    if ('Bool' in record) return record.Bool;
    if ('Address' in record) return record.Address;
    if ('Vector' in record) return extractMoveValue(record.Vector);
    if ('vector' in record) return extractMoveValue(record.vector);
  }

  return value;
};

export const structToFields = (edge: ObjectIDMoveEdge): Record<string, unknown> => {
  const rpcFields = (edge as unknown as { data?: { content?: { fields?: Record<string, unknown> } } }).data?.content
    ?.fields;
  if (rpcFields && typeof rpcFields === 'object') return rpcFields;

  const struct = edge.node.asMoveObject?.contents?.data?.Struct ?? [];
  const fields: Record<string, unknown> = {};

  for (const field of struct) {
    fields[field.name] = extractMoveValue(field.value);
  }

  return fields;
};

const firstString = (...values: unknown[]) => {
  for (const value of values) {
    const str = String(value ?? '').trim();
    if (str) return str;
  }
  return '';
};

export const normalizeProductImageUrl = (value: unknown) => {
  const imageUrl = String(value ?? '').trim();
  if (!imageUrl) return '';

  if (/^(https?:|data:image\/|blob:)/i.test(imageUrl)) return imageUrl;

  return '';
};

const parseMetadata = (value: unknown): Record<string, unknown> => {
  if (!value) return {};
  if (typeof value === 'object' && !Array.isArray(value)) return value as Record<string, unknown>;
  if (typeof value !== 'string') return {};

  try {
    const parsed = JSON.parse(value);
    return parsed && typeof parsed === 'object' && !Array.isArray(parsed) ? parsed : {};
  } catch {
    return {};
  }
};

const productFromEdge = (edge: ObjectIDMoveEdge, network: string): ObjectIDProduct => {
  const rawFields = structToFields(edge);
  const mutableMetadata = parseMetadata(rawFields.mutable_metadata ?? rawFields.metadata ?? rawFields.mutableMetadata);
  const immutableMetadata = parseMetadata(rawFields.immutable_metadata ?? rawFields.immutableMetadata);
  const metadata = { ...immutableMetadata, ...mutableMetadata };
  const fields = { ...metadata, ...rawFields };

  const typeRepr = edge.node.asMoveObject?.contents?.type?.repr ?? '';
  const objectType = firstString(fields.object_type, metadata.object_type);
  const title = firstString(fields.name, fields.title, metadata.name, metadata.title, objectType, 'ObjectID object');
  const description = firstString(
    fields.description,
    fields.product_description,
    fields.product_desc,
    metadata.description,
    metadata.product_description,
    metadata.product_desc,
    'No description available.',
  );
  const imageUrl = normalizeProductImageUrl(
    firstString(
      fields.product_img_url,
      fields.product_image_url,
      fields.image_url,
      fields.image,
      metadata.product_img_url,
      metadata.product_image_url,
      metadata.image_url,
      metadata.image,
    ),
  );

  return {
    id: edge.node.address,
    network,
    typeRepr,
    title,
    description,
    imageUrl,
    fields,
  };
};

const sortProducts = (products: ObjectIDProduct[]) =>
  [...products].sort((a, b) => {
    const at = Number(
      a.fields.last_update ?? a.fields.updated_at ?? a.fields.creation_date ?? a.fields.created_at ?? 0,
    );
    const bt = Number(
      b.fields.last_update ?? b.fields.updated_at ?? b.fields.creation_date ?? b.fields.created_at ?? 0,
    );
    return bt - at;
  });

const normalizeHex = (value: unknown) => {
  const text = String(value ?? '')
    .trim()
    .toLowerCase();
  if (!text) return '';
  return text.startsWith('0x') ? text : `0x${text}`;
};

const getObjectPackages = async (network: string): Promise<string[]> => {
  const config = (await oid.session.config(network)) as Record<string, unknown>;
  const raw = config.objectPackages ?? config.object_packages ?? [];
  const list = Array.isArray(raw) ? raw : [raw];
  return Array.from(new Set(list.map(normalizeHex).filter(Boolean)));
};

const edgeOwnerContainsDid = (edge: ObjectIDMoveEdge, did: string) => {
  const fields = structToFields(edge);
  return didContains(fields.owner_did, did) || didContains(fields.owner, did);
};

const fetchEdgesByType = async (
  typeRepr: string,
  ownerAddress: string,
  network: string,
): Promise<ObjectIDMoveEdge[]> => {
  if (ownerAddress) {
    const owned = (await oid.getObjectsByTypeAndOwner(typeRepr, ownerAddress, network)) as ObjectIDMoveEdge[];
    if (owned.length) return owned;
  }

  return (await oid.getObjectsByType(typeRepr, network)) as ObjectIDMoveEdge[];
};

export const loadOwnedObjectProducts = async (args: {
  did: string;
  ownerAddress?: string | null;
  network: string;
}): Promise<ObjectIDProduct[]> => {
  const did = args.did.trim();
  const network = args.network.trim() || 'testnet';
  const ownerAddress = String(args.ownerAddress ?? '').trim();

  if (!did) return [];

  const packages = await getObjectPackages(network);
  const types = packages.map((packageId) => `${packageId}::oid_object::OIDObject`);

  const edges = (await Promise.all(types.map((type) => fetchEdgesByType(type, ownerAddress, network)))).flat();
  const products = edges
    .filter((edge) => edgeOwnerContainsDid(edge, did))
    .map((edge) => productFromEdge(edge, network));

  return sortProducts(products);
};

export const loadObjectIDProduct = async (id: string, network: string): Promise<ObjectIDProduct | null> => {
  const productId = id.trim();
  if (!productId) return null;

  const object = (await oid.getObject(productId, network || 'testnet')) as unknown;
  if (!object) return null;

  const objectData = object as {
    objectId?: string;
    type?: string;
    display?: {
      data?: Record<string, unknown>;
    };
    content?: {
      type?: string;
      fields?: Record<string, unknown>;
    };
    data?: {
      objectId?: string;
      type?: string;
      display?: {
        data?: Record<string, unknown>;
      };
      content?: {
        type?: string;
        fields?: Record<string, unknown>;
      };
    };
  };

  const directFields = objectData.content?.fields;
  const wrappedFields = objectData.data?.content?.fields;
  const displayFields = objectData.display?.data ?? objectData.data?.display?.data ?? {};
  const fields = {
    ...displayFields,
    ...(wrappedFields ?? directFields ?? {}),
  };
  const typeRepr =
    objectData.type ?? objectData.content?.type ?? objectData.data?.type ?? objectData.data?.content?.type ?? '';
  const objectId = objectData.objectId ?? objectData.data?.objectId ?? productId;

  return productFromEdge(
    {
      data: {
        content: {
          fields,
        },
      },
      node: {
        address: objectId,
        asMoveObject: {
          contents: {
            type: {
              repr: typeRepr,
            },
          },
        },
      },
    } as unknown as ObjectIDMoveEdge,
    network || 'testnet',
  );
};

export const updateObjectGeolocation = async (args: {
  did: string;
  seed: string;
  network: string;
  objectId: string;
  geolocation: string;
}) => {
  const did = args.did.trim();
  const seed = args.seed.trim();
  const network = args.network.trim() || 'testnet';
  const objectId = args.objectId.trim();
  const geolocation = args.geolocation.trim();

  if (!did) throw new Error('Distributed Identity is missing.');
  if (!seed) throw new Error('Wallet seed is missing.');
  if (!objectId) throw new Error('Object ID is missing.');
  if (!geolocation) throw new Error('Device location is unavailable.');

  await oid.connect({ did, seed, network });

  const creditToken = oid.session.creditToken();
  const controllerCap = oid.session.oidControllerCap;

  if (!creditToken) throw new Error('No ObjectID credit token is available for this wallet.');
  if (!controllerCap) throw new Error('ObjectID controller cap is not available for this identity.');

  const result = await oid.update_geolocation({
    creditToken,
    controllerCap,
    object: objectId,
    new_location: geolocation,
  });

  if (!result?.success) {
    const errorValue = result?.error ?? result?.status?.error ?? 'Update geolocation transaction failed.';
    throw new Error(errorValue instanceof Error ? errorValue.message : String(errorValue));
  }

  return result;
};

export const formatProductFieldLabel = (key: string) =>
  key
    .replace(/_/g, ' ')
    .replace(/\b\w/g, (char) => char.toUpperCase())
    .trim();
