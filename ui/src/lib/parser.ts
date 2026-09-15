// I really, really, really despise how JS doesn't support large integers properly.
export const parseSafeJSON = (data: string) => {
	return JSON.parse(data, reviver);
};
export const reviver = (key: any, value: any, context: any) => {
	// If _anything_ isn't a safe int, that's actually an int, treat it as a big int
	if (typeof value === 'number' && Number.isInteger(value) && !Number.isSafeInteger(value)) {
		return BigInt(context.source);
	}
	return value;
};
