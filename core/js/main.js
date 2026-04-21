// Main is always loaded at the start of the engine. This goes into GLOBAL scope

// Internal register for JS objects.
// This allows quickjs to handle its own state. It's not the "fastest" way but its up there.
globalThis.PXS_Register = {
    objects: {},
    next_id: 0,
    new_register: function(obj) {
        this.objects[this.next_id] = obj;
        this.next_id += 1;
        return this.next_id;
    }
};
