// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//

// cargo test --test <test goes here> --no-default-features --features "lua,python,pxs-debug,testing" -- --nocapture --test-threads=1

#[cfg(test)]
#[allow(unused)]
mod tests {
    use pixelscript::{create_raw_string, free_raw_string, pxs_finalize, pxs_initialize, pxs_newmod, shared::{module::pxs_Module, utils}};
    
    fn print_helper(lang: &str) {
        println!("====================== {lang} ===================");
    }

    fn test_python() {
    }
    fn test_lua() {}

    #[test]
    fn run_test() {
        pxs_initialize();
        utils::setup_pxs();

        print_helper("PYTHON");
        test_python();
        print_helper("LUA");
        test_lua();

        pxs_finalize();
    }
}
