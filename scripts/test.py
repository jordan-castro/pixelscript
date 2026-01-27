# Script for runnig /tests except for test_repl

import os
import sys


# Get skipped tests
skipped_tests = sys.argv[1:]
skipped_tests.append("test_repl.rs")


tests = os.listdir("tests")
for t in tests:
    if t in skipped_tests:
        continue
    
    # Get line 9
    line = ""
    with open(f"tests/{t}", 'r') as f:
        line = f.readlines()[8]

    if not line.startswith("//"):
        continue
    
    # Get cmd
    command = line.split("// ")[-1]
    # for i in range(len(command)):
        # if command[i].startswith('"'):
            # command[i] = command[i][1:-1]
    print(command)
    os.system(command)
    # subprocess.call(command)