// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use std::{collections::HashMap, os::raw::c_void, ptr, sync::{Arc, Mutex, OnceLock}};

use crate::shared::{
    PtrMagic,
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
/// pixelscript_add_object("Person", new_person);
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
/// This is why a Objects are more like Pseudo types than actual class/objects.
pub struct PixelObject {
    /// Type name (this is a hash)
    pub type_name: String,
    /// The Host pointer
    pub ptr: *mut c_void,
    /// The language object pointer.
    pub lang_ptr: Mutex<*mut c_void>,
    /// Should the lang_ptr be freed by PixelScript?
    pub free_lang_ptr: Mutex<bool>,
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
    pub fn new(ptr: *mut c_void, free_method: FreeMethod, type_name: &str) -> Self {
        Self {
            ptr,
            free_method,
            callbacks: vec![],
            lang_ptr: Mutex::new(ptr::null_mut()),
            type_name: type_name.to_string(),
            free_lang_ptr: Mutex::new(true),
        }
    }

    pub fn add_callback(&mut self, name: &str, full_name: &str, idx: i32) {
        self.callbacks.push(ModuleCallback {
            name: name.to_string(),
            full_name: full_name.to_string(),
            idx
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

    pub fn update_free_lang_ptr(&self, val: bool) {
        let mut guard = self.free_lang_ptr.lock().unwrap();

        *guard = val;
    }
}

impl PtrMagic for PixelObject {}
unsafe impl Send for PixelObject {}
unsafe impl Sync for PixelObject {}
impl Drop for PixelObject {
    fn drop(&mut self) {
        let mut lang_ptr = self.lang_ptr.lock().unwrap();
        if *self.free_lang_ptr.lock().unwrap() {
            if !lang_ptr.is_null() {
                //  Free Language memory
                let _ = unsafe { Box::from_raw(*lang_ptr) };
            }
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

/// The object lookup!
static OBJECT_LOOKUP: OnceLock<Mutex<ObjectLookup>> = OnceLock::new();

/// Get the Object lookup global state. Shared between all runtimes.
fn get_object_lookup() -> std::sync::MutexGuard<'static, ObjectLookup> {
    OBJECT_LOOKUP.get_or_init(|| {
        Mutex::new(ObjectLookup {
            object_hash: HashMap::new(),
        })
    })
    .lock()
    .unwrap()
}

pub(crate) fn clear_object_lookup() {
    let mut lookup = get_object_lookup();
    lookup.object_hash.clear();
}

// add_object(Arc::clone(&pixel_arc))
pub(crate) fn lookup_add_object(pixel_obj: Arc<PixelObject>) -> i32 {
    let mut lookup = get_object_lookup();

    let idx = lookup.object_hash.len();

    lookup.object_hash.insert(idx as i32, Arc::clone(&pixel_obj));

    idx as i32
}

/// Get a PixelObject Arc
pub(crate) fn get_object(idx: i32) -> Option<Arc<PixelObject>> {
    let lookup = get_object_lookup();

    if let Some(res) = lookup.object_hash.get(&idx) {
        Some(res.to_owned())
    } else {
        None
    }
}