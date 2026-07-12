// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
// cargo test --test test_bytes --no-default-features --features "lua,python,js,testing" -- --nocapture --test-threads=1

#[cfg(test)]
#[allow(unused)]
mod tests {
    use pixelscript::{
        pxs_copybytes, pxs_copystring, pxs_finalize, pxs_freearena, pxs_freevar, pxs_initialize, pxs_newarena, pxs_newbytes, pxs_newmod, pxs_newstring, pxs_varsize, shared::{module::pxs_Module, pxs_Opaque, pxs_Runtime, utils, var::pxs_VarT},
    };
    use etffi::{cstring::CStringSafe, borrow_string, create_raw_string, free_raw_string, own_string, ptr_magic::PtrMagic};

    fn print_helper(lang: &str) {
        println!("====================== {lang} ===================");
    }

    #[test]
    fn run_test() {
        println!();
        pxs_initialize();

        let mut data: [i32; 7] = [1,2,3,4,5,6,7];

        // Create new bytes
        let bytes = pxs_newbytes(data.as_mut_ptr() as pxs_Opaque, size_of::<i32>(), 7);

        assert!(pxs_varsize(bytes) == size_of_val(&data), "Sizes dont match");

        // Convert bytes back into data
        let mut copy_data: [i32; 7] = [0;7];
        pxs_copybytes(bytes, copy_data.as_mut_ptr() as pxs_Opaque);
        pxs_freevar(bytes);

        assert!(data == copy_data, "Data does not match.");

        // Read the bytes of a string
        let string = pxs_newstring(c"test".as_ptr());
        let size = pxs_varsize(string);
        let mut string_bytes = vec![0; size];

        pxs_copystring(string, string_bytes.as_mut_ptr() as *mut core::ffi::c_char);
        pxs_freevar(string);

        let rstring = String::from_utf8(string_bytes).expect("Could not extract string");
        assert!(rstring == "test", "Strings do not match.");

        pxs_finalize();
    }
}
