# Script for runnig /tests except for test_repl

import os
import sys


# Get skipped tests
included_tests = sys.argv[1:]
skip_tests = ["test_repl.rs"]


if len(included_tests) == 0:
    included_tests = os.listdir("tests")

for t in included_tests:
    if t in skip_tests:
        continue
    
    # Get line 9
    line = ""
    with open(f"tests/{t}", 'r') as f:
        line = f.readlines()[8]

    if not line.startswith("//"):
        continue
    
    # Get cmd
    command = line.split("// ")[-1]
    print(command)
    os.system(command)
