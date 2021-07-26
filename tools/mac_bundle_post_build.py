#!/usr/bin/env python3

"""
Post build script to run with the path to the bundle as argument.

Performed steps:
    - Strip off debug symbols from executable
    - Copy external frameworks and libraries to the bundle and configure the paths to them
"""

import sys
import os
import subprocess
import shutil


# System library locations, libs from these locations are not copied to bundle
EXCLUDE_LIB_DIRS = ['/System/Library', '/usr/lib']


def get_exec_file(macos_dir):
    """Return the executable file of the bundle"""
    for item in os.listdir(macos_dir):
        file_ = os.path.join(macos_dir, item)
        executable = os.access(file_, os.X_OK)
        if os.path.isfile(file_) and executable:
            return file_


def strip_symbols(exec_file):
    """Strip debug symbols"""
    print(f"Stripping debug symbols on {exec_file}")
    subprocess.call(["strip", exec_file])


def get_used_libs(file_):
    """Return a list of libraries used by given application or dylib"""
    libs_output = subprocess.check_output(
        ['otool', '-L', file_],
        universal_newlines=True
    )

    libs = []

    for line in libs_output.splitlines()[1:]:
        lib_file = line.split(' ')[0].strip()
        for exclude_dir in EXCLUDE_LIB_DIRS:
            if lib_file.startswith(exclude_dir):
                lib_file = None
                break
        if lib_file:
            libs.append(lib_file)

    return libs


def create_dmg(bundle_dir):
    """Create a DMG file with the bundle"""
    name, ext = os.path.splitext(os.path.basename(bundle_dir))
    dmg_file = os.path.join(os.path.dirname(bundle_dir), name + '.dmg')
    print(f"Creating DMG file {dmg_file}")
    if os.path.exists(dmg_file):
        os.remove(dmg_file)
    subprocess.call(['hdiutil', 'create', '-fs', 'HFS+',
                    '-volname', name, '-srcfolder', bundle_dir, dmg_file])


def main():
    if len(sys.argv) < 2:
        sys.exit("No input argument given.")

    bundle_dir = sys.argv[1]

    if not os.path.exists(bundle_dir):
        sys.exit(f"Bundle {bundle_dir} does not exist")

    macos_dir = os.path.join(bundle_dir, "Contents", "MacOS")
    frameworks_dir = os.path.join(bundle_dir, "Contents", "Frameworks")
    exec_file = get_exec_file(macos_dir)

    if exec_file is None:
        sys.exit("Executable file not found")

    strip_symbols(exec_file)

    libs = get_used_libs(exec_file)

    if libs:
        os.makedirs(frameworks_dir)

        for lib in libs:
            orig_lib_file = os.path.realpath(lib)
            lib_file = os.path.join(frameworks_dir,
                                    os.path.basename(orig_lib_file))

            print(f"Copying library {orig_lib_file} to bundle.")
            shutil.copy(orig_lib_file, frameworks_dir)
            os.chmod(lib_file, 0o755)

            new_lib_path = os.path.join('@executable_path',
                                        os.path.relpath(lib_file, macos_dir))
            subprocess.call(['install_name_tool', '-id',
                            new_lib_path, lib_file])
            subprocess.call(['install_name_tool', '-change', lib,
                            new_lib_path, exec_file])

            lib_deps = get_used_libs(lib_file)

            for lib_dep in lib_deps:
                lib_dep_file = os.path.realpath(lib_dep)
                lib_dep_file = os.path.join(frameworks_dir,
                                            os.path.basename(lib_dep_file))
                new_lib_path = os.path.join('@executable_path',
                                            os.path.relpath(lib_dep_file, macos_dir))
                subprocess.call(['install_name_tool', '-change', lib_dep,
                                new_lib_path, lib_file])

    create_dmg(bundle_dir)


if __name__ == '__main__':
    main()
