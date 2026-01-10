// Methods to override builtins:

use rustpython::vm::{PyObjectRef, VirtualMachine, scope::Scope};

use crate::{python::get_state, shared::read_file};

/// Add the _pixelscript_load_pymodule function
fn add_pixelscript_load_pymodule(vm: &VirtualMachine) {
    let state = get_state();

    let pyfunc = vm.new_function("_pixelscript_load_pymodule", |name: String, vm: &VirtualMachine| {
        // Add .py just in case.
        let name = if !name.ends_with(".py") {
            format!("{name}.py").to_string()
        } else {
            name
        };
        let contents = read_file(name.as_str());
        if contents.len() == 0 {
            vm.ctx.none()
        } else {
            vm.ctx.new_str(contents).into()
        }
    });
    state.global_scope.set_item("_pixelscript_load_pymodule", pyfunc.into(), vm).expect("Could not set _pixelscript_load_pymodule");
}

/// Makes it possible to import user written modules.
pub(super) fn override_import_loader(vm: &VirtualMachine, scope: PyObjectRef) {
    add_pixelscript_load_pymodule(vm);

    // Run specific code
    let dict = scope.clone().downcast::<rustpython::vm::builtins::PyDict>().expect("Could not downcast to Dict, Python.");
    let scope = Scope::with_builtins(None, dict, vm);

    let code = r#"
import sys

# PixelFinder acts as both Finder and Loader
class PixelFinder:
    @classmethod
    def find_spec(cls, fullname, path=None, target=None):
        # 1. Check if the module exists via your Rust helper
        v_path = fullname.replace('.', '/')
        source = _pixelscript_load_pymodule(v_path)
        if source is not None:
            # We return ourselves as the 'loader'
            # In a bare-bones env, we return a simple object with a loader attr
            class Spec:
                def __init__(self, name, loader):
                    self.name = name
                    self.loader = loader
                    self.submodule_search_locations = [v_path] if name.endswith('__init__') else []
                    self.cached = None  
                    self.has_location = True
                    
            return Spec(fullname, cls)
        return None

    @classmethod
    def create_module(cls, spec):
        # Returning None tells Python to create a standard module object
        return None

    @classmethod
    def exec_module(cls, module):
        # 2. Get the source code again for execution
        source = _pixelscript_load_pymodule(module.__name__.replace('.', '/'))
        
        virtual_path = module.__name__.replace('.', '/') + '.py'

        # 3. Manual compilation and execution
        code = compile(source, virtual_path, 'exec')
        
        # Set basic module attributes manually
        module.__file__ = virtual_path
        
        exec(code, module.__dict__)

# Inject into the start of the search list
sys.meta_path.insert(0, PixelFinder)
"#;

    vm.run_code_string(scope, code, "<pixelscript_import_loader>".to_owned()).expect("Could not add PixelFinde in Python.");
}
