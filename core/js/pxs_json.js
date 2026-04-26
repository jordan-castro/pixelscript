/**
 * Encode a JS object into JSON string.
 * @param {*} obj the object
 * @returns string
 */
export function encode(obj) {
    return JSON.stringify(obj);
}

/**
 * Decode a JSON string into a JS object.
 * @param {*} json_str the string.
 * @returns object
 */
export function decode(json_str) {
    return JSON.parse(json_str);
}

