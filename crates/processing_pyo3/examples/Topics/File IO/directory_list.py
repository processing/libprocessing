# Listing files in directories and subdirectories
# by Daniel Shiffman.
#
# This example has three functions:
# 1) List the names of files in a directory
# 2) List the names along with metadata (size, last modified)
#    of files in a directory
# 3) List the names along with metadata (size, last modified)
#    of files in a directory and all subdirectories (using recursion)
from mewnala import *
import os
import time


def setup():
    # Using just the path of this sketch to demonstrate,
    # but you can list any directory you like.
    path = os.path.dirname(os.path.abspath(__file__))

    print("Listing all filenames in a directory: ")
    filenames = list_file_names(path)
    print(filenames)

    print("\nListing info about all files in a directory: ")
    files = list_files(path)
    for f in files:
        print("Name: " + os.path.basename(f))
        print("Is directory: " + str(os.path.isdir(f)))
        print("Size: " + str(os.path.getsize(f)))
        last_modified = time.ctime(os.path.getmtime(f))
        print("Last Modified: " + last_modified)
        print("-----------------------")

    print("\nListing info about all files in a directory and all subdirectories: ")
    all_files = list_files_recursive(path)

    for f in all_files:
        print("Name: " + os.path.basename(f))
        print("Full path: " + os.path.abspath(f))
        print("Is directory: " + str(os.path.isdir(f)))
        print("Size: " + str(os.path.getsize(f)))
        last_modified = time.ctime(os.path.getmtime(f))
        print("Last Modified: " + last_modified)
        print("-----------------------")

    no_loop()


# Nothing is drawn in this program and the draw() doesn't loop because
# of the no_loop() in setup()
def draw():
    pass


# This function returns all the files in a directory as a list of names
def list_file_names(d):
    if os.path.isdir(d):
        names = os.listdir(d)
        return names
    else:
        # If it's not a directory
        return None


# This function returns all the files in a directory as a list of full paths
# This is useful if you want more info about the file
def list_files(d):
    if os.path.isdir(d):
        files = []
        for name in os.listdir(d):
            files.append(os.path.join(d, name))
        return files
    else:
        # If it's not a directory
        return None


# Function to get a list of all files in a directory and all subdirectories
def list_files_recursive(d):
    file_list = []
    recurse_dir(file_list, d)
    return file_list


# Recursive function to traverse subdirectories
def recurse_dir(a, d):
    if os.path.isdir(d):
        # If you want to include directories in the list
        a.append(d)
        subfiles = list_files(d)
        for i in range(len(subfiles)):
            # Call this function on all files in this directory
            recurse_dir(a, os.path.abspath(subfiles[i]))
    else:
        a.append(d)


run()
