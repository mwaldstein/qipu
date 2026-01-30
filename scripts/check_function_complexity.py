#!/usr/bin/env python3
import re
import sys
from pathlib import Path

# Functions grandfathered in (already >100 lines when check was added)
ALLOWED_FUNCTIONS = set()

def count_braces_outside_strings(line):
    """Count braces only outside string/char literals."""
    count_open = 0
    count_close = 0
    in_string = False
    in_char = False
    escape_next = False
    
    for i, ch in enumerate(line):
        if escape_next:
            escape_next = False
            continue
        
        if ch == '\\':
            escape_next = True
            continue
        
        if in_string:
            if ch == '"':
                in_string = False
            continue
        
        if in_char:
            if ch == "'":
                in_char = False
            continue
        
        if ch == '"':
            in_string = True
        elif ch == "'":
            in_char = True
        elif ch == '{':
            count_open += 1
        elif ch == '}':
            count_close += 1
    
    return count_open, count_close

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
                    
                    open_braces, close_braces = count_braces_outside_strings(lines[i])
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
