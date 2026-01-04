use std::{collections::HashMap, os::raw::c_void, ptr, sync::{Arc, Mutex, OnceLock}};

use crate::shared::{
    PtrMagic,
    func::{Func, Function},
    module::ModuleCallback,
};

pub type FreeMethod = unsafe extern "C" fn(ptr: *mut c_void);

/// A PixelScript Object.
///
/// The way this works is via the host, a Pseudo type can be created. So when the scripting
/// language interacts with the object, it calls it's pseudo methods.
///
/// example:
/// ```c
/// struct Person {
///     const char* name;
///     int age;
///
///     Person(const char* name, int age) {
///         this->name = name;
///         this->age = age;
///     }
///
///     void set_name(const char* name) {
///         this->name = name;
///     }
///
///     void set_age(int age) {
///         this->age = age;
///     }
///
///     int get_age() {
///         return this->age;
///     }
///
///     const char* get_name() {
///         return this->name;
///     }
/// };
///
/// void free_person(void* p) {
///     // TODO
/// }
/// Var* person_set_name(int argc, Var** argv, void* opaque) {
///     Var* object = argv[0];
///     Person* p = object.value.object_val as Person;
///     Var* name = argv[1];
///     p->set_name(name.value.string_val);
///     return NULL;
/// }
/// Var* new_person(int argc, Var** argv, void* opaque) {
///     Person* p = malloc();
///     PixelObject* object_ptr = pixelscript_new_object(p, free_person);
///     pixelscript_object_add_callback(object_ptr, "set_name", person_set_name);
///     return pixelscript_var_object(object_ptr);
/// }
///
/// // OOP base
/// PixelClass* person_class = pixelscript_new_class("Person");
/// pixelscript_class_add_object(person_class, new_person);
/// pixelscript_add_class(person_class);
///
/// // Or functional
/// pixelscript_add_callback("new_person", new_person);
/// ```
/// 
/// In a JS example:
/// ```js
/// let p = new Person("Jordan");
/// p.set_name("James"); 
/// ```
///
/// So first you add a Class with a constructor. Then within the constructor you return the object.
/// This is why a Class/Object are more like Pseudo types than actual class/objects.
pub struct PixelObject {
    /// The Host pointer
    pub ptr: *mut c_void,
    /// The language object pointer.
    pub lang_ptr: Mutex<*mut c_void>,
    /// The Method for freeing
    pub free_method: FreeMethod,
    /// Callbacks with names.
    ///
    /// Important to note that a PixelObject can not have `static` callbacks.
    ///
    /// The first Var will always be the ptr.
    pub callbacks: Vec<ModuleCallback>,
    // PixelObject does not hold variables. They are all getters/setters
}

impl PixelObject {
    pub fn new(ptr: *mut c_void, free_method: FreeMethod) -> Self {
        Self {
            ptr,
            free_method,
            callbacks: vec![],
            lang_ptr: Mutex::new(ptr::null_mut())
        }
    }

    pub fn add_callback(&mut self, name: &str, callback: Func, opaque: *mut c_void) {
        self.callbacks.push(ModuleCallback {
            name: name.to_owned(),
            func: Function { 
                func: callback, 
                opaque
            },
        });
    }

    pub fn update_lang_ptr(&self, n_ptr: *mut c_void) {
        let mut guard = self.lang_ptr.lock().unwrap();

        if !guard.is_null() {
            eprintln!("Can not mutate if ptr is already set.");
            return;
        }

        *guard = n_ptr;
    }
}

impl PtrMagic for PixelObject {}
unsafe impl Send for PixelObject {}
unsafe impl Sync for PixelObject {}
impl Drop for PixelObject {
    fn drop(&mut self) {
        let mut lang_ptr = self.lang_ptr.lock().unwrap();
        if !lang_ptr.is_null() {
            //  Free Language memory
            let _ = unsafe { Box::from_raw(*lang_ptr) };
        }
        *lang_ptr = ptr::null_mut();
        // Free host memory
        unsafe {
            (self.free_method)(self.ptr);
        }
    }
}


/// Lookup state structure
pub struct ObjectLookup {
    /// Object hash shared between all runtimes.
    /// 
    /// Negative numbers are valid here.
    pub object_hash: HashMap<i32, Arc<PixelObject>>
}

impl ObjectLookup {
    pub fn get_object(&self, idx: i32) -> Option<&Arc<PixelObject>> {
        self.object_hash.get(&idx)
    }
    pub fn add_object(&mut self, pixel_object: Arc<PixelObject>) -> i32 {
        // TODO: Allow for negative idxs.
        self.object_hash.insert(self.object_hash.len() as i32, pixel_object);

        return (self.object_hash.len() - 1) as i32;
    }
}

/// The object lookup!
static OBJECT_LOOKUP: OnceLock<Mutex<ObjectLookup>> = OnceLock::new();

/// Get the Object lookup global state. Shared between all runtimes.
pub(crate) fn get_object_lookup() -> std::sync::MutexGuard<'static, ObjectLookup> {
    OBJECT_LOOKUP.get_or_init(|| {
        Mutex::new(ObjectLookup {
            object_hash: HashMap::new(),
        })
    })
    .lock()
    .unwrap()
}