mod fat32;
mod shell;
mod commands;
mod open_files;

use std::env;
use std::fs::File;
use std::io::{self};

use fat32::FAT32;

fn main() -> io::Result<()> {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: filesys [FAT32_IMAGE]");
        std::process::exit(1);
    }

    let image_path = &args[1];

    // Open the image file with read and write permissions
    let image_file = File::options().read(true).write(true).open(image_path);
    if image_file.is_err() {
        eprintln!("Error: Cannot open image file '{}'.", image_path);
        std::process::exit(1);
    }
    let mut image_file = image_file.unwrap();

    // Initialize FAT32 file system
    let fat32 = FAT32::new(&mut image_file);
    if fat32.is_err() {
        eprintln!("Error: Invalid FAT32 file system.");
        std::process::exit(1);
    }
    let mut fat32 = fat32.unwrap();

    // Run the shell
    shell::run_shell(&mut image_file, &mut fat32)?;

    Ok(())
}
