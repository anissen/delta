#!/usr/bin/env python3
"""
Script to create snapshot test files from workbench.∆ sections.
This script will:
1. Read the workbench.∆ file
2. Split it into sections at "# =======" lines
3. For each section, interactively ask for directory and filename
4. Create the snapshot test TOML file with the cleaned content
"""

import os
from pathlib import Path

def read_workbench_file(file_path):
    """Read the workbench file and return its content."""
    with open(file_path, 'r', encoding='utf-8') as f:
        return f.read()

def split_into_sections_with_delimiters(content):
    """Split content into sections based on '# =======' separators, preserving delimiters."""
    sections = []
    lines = content.split('\n')
    current_section = []
    current_delimiter = None

    for line in lines:
        if line.strip() == '# =======':
            if current_section:
                sections.append({
                    'content': '\n'.join(current_section),
                    'delimiter': current_delimiter
                })
                current_section = []
            current_delimiter = line
        else:
            current_section.append(line)

    # Add the last section if it exists
    if current_section:
        sections.append({
            'content': '\n'.join(current_section),
            'delimiter': current_delimiter
        })

    return sections

def write_workbench_file(file_path, content):
    """Write content back to the workbench file."""
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(content)

def remove_section_from_workbench(workbench_path, section_index=0):
    """Remove the first section and its delimiter from the workbench file."""
    # Read current content
    with open(workbench_path, 'r', encoding='utf-8') as f:
        content = f.read()

    # Split into sections again
    sections = split_into_sections_with_delimiters(content)

    if not sections or section_index >= len(sections):
        print("No section to remove")
        return

    # Rebuild content without the first section
    remaining_sections = sections[1:]

    if not remaining_sections:
        # If no sections remain, write empty content
        updated_content = ""
    else:
        # Rebuild the content from remaining sections
        updated_content = ""
        for section in remaining_sections:
            if section['delimiter']:
                updated_content += section['delimiter'] + "\n"
            updated_content += section['content']
            if not section['content'].endswith('\n'):
                updated_content += "\n"

    # Clean up excessive newlines
    while '\n\n\n' in updated_content:
        updated_content = updated_content.replace('\n\n\n', '\n\n')

    # Write back to file
    with open(workbench_path, 'w', encoding='utf-8') as f:
        f.write(updated_content)

    print(f"Removed processed section from {workbench_path}")

def clean_section(section):
    """Remove leading '#' and space from each line in the section."""
    lines = section.split('\n')
    cleaned_lines = []

    for line in lines:
        # Skip empty lines
        if not line.strip():
            continue

        # Remove leading '# ' or just '#' from commented lines
        if line.startswith('# '):
            cleaned_lines.append(line[2:])  # Remove '# '
        elif line.startswith('#'):
            cleaned_lines.append(line[1:])  # Remove '#'
        else:
            # Keep non-commented lines as-is
            cleaned_lines.append(line)

    return '\n'.join(cleaned_lines)

def create_toml_file(filepath, script_content):
    """Create a TOML file with the given script content."""
    # Create directory if it doesn't exist

    # Create the full file path
    actual_file_path = Path("snapshots/tests/" + filepath + ".toml")
    directory_path = os.path.dirname(actual_file_path)
    Path(directory_path).mkdir(parents=True, exist_ok=True)

    # Create TOML content
    toml_content = f"""script = '''
{script_content}
'''
"""

    # Write the file
    with open(actual_file_path, 'w', encoding='utf-8') as f:
        f.write(toml_content)

    print(f"Created: {actual_file_path}")

def main():
    # Read the workbench file
    workbench_path = "examples/workbench.∆"

    if not os.path.exists(workbench_path):
        print(f"Error: {workbench_path} not found!")
        return

    print(f"Reading {workbench_path}...")

    # Process sections iteratively, removing each after processing
    section_count = 1
    while True:
        # Re-read the workbench file each time to get updated content
        content = read_workbench_file(workbench_path)

        # Split into sections
        sections = split_into_sections_with_delimiters(content)

        # If no sections left, we're done
        if not sections:
            print("\nNo more sections to process.")
            break

        # Process the first section
        section = sections[0]
        cleaned_content = clean_section(section['content'])

        # Skip empty sections
        if not cleaned_content.strip():
            print(f"\nSkipping empty section {section_count}")
            # Remove the empty section from workbench
            remove_section_from_workbench(workbench_path, 0)
            section_count += 1
            continue

        print(f"\n--- Section {section_count} ---")
        print(cleaned_content)
        print("-" * 50)

        # Ask user for directory and filename
        while True:
            user_input = input(f"Enter directory path and file name for section {section_count} (or 'q' to quit): ").strip()
            if user_input.lower() == 'q':
                print("Exiting...")
                return
            if user_input:
                filepath = user_input
                break
            print("File path cannot be empty!")

        # Create the file
        try:
            create_toml_file(filepath, cleaned_content)

            # Remove the processed section from workbench
            remove_section_from_workbench(workbench_path, 0)

            print(f"Successfully processed section {section_count}")
            section_count += 1

        except Exception as e:
            print(f"Error creating file: {e}")
            continue

    print("\nDone!")

if __name__ == "__main__":
    main()
