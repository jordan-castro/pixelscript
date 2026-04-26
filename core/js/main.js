// Main is always loaded at the start of the engine. This goes into GLOBAL scope
import * as pxs from 'pxs_json';

// Expose it to globalThis
globalThis.pxs_json = {
    encode: pxs.encode,
    decode: pxs.decode
};