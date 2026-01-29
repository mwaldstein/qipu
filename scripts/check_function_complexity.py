#!/usr/bin/env python3
import re
import sys
from pathlib import Path

# Functions grandfathered in (already >100 lines when check was added)
ALLOWED_FUNCTIONS = {
    "commands/compact/report.rs:execute",
    "commands/compact/show.rs:execute",
    "commands/compact/suggest.rs:execute",
    "commands/compact/status.rs:execute",
    "commands/export/emit/bibliography.rs:export_bibtex",
    "commands/load/deserialize.rs:looks_like_json",
}

def find_large_functions(src_dir, max_lines=100):
    violations = []
    
    for rs_file in Path(src_dir).rglob("*.rs"):
        with open(rs_file) as f:
            content = f.read()
            lines = content.split('\n')
        
        # Find all function definitions with line numbers
        fn_pattern = re.compile(r'^(?P<indent>\s*)(?P<mods>(pub\s+)?(async\s+)?(unsafe\s+)?)fn\s+(?P<name>\w+)')
        
        for line_num, line in enumerate(lines, 1):
            match = fn_pattern.match(line)
            if match:
                fn_name = match.group('name')
                fn_start = line_num
                
                # Find matching closing brace
                brace_level = 0
                fn_lines = 0
                
                for i in range(fn_start - 1, len(lines)):
                    fn_lines = i - fn_start + 1
                    
                    open_braces = lines[i].count('{')
                    close_braces = lines[i].count('}')
                    brace_level += open_braces - close_braces
                    
                    if brace_level == 0 and i > fn_start - 1:
                        break
                
                if fn_lines > max_lines:
                    rel_path = str(rs_file.relative_to(src_dir))
                    violations.append((rel_path, fn_name, fn_start, fn_lines))
    
    return violations

if __name__ == '__main__':
    violations = find_large_functions('src', 100) + find_large_functions('crates', 100)
    
    violation_found = False
    for file, fn, line_num, lines in violations:
        key = f"{file}:{fn}"
        if key not in ALLOWED_FUNCTIONS:
            print(f"ERROR: {file}:{fn} (line {line_num}) has {lines} lines (>100)")
            violation_found = True
    
    if violation_found:
        print()
        print("Functions exceeding 100 lines trigger CI failure. Options:")
        print("  - Refactor to reduce complexity")
        print("  - Request exception in beads")
        sys.exit(1)
