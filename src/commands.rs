use std::fs::File;
use std::io::{self};

use crate::fat32::{DirectoryEntry, FAT32};
use crate::open_files::{FileMode, OpenFile, OpenFiles};

pub fn info(fat32: &FAT32) -> io::Result<()> {
    println!(
        "Position of root cluster (cluster #): {}",
        fat32.boot_sector.root_cluster
    );
    println!("Bytes per sector: {}", fat32.boot_sector.bytes_per_sector);
    println!(
        "Sectors per cluster: {}",
        fat32.boot_sector.sectors_per_cluster
    );
    println!(
        "Total # of clusters in data region: {}",
        fat32.total_clusters
    );
    let num_fat_entries = (fat32.boot_sector.fat_size_32
        * fat32.boot_sector.bytes_per_sector as u32)
        / 4;
    println!("# of entries in one FAT: {}", num_fat_entries);
    let size_of_image = fat32.boot_sector.total_sectors as u64
        * fat32.boot_sector.bytes_per_sector as u64;
    println!("Size of image (in bytes): {}", size_of_image);
    Ok(())
}

pub fn ls(
    image_file: &mut File,
    fat32: &FAT32,
    current_dir_cluster: u32,
) -> io::Result<()> {
    let entries = fat32.read_directory_entries(image_file, current_dir_cluster)?;

    for entry in entries {
        println!("{}", format_name(&entry.name));
    }

    Ok(())
}

pub fn cd(
    image_file: &mut File,
    fat32: &FAT32,
    current_dir_cluster: &mut u32,
    dirname: &str,
) -> io::Result<()> {
    if dirname == "." {
        // Do nothing
        return Ok(());
    } else if dirname == ".." {
        // Navigate to parent directory
        let entries = fat32.read_directory_entries(image_file, *current_dir_cluster)?;
        for entry in entries {
            if format_name(&entry.name) == ".." {
                *current_dir_cluster = entry.first_cluster;
                return Ok(());
            }
        }
        eprintln!("Error: Parent directory not found.");
        return Ok(());
    } else {
        // Find the directory with name DIRNAME
        let entries = fat32.read_directory_entries(image_file, *current_dir_cluster)?;
        for entry in entries {
            if format_name(&entry.name) == dirname && entry.is_directory() {
                *current_dir_cluster = entry.first_cluster;
                return Ok(());
            }
        }
        eprintln!("Error: Directory '{}' not found.", dirname);
        return Ok(());
    }
}

pub fn mkdir(
    image_file: &mut File,
    fat32: &mut FAT32,
    current_dir_cluster: u32,
    dirname: &str,
) -> io::Result<()> {
    // Check if directory already exists
    let entries = fat32.read_directory_entries(image_file, current_dir_cluster)?;
    for entry in entries {
        if format_name(&entry.name) == dirname {
            eprintln!("Error: Directory '{}' already exists.", dirname);
            return Ok(());
        }
    }

    // Create the directory
    fat32.create_directory(image_file, current_dir_cluster, dirname)?;

    println!("Directory '{}' created.", dirname);
    Ok(())
}

pub fn creat(
    image_file: &mut File,
    fat32: &mut FAT32,
    current_dir_cluster: u32,
    filename: &str,
) -> io::Result<()> {
    // Check if file already exists
    let entries = fat32.read_directory_entries(image_file, current_dir_cluster)?;
    for entry in entries {
        if format_name(&entry.name) == filename {
            eprintln!("Error: File '{}' already exists.", filename);
            return Ok(());
        }
    }

    // Create the file
    fat32.create_file(image_file, current_dir_cluster, filename)?;

    println!("File '{}' created.", filename);
    Ok(())
}

pub fn open(
    image_file: &mut File,
    fat32: &mut FAT32,
    current_dir_cluster: u32,
    filename: &str,
    flags: &str,
    open_files: &mut OpenFiles,
) -> io::Result<()> {
    // Check if file is already open
    if open_files.is_file_open(filename) {
        eprintln!("Error: File '{}' is already open.", filename);
        return Ok(());
    }

    // Validate flags
    let mode = match flags {
        "-r" => FileMode::ReadOnly,
        "-w" => FileMode::WriteOnly,
        "-rw" | "-wr" => FileMode::ReadWrite,
        _ => {
            eprintln!("Error: Invalid mode '{}'.", flags);
            return Ok(());
        }
    };

    // Find the file in the directory
    let entries = fat32.read_directory_entries(image_file, current_dir_cluster)?;
    let mut file_entry: Option<DirectoryEntry> = None;
    for entry in entries {
        if format_name(&entry.name) == filename && entry.is_file() {
            file_entry = Some(entry);
            break;
        }
    }

    if let Some(entry) = file_entry {
        let open_file = OpenFile {
            filename: filename.to_string(),
            mode,
            offset: 0,
            first_cluster: entry.first_cluster,
            file_size: entry.file_size,
        };
        open_files.open_file(open_file).map_err(|e| {
            eprintln!("{}", e);
            io::Error::new(io::ErrorKind::Other, e)
        })?;
        println!("File '{}' opened.", filename);
    } else {
        eprintln!("Error: File '{}' does not exist.", filename);
    }

    Ok(())
}

pub fn close(filename: &str, open_files: &mut OpenFiles) -> io::Result<()> {
    open_files.close_file(filename).map_err(|e| {
        eprintln!("{}", e);
        io::Error::new(io::ErrorKind::Other, e)
    })?;
    println!("File '{}' closed.", filename);
    Ok(())
}

pub fn lsof(open_files: &OpenFiles) -> io::Result<()> {
    let open_files_list = open_files.list_open_files();
    if open_files_list.is_empty() {
        println!("No files are open.");
    } else {
        for (index, file) in open_files_list.iter().enumerate() {
            let mode_str = match file.mode {
                FileMode::ReadOnly => "Read Only",
                FileMode::WriteOnly => "Write Only",
                FileMode::ReadWrite => "Read/Write",
            };
            println!(
                "{}: {} Mode: {} Offset: {}",
                index, file.filename, mode_str, file.offset
            );
        }
    }
    Ok(())
}

pub fn size(
    image_file: &mut File,
    fat32: &FAT32,
    current_dir_cluster: u32,
    filename: &str,
) -> io::Result<()> {
    // Find the file in the directory
    let entries = fat32.read_directory_entries(image_file, current_dir_cluster)?;
    for entry in entries {
        if format_name(&entry.name) == filename && entry.is_file() {
            println!("Size of '{}': {} bytes", filename, entry.file_size);
            return Ok(());
        }
    }
    eprintln!("Error: File '{}' does not exist or is a directory.", filename);
    Ok(())
}

pub fn lseek(
    filename: &str,
    offset_str: &str,
    open_files: &mut OpenFiles,
) -> io::Result<()> {
    let offset: u32 = offset_str.parse().unwrap_or_else(|_| {
        eprintln!("Error: Invalid offset '{}'.", offset_str);
        0
    });

    if let Some(open_file) = open_files.get_file_mut(filename) {
        if offset > open_file.file_size {
            eprintln!("Error: Offset exceeds file size.");
            return Ok(());
        }
        open_file.offset = offset;
        println!("Offset of '{}' set to {}.", filename, offset);
    } else {
        eprintln!("Error: File '{}' is not open.", filename);
    }
    Ok(())
}

pub fn read(
    image_file: &mut File,
    fat32: &FAT32,
    filename: &str,
    size_str: &str,
    open_files: &mut OpenFiles,
) -> io::Result<()> {
    let size: u32 = size_str.parse().unwrap_or_else(|_| {
        eprintln!("Error: Invalid size '{}'.", size_str);
        0
    });

    if let Some(open_file) = open_files.get_file_mut(filename) {
        if open_file.mode == FileMode::WriteOnly {
            eprintln!("Error: File '{}' is not open for reading.", filename);
            return Ok(());
        }

        let cluster_chain = fat32.get_cluster_chain(image_file, open_file.first_cluster)?;
        let data = fat32.read_file_data(
            image_file,
            &cluster_chain,
            open_file.offset,
            size,
        )?;

        // Update the offset
        open_file.offset += data.len() as u32;

        // Print the data as a string
        if let Ok(string) = String::from_utf8(data) {
            println!("{}", string);
        } else {
            eprintln!("Error: Failed to read data as string.");
        }
    } else {
        eprintln!("Error: File '{}' is not open.", filename);
    }

    Ok(())
}

pub fn write(
    image_file: &mut File,
    fat32: &mut FAT32,
    current_dir_cluster: u32,
    filename: &str,
    string: &str,
    open_files: &mut OpenFiles,
) -> io::Result<()> {
    // Remove quotes from string
    let string = string.trim_matches('"');
    let data = string.as_bytes();

    if let Some(open_file) = open_files.get_file_mut(filename) {
        if open_file.mode == FileMode::ReadOnly {
            eprintln!("Error: File '{}' is not open for writing.", filename);
            return Ok(());
        }

        let mut cluster_chain =
            fat32.get_cluster_chain(image_file, open_file.first_cluster)?;

        // Write data
        fat32.write_file_data(
            image_file,
            &mut cluster_chain,
            open_file.offset,
            data,
        )?;

        // Update the offset and file size
        open_file.offset += data.len() as u32;
        if open_file.offset > open_file.file_size {
            open_file.file_size = open_file.offset;
            // Update the file size in the directory entry
            fat32.update_file_size(
                image_file,
                open_file.first_cluster,
                current_dir_cluster,
                open_file.file_size,
            )?;
        }

        println!("Wrote to '{}'.", filename);
    } else {
        eprintln!("Error: File '{}' is not open.", filename);
    }

    Ok(())
}

pub fn rename(
    image_file: &mut File,
    fat32: &mut FAT32,
    current_dir_cluster: u32,
    filename: &str,
    new_filename: &str,
    open_files: &OpenFiles,
) -> io::Result<()> {
    // Cannot rename '.' or '..'
    if filename == "." || filename == ".." {
        eprintln!("Error: Cannot rename special directories '.' or '..'.");
        return Ok(());
    }

    // Check if FILENAME exists in current directory
    let entries = fat32.read_directory_entries(image_file, current_dir_cluster)?;
    let mut entry_to_rename: Option<DirectoryEntry> = None;
    for entry in &entries {
        if format_name(&entry.name) == filename {
            entry_to_rename = Some(entry.clone());
            break;
        }
    }

    if entry_to_rename.is_none() {
        eprintln!("Error: File or directory '{}' does not exist.", filename);
        return Ok(());
    }

    // Check if NEW_FILENAME already exists
    for entry in &entries {
        if format_name(&entry.name) == new_filename {
            eprintln!("Error: A file or directory named '{}' already exists.", new_filename);
            return Ok(());
        }
    }

    // Check if file is open
    if open_files.is_file_open(filename) {
        eprintln!("Error: File '{}' must be closed before renaming.", filename);
        return Ok(());
    }

    // Update the directory entry's name
    let entry = entry_to_rename.unwrap();
    fat32.update_entry_name(
        image_file,
        current_dir_cluster,
        &entry,
        new_filename,
    )?;

    println!("'{}' renamed to '{}'.", filename, new_filename);
    Ok(())
}

pub fn rm(
    image_file: &mut File,
    fat32: &mut FAT32,
    current_dir_cluster: u32,
    filename: &str,
    open_files: &OpenFiles,
) -> io::Result<()> {
    // Check if FILENAME exists and is a file
    let entries = fat32.read_directory_entries(image_file, current_dir_cluster)?;
    let mut file_entry = None;
    for entry in &entries {
        if format_name(&entry.name) == filename {
            if entry.is_directory() {
                eprintln!("Error: '{}' is a directory.", filename);
                return Ok(());
            }
            file_entry = Some(entry.clone());
            break;
        }
    }

    if file_entry.is_none() {
        eprintln!("Error: File '{}' does not exist.", filename);
        return Ok(());
    }

    // Check if file is open
    if open_files.is_file_open(filename) {
        eprintln!("Error: File '{}' is open.", filename);
        return Ok(());
    }

    // Remove the directory entry from the current directory
    let file_entry_unwrapped = file_entry.unwrap();
    fat32.remove_directory_entry(
        image_file,
        current_dir_cluster,
        &file_entry_unwrapped,
    )?;

    // Free the clusters used by the file
    fat32.free_cluster_chain(
        image_file,
        file_entry_unwrapped.first_cluster,
    )?;

    println!("File '{}' deleted.", filename);
    Ok(())
}

pub fn rmdir(
    image_file: &mut File,
    fat32: &mut FAT32,
    current_dir_cluster: u32,
    dirname: &str,
    open_files: &OpenFiles,
) -> io::Result<()> {
    // Cannot remove '.' or '..'
    if dirname == "." || dirname == ".." {
        eprintln!("Error: Cannot remove special directories '.' or '..'.");
        return Ok(());
    }

    // Check if DIRNAME exists and is a directory
    let entries = fat32.read_directory_entries(image_file, current_dir_cluster)?;
    let mut dir_entry = None;
    for entry in &entries {
        if format_name(&entry.name) == dirname {
            if !entry.is_directory() {
                eprintln!("Error: '{}' is not a directory.", dirname);
                return Ok(());
            }
            dir_entry = Some(entry.clone());
            break;
        }
    }

    if dir_entry.is_none() {
        eprintln!("Error: Directory '{}' does not exist.", dirname);
        return Ok(());
    }

    // Check if directory is empty (contains only '.' and '..')
    let dir_cluster = dir_entry.as_ref().unwrap().first_cluster;
    let dir_entries = fat32.read_directory_entries(image_file, dir_cluster)?;
    let mut is_empty = true;
    for entry in &dir_entries {
        let name = format_name(&entry.name);
        if name != "." && name != ".." {
            is_empty = false;
            break;
        }
    }

    if !is_empty {
        eprintln!("Error: Directory '{}' is not empty.", dirname);
        return Ok(());
    }

    // Remove the directory entry from the current directory
    let dir_entry_unwrapped = dir_entry.unwrap();
    fat32.remove_directory_entry(
        image_file,
        current_dir_cluster,
        &dir_entry_unwrapped,
    )?;

    // Free the clusters used by the directory
    fat32.free_cluster_chain(
        image_file,
        dir_cluster,
    )?;

    println!("Directory '{}' removed.", dirname);
    Ok(())
}

// Helper function to format the 11-byte name
fn format_name(name: &str) -> String {
    let mut formatted = name.to_string();
    formatted = formatted.trim().to_string();
    formatted = formatted.replace(" ", "");
    formatted = formatted.trim_matches(char::from(0)).to_string();
    formatted
}
