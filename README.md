# Project 3 - FAT32 File System 

This project involves developing a user-space shell utility that interprets and manipulates FAT32 file system images. The utility allows users to mount an image file and interact with it through a set of commands similar to those in a typical shell environment. Users can navigate directories, list contents, create and delete files and directories, open and close files, read from and write to files, and manage file offsets - all within the FAT32 image. The program emphasizes robust error handling and maintains the integrity of the file system image throughout its operations, ensuring it remains uncorrupted even when faced with erroneous commands.

## Group Members
- **Julian Schumacher**: jgs21h@fsu.edu
- **Nicholas Miller**: nrm21e@fsu.edu
- **Juan Dangon**: jmd21@fsu.edu

## Division of Labor

### Part 1: Mounting File Image
- **Responsibilities**: 
	- Parse command-line arguments to mount the FAT32 image file.
	- Read and interpret the FAT32 file system structure for navigation.
	- Handle errors if the image file does not exist.
	- Initialize the shell environment with the correct prompt displaying the image name and current path.
	- Implement the `info` command to parse and display boot sector information (e.g., bytes per sector, sectors per cluster, size of image).
	- Implement the `exit` command to safely close the program and free allocated resources.
- **Assigned to**: Julian Schumacher

### Part 2: Navigation 
- **Responsibilities**: 
	- Maintain the current working directory state within the FAT32 image.
	- Implement the `cd [DIRNAME]` command to change directories, with error handling for invalid or non-directory targets.
	- Implement the `ls` command to list directory contents, including special entries like `"."` and `".."`.
- **Assigned to**: Julian Schumacher

### Part 3: Create 
- **Responsibilities**: 
	- Implement the `mkdir [DIRNAME]` command to create new directories in the current working directory, ensuring no name conflicts.
	- Implement the `creat [FILENAME]` command to create new zero-byte files, handling errors if a file or directory with the same name exists.
- **Assigned to**: Nicholas Miller

### Part 4: Read 
- **Responsibilities**: 
	- Implement the `open [FILENAME] [FLAGS]` command to open files with specified access modes (`-r`, `-w`, `-rw`, `-wr`), maintaining a data structure for opened files.
	- Implement the `close [FILENAME]` command to close files and update the open files data structure.
	- Implement the `lsof` command to list all opened files along with their index, name, mode, offset, and path.
	- Implement the `lseek [FILENAME] [OFFSET]` command to set the file's offset for reading or writing, with error handling for invalid offsets.
	- Implement the `read [FILENAME] [SIZE]` command to read data from a file starting at the current offset, updating the offset after reading, and handling end-of-file conditions.
- **Assigned to**: Nicholas Miller

### Part 5: Update
- **Responsibilities**: 
	- Implement the `write [FILENAME] "[STRING]"` command to write data to a file starting at the current offset, extending the file size if necessary, and updating the offset after writing.
	- Implement the `rename [FILENAME] [NEW_FILENAME]` command to rename files or directories, ensuring the target name does not exist and special directories like `"."` or `".."` are not renamed.
- **Assigned to**: Juan Dangon

### Part 6: Delete
- **Responsibilities**: 
	- Implement the `rm [FILENAME]` command to delete files from the current working directory, removing directory entries and reclaiming data, with checks to prevent deletion of directories or opened files.
	- Implement the `rmdir [DIRNAME]` command to remove empty directories, ensuring that the directory is empty (excluding `"."` and `".."`), and handling errors if files within are opened or if the directory does not exist.
- **Assigned to**: Juan Dangon

## File Listing
```shell
├── .gitignore
├── Cargo.lock
├── cargo.toml
├── Makefile
├── README.md
├── src
│   ├── commands.rs
│   ├── fat32.rs
│   ├── main.rs
│   ├── open_files.rs
│   └── shell.rs
└── tests
    ├── test_basic.txt
    ├── test_creation.txt
    ├── test_deletion.txt
    ├── test_errors.txt
    ├── test_file_ops.txt
    └── test_rename.txt
```

## How to Compile and Execute

### Requirements
- **Compiler**: `rustc`

### Compilation and Execution
```shell
make
```
This will build an executable called `filesys` in the `/target/release/` directory. This command also executes the program, passing `fat32.img` as a parameter automatically. This assumes that the `fat32.img` file is already present in the root directory (in the same directory as the Makefile)

## Bugs


## Extra Credit
