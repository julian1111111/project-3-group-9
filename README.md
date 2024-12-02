# Project 3 - FAT32 File System 

This project involves developing a user-space shell utility that interprets and manipulates FAT32 file system images. The utility allows users to mount an image file and interact with it through a set of commands similar to those in a typical shell environment. Users can navigate directories, list contents, create and delete files and directories, open and close files, read from and write to files, and manage file offsets - all within the FAT32 image. The program emphasizes robust error handling and maintains the integrity of the file system image throughout its operations, ensuring it remains uncorrupted even when faced with erroneous commands.

## Group Members
- **Julian Schumacher**: jgs21h@fsu.edu
- **Nicholas Miller**: nrm21e@fsu.edu
- **Juan Dangon**: jmd21@fsu.edu

### Division of Labor

### Part 1: Mounting File Image
- **Responsibilities**: 
- **Assigned to**: Julian Schumacher

### Part 2: Navigation 
- **Responsibilities**: 
- **Assigned to**: Julian Schumacher

### Part 3: Create 
- **Responsibilities**: 
- **Assigned to**: Nicholas Miller

### Part 4: Read 
- **Responsibilities**: 
- **Assigned to**: Nicholas Miller

### Part 5: Update
- **Responsibilities**: 
- **Assigned to**: Juan Dangon

### Part 6: Delete
- **Responsibilities**: 
- **Assigned to**: Juan Dangon

## File Listing
```shell
.gitignore
src/
	commands.rs
	fat32.rs
	main.rs
	open_files.rs
	shell.rs
tests/
	test_basic.txt
	test_creation.txt
	test_deletion.txt
	test_errors.txt
	test_file_ops.txt
	test_rename.txt
Cargo.lock
cargo.toml
Makefile
README.md
```

## How to Compile and Execute

### Requirements
- **Compiler**: `rustc`

### Compilation and Execution
```shell
make
```
This will build an executable called `filesys` in the `/target/release/` directory. This command also executes the program, passing `fat32.img` as a parameter automatically. This assumes that the `fat32.img` file is already present in the root directory (in the same directory as the Makefile)