import { SchemaType, deserializeSchemaType } from "@concordium/web-sdk";
import { Buffer } from "buffer/";

export const parseUiToContract = <TUi, TContract>(
	json: TUi,
	schemaBase64: string,
): TContract => {
	const schemaType = deserializeSchemaType(Buffer.from(schemaBase64, "base64"));
	return parseContractRequest(json, schemaType);
};

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const parseContractRequest = (json: any, type: SchemaType): any => {
	switch (type.type) {
		case "Struct": {
			switch (type.fields.type) {
				case "None":
					return {};
				case "Named": {
					const ret = {} as Record<string, unknown>;
					for (const field of type.fields.fields) {
						ret[field.name] = parseContractRequest(
							(json as Record<string, unknown>)[field.name],
							field.field,
						);
					}

					return ret;
				}
				case "Unnamed": {
					const ret = [] as unknown[];
					for (const field of type.fields.fields) {
						ret.push(
							parseContractRequest((json as unknown[])[ret.length], field),
						);
					}

					return ret;
				}
			}
			break;
		}
		case "Enum": {
			const variantType = type.variants.find(
				(variant) => variant.name === json.tag,
			)!;
			const variantValue = json[json.tag];
			const ret = {
				[json.tag]: parseContractRequest(variantValue, {
					type: "Struct",
					fields: variantType.fields,
				}),
			};
			return ret;
		}
		case "Pair": {
			return [
				parseContractRequest(json[0], type.first),
				parseContractRequest(json[1], type.second),
			];
		}
		case "List":
		case "Set":
		case "Array": {
			return json.map((item: unknown) => parseContractRequest(item, type.item));
		}
		case "Map": {
			const value = json as [unknown, unknown][];
			const ret = [];
			for (const [key, val] of value) {
				ret.push([
					parseContractRequest(key, type.key),
					parseContractRequest(val, type.value),
				]);
			}

			return ret;
		}
		case "TaggedEnum": {
			const variants = Array.from(type.variants.values()!);
			return parseContractRequest(json, { type: "Enum", variants });
		}
		default:
			return json;
	}
};

// Parses Contract Response to UI friendly format
export const parseContractToUi = <TUi, TContract>(
	json: TContract,
	schemaBase64: string,
): TUi => {
	const schemaType = deserializeSchemaType(Buffer.from(schemaBase64, "base64"));
	console.info("parseContractToUi", json, schemaType);
	return parseContractResponse(json, schemaType) as TUi;
};

const parseContractResponse = (json: unknown, type: SchemaType): unknown => {
	console.info("parseContractResponse", json, type);
	switch (type.type) {
		case "ContractAddress": {
			const val = json as { index: bigint; subindex: bigint };
			return { index: Number(val.index), subindex: Number(val.subindex) };
		}
		case "Struct": {
			switch (type.fields.type) {
				case "None":
					return undefined;
				case "Named": {
					const ret = {} as Record<string, unknown>;
					for (const field of type.fields.fields) {
						ret[field.name] = parseContractResponse(
							(json as Record<string, unknown>)[field.name],
							field.field,
						);
					}
					return ret;
				}
				case "Unnamed": {
					const ret = [] as unknown[];
					for (const field of type.fields.fields) {
						ret.push(
							parseContractResponse((json as unknown[])[ret.length], field),
						);
					}
					return ret;
				}
			}
			break;
		}
		case "Enum": {
			for (const variant of type.variants) {
				if (variant.name in (json as Record<string, unknown>)) {
					return {
						tag: variant.name,
						[variant.name]: parseContractResponse(
							(json as Record<string, unknown>)[variant.name],
							{
								type: "Struct",
								fields: variant.fields,
							},
						),
					};
				}
			}
			console.warn("No Variant found out of ", type.variants, json);
			throw new Error("No Variant found");
		}
		case "Pair": {
			return [
				parseContractResponse((json as [unknown, unknown])[0], type.first),
				parseContractResponse((json as [unknown, unknown])[1], type.second),
			];
		}
		case "List":
		case "Set":
		case "Array": {
			return (json as unknown[]).map((item: unknown) =>
				parseContractResponse(item, type.item),
			);
		}
		case "Map": {
			const value = json as [unknown, unknown][];
			const ret = [];
			for (const [key, val] of value) {
				ret.push([
					parseContractResponse(key, type.key),
					parseContractResponse(val, type.value),
				]);
			}

			return ret;
		}
		case "TaggedEnum": {
			const variants = Array.from(type.variants.values()!);
			return parseContractResponse(json, { type: "Enum", variants });
		}
		case "I128":
		case "I64":
		case "I32":
		case "I16":
		case "I8":
		case "U128":
		case "U64":
		case "U32":
		case "U16":
		case "U8": {
			console.log("Number", json);
			return Number(json);
		}
		case "ILeb128":
		case "ULeb128":
			return String(json);
		default:
			return json;
	}
};
