use std::fs::File;
use std::io::{self, Write};

use crate::commands;
use crate::fat32::FAT32;
use crate::open_files::OpenFiles;

pub fn run_shell(image_file: &mut File, fat32: &mut FAT32) -> io::Result<()> {
    let mut input = String::new();
    let stdin = io::stdin();

    let mut current_dir_cluster = fat32.boot_sector.root_cluster;
    let mut open_files = OpenFiles::new();

    loop {
        print!("filesys> ");
        io::stdout().flush()?;
        input.clear();
        stdin.read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        let mut args = input.split_whitespace();
        let command = args.next().unwrap();

        match command {
            "exit" => {
                println!("Exiting...");
                break;
            }
            "info" => {
                commands::info(fat32)?;
            }
            "ls" => {
                commands::ls(image_file, fat32, current_dir_cluster)?;
            }
            "cd" => {
                if let Some(dirname) = args.next() {
                    commands::cd(image_file, fat32, &mut current_dir_cluster, dirname)?;
                } else {
                    eprintln!("Error: 'cd' command requires a directory name.");
                }
            }
            "mkdir" => {
                if let Some(dirname) = args.next() {
                    commands::mkdir(image_file, fat32, current_dir_cluster, dirname)?;
                } else {
                    eprintln!("Error: 'mkdir' command requires a directory name.");
                }
            }
            "creat" => {
                if let Some(filename) = args.next() {
                    commands::creat(image_file, fat32, current_dir_cluster, filename)?;
                } else {
                    eprintln!("Error: 'creat' command requires a file name.");
                }
            }
            "open" => {
                let filename = args.next();
                let flags = args.next();
                if filename.is_some() && flags.is_some() {
                    commands::open(
                        image_file,
                        fat32,
                        current_dir_cluster,
                        filename.unwrap(),
                        flags.unwrap(),
                        &mut open_files,
                    )?;
                } else {
                    eprintln!("Error: 'open' command requires a filename and flags.");
                }
            }
            "close" => {
                if let Some(filename) = args.next() {
                    commands::close(filename, &mut open_files)?;
                } else {
                    eprintln!("Error: 'close' command requires a filename.");
                }
            }
            "lsof" => {
                commands::lsof(&open_files)?;
            }
            "size" => {
                if let Some(filename) = args.next() {
                    commands::size(image_file, fat32, current_dir_cluster, filename)?;
                } else {
                    eprintln!("Error: 'size' command requires a filename.");
                }
            }
            "lseek" => {
                let filename = args.next();
                let offset = args.next();
                if filename.is_some() && offset.is_some() {
                    commands::lseek(filename.unwrap(), offset.unwrap(), &mut open_files)?;
                } else {
                    eprintln!("Error: 'lseek' command requires a filename and offset.");
                }
            }
            "read" => {
                let filename = args.next();
                let size = args.next();
                if filename.is_some() && size.is_some() {
                    commands::read(
                        image_file,
                        fat32,
                        filename.unwrap(),
                        size.unwrap(),
                        &mut open_files,
                    )?;
                } else {
                    eprintln!("Error: 'read' command requires a filename and size.");
                }
            }
            "write" => {
                let filename = args.next();
                if let Some(filename) = filename {
                    let string = args.collect::<Vec<&str>>().join(" ");
                    if !string.is_empty() {
                        commands::write(
                            image_file,
                            fat32,
                            current_dir_cluster,
                            filename,
                            &string,
                            &mut open_files,
                        )?;
                    } else {
                        eprintln!("Error: 'write' command requires a string to write.");
                    }
                } else {
                    eprintln!("Error: 'write' command requires a filename and string.");
                }
            }
            "rm" => {
                if let Some(filename) = args.next() {
                    commands::rm(
                        image_file,
                        fat32,
                        current_dir_cluster,
                        filename,
                        &open_files,
                    )?;
                } else {
                    eprintln!("Error: 'rm' command requires a filename.");
                }
            }
            "rmdir" => {
                if let Some(dirname) = args.next() {
                    commands::rmdir(
                        image_file,
                        fat32,
                        current_dir_cluster,
                        dirname,
                        &open_files,
                    )?;
                } else {
                    eprintln!("Error: 'rmdir' command requires a directory name.");
                }
            }
            "rename" => {
                let old_name = args.next();
                let new_name = args.next();
                if old_name.is_some() && new_name.is_some() {
                    commands::rename(
                        image_file,
                        fat32,
                        current_dir_cluster,
                        old_name.unwrap(),
                        new_name.unwrap(),
                        &open_files,
                    )?;
                } else {
                    eprintln!("Error: 'rename' command requires old and new filenames.");
                }
            }
            _ => {
                eprintln!("Unknown command: {}", command);
            }
        }
    }

    Ok(())
}
