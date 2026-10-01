const object_map = new Map();
let next_id = 2;

function create_opaque(cls) {
    object_map.set(next_id, cls);
    next_id += 1;
    return next_id - 1;
}

function get_opaque(opt) {
    if (object_map.has(opt)) {
        return object_map[opt];
    }
    return null;
}

function remove_opaque(opt) {
    if (object_map.has(opt)) {
        object_map.delete(opt);
    }
}

class PxsElement {
    constructor(element) {
        this.el = element;
    }

    static create(args) {
        let element_ptr = get_opaque(pxs_arg(args, 0));
        let opaque = create_opaque(new Element());
        let object = pxs_newobject(opaque, remove_opaque, "Element");
    }
    static inner_html_prop(args) {
        // TODO
    }
}

class PxsDom {
    static create(args) {
        let dom_object = pxs_newobject(create_opaque(new PxsDom()), remove_opaque, "Dom");
        pxs_object_addprop(dom_object, "title", PxsDom.title_prop);
        return pxs_newhost(dom_object);
    }
    static title_prop(args) {
        let argc = pxs_argc(args);
        if (argc == 1) {
            return pxs_newstring(document.title);
        } else {
            document.title = pxs_getstring(pxs_arg(args, 1));
            return pxs_newnull();
        }
    }
    static query_selector(args) {
        let argc = pxs_argc(args);
        if (!argc == 1) {
            return pxs_newexception("Expected at least 1 arg.");
        }
    }
}

function setup_browser_apis() {
    let module = pxs_newmod("browser");
    pxs_addvar(module, "dom", pxs_newfactory(PxsDom.create, pxs_newlist()));
    pxs_addfunc(module, "alert", (args) => {
        let argc = pxs_argc(args);
        if (argc > 0) {
            let msg_arg = pxs_arg(args, 0);
            if (!pxs_isstring(msg_arg)) {
                return pxs_newexception("Expected string.");
            }
            alert(pxs_getstring(msg_arg));
        }
        return pxs_newnull();
    });
    pxs_addmod(module);
}

