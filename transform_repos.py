#!/usr/bin/env python3
"""Transform repositories.rs to wrap pub async fn bodies in tokio::task::spawn_blocking."""

import re
import sys

def find_matching_brace(text, start):
    """Find matching closing brace starting from 'start' (which should be at an opening brace)."""
    depth = 0
    i = start
    in_string = False
    string_char = None
    in_line_comment = False
    in_block_comment = False
    
    while i < len(text):
        c = text[i]
        
        if in_line_comment:
            if c == '\n':
                in_line_comment = False
            i += 1
            continue
        
        if in_block_comment:
            if text[i:i+2] == '*/':
                in_block_comment = False
                i += 2
                continue
            i += 1
            continue
        
        if in_string:
            if c == '\\' and string_char != '\'':
                i += 2  # skip escaped char
                continue
            if c == string_char:
                in_string = False
            i += 1
            continue
        
        if text[i:i+2] == '//':
            in_line_comment = True
            i += 2
            continue
        
        if text[i:i+2] == '/*':
            in_block_comment = True
            i += 2
            continue
        
        if c in ('"', '\''):
            # check for raw string r#"..."#
            if c == 'r' and i+1 < len(text) and text[i+1] == '#':
                # raw string - just skip for brace counting
                i += 1
                continue
            in_string = True
            string_char = c
            i += 1
            continue
        
        if c == '{':
            depth += 1
        elif c == '}':
            depth -= 1
            if depth == 0:
                return i
        
        i += 1
    
    return -1

def get_method_return_type(signature):
    """Extract the return type from a method signature."""
    # Find -> in the signature
    arrow_idx = signature.rfind('->')
    if arrow_idx == -1:
        return None
    ret = signature[arrow_idx+2:].strip()
    # Remove trailing { and whitespace
    ret = ret.rstrip('{').strip()
    return ret

def transform_file(content):
    """Transform the entire file content."""
    
    # Step 1: Add DbError type alias after "use super::models::*;"
    content = content.replace(
        'use super::models::*;\n',
        'use super::models::*;\n\ntype DbError = Box<dyn std::error::Error + Send + Sync>;\n',
        1
    )
    
    return content

# Read the file
with open('/home/runner/work/simple-blog/simple-blog/src/db/repositories.rs', 'r') as f:
    original = f.read()

# Apply transformation
result = transform_file(original)

# Write result
with open('/home/runner/work/simple-blog/simple-blog/src/db/repositories.rs', 'w') as f:
    f.write(result)

print("Done: added DbError type alias")
