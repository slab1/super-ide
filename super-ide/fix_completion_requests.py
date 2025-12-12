#!/usr/bin/env python3
"""
Fix CompletionRequest missing fields by adding default values
"""
import re
import sys

def fix_completion_requests(file_path):
    with open(file_path, 'r') as f:
        content = f.read()
    
    # Pattern to find CompletionRequest initializations missing cursor_position and text_before_cursor
    pattern = r'(let ai_request = |let completion_request = |let request = )crate::ai::CompletionRequest \{([^}]*)\};'
    
    def add_missing_fields(match):
        prefix = match.group(1)
        body = match.group(2)
        
        # Check if cursor_position and text_before_cursor are missing
        if 'cursor_position:' not in body and 'text_before_cursor:' not in body:
            # Add the missing fields
            new_body = body.rstrip() + ',\n            cursor_position: None,\n            text_before_cursor: String::new(),'
            return f'{prefix}crate::ai::CompletionRequest {{\n{new_body}\n            }};'
        elif 'cursor_position:' not in body:
            new_body = body.rstrip() + ',\n            cursor_position: None,'
            return f'{prefix}crate::ai::CompletionRequest {{\n{new_body}\n            }};'
        elif 'text_before_cursor:' not in body:
            new_body = body.rstrip() + ',\n            text_before_cursor: String::new(),'
            return f'{prefix}crate::ai::CompletionRequest {{\n{new_body}\n            }};'
        else:
            return match.group(0)
    
    fixed_content = re.sub(pattern, add_missing_fields, content, flags=re.MULTILINE | re.DOTALL)
    
    with open(file_path, 'w') as f:
        f.write(fixed_content)
    
    print(f"Fixed {file_path}")

if __name__ == "__main__":
    for file_path in sys.argv[1:]:
        fix_completion_requests(file_path)