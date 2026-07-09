use etffi::ptr_magic::PtrMagic;
use pixelscript::{pxs_addmod, pxs_addobject, pxs_arenaput, pxs_arg, pxs_exec, pxs_finalize, pxs_freearena, pxs_freevar, pxs_getint, pxs_getrt, pxs_gettype, pxs_initialize, pxs_listadd, pxs_newarena, pxs_newcopy, pxs_newexception, pxs_newhost, pxs_newint, pxs_newlist, pxs_newmod, pxs_newnull, pxs_newobject, pxs_newtype, pxs_object_addfunc, pxs_varis, shared::{pxs_Opaque, pxs_Runtime, var::{pxs_VarT, pxs_VarType}}};

fn main() {
    pxs_initialize();
    let pxs = pxs_newmod(c"pxs".as_ptr());
    pxs_addobject(pxs, c"Vector2".as_ptr(), Vector2::new);
    pxs_addmod(pxs);
    divan::main();
    pxs_finalize();
}

struct Vector2 {
    x: i64,
    y: i64
}

impl PtrMagic for Vector2 {}

impl Vector2 {
    unsafe extern "C" fn free(ptr: pxs_Opaque) {
        let _ = Self::from_raw(ptr as *mut Vector2);
    }

    unsafe extern "C" fn add(args: pxs_VarT) -> pxs_VarT {
        let this = pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), 0);
        if this.is_null() {
            return pxs_newexception(c"Expected this".as_ptr());
        }
        let this = unsafe{Self::from_borrow_void(this)};
        let other = pxs_gettype(pxs_getrt(args), pxs_arg(args, 1), 0);
        if other.is_null() {
            return pxs_newexception(c"Expected this".as_ptr());
        }
        let other = unsafe{Self::from_borrow_void(other)};
        
        let arena = pxs_newarena();
        let new_args = pxs_arenaput(arena, pxs_newlist());

        pxs_listadd(new_args, pxs_newcopy(pxs_getrt(args)));
        pxs_listadd(new_args, pxs_newint(this.x + other.x));
        pxs_listadd(new_args, pxs_newint(this.y + other.y));

        let res = unsafe{Self::new(new_args)};
        pxs_freearena(arena);
        res
    }

    unsafe extern "C" fn new(args: pxs_VarT) -> pxs_VarT {
        let x = pxs_arg(args, 0);
        if !pxs_varis(x, pxs_VarType::pxs_Int64) {
            return pxs_newexception(c"Expected arg 1 x:int".as_ptr());
        }
        let y = pxs_arg(args, 1);
        if !pxs_varis(y, pxs_VarType::pxs_Int64) {
            return pxs_newexception(c"Expected arg 2 y:int".as_ptr());
        }

        let v2 = Vector2{
            x: pxs_getint(x),
            y: pxs_getint(y)
        };

        // Object it
        let obj = pxs_newtype(v2.into_void(), Vector2::free, c"Vector2".as_ptr(), 0);
        pxs_object_addfunc(obj, c"add".as_ptr(), Self::add);

        // Host it
        pxs_newhost(obj)
    }
}

#[divan::bench]
fn bench_py() {
    pxs_freevar(pxs_exec(pxs_Runtime::pxs_Python, c"from pxs import Vector2\nv1 = Vector2(0,1)\nv2 = Vector2(1,0)\nv3 = v1.add(v2)".as_ptr(), c"<bench>".as_ptr()));
}