use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};

pub struct BootSector {
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sector_count: u16,
    pub num_fats: u8,
    pub total_sectors: u32,
    pub fat_size_32: u32,
    pub root_cluster: u32,
    pub signature: u16,
}

pub struct FAT32 {
    pub boot_sector: BootSector,
    pub total_clusters: u32,
    pub fat_offset: u64,
    pub data_region_offset: u64,
}

impl FAT32 {
    pub fn new(image_file: &mut File) -> io::Result<Self> {
        let boot_sector = Self::read_boot_sector(image_file)?;

        // Validate FAT32 signature
        if boot_sector.signature != 0xAA55 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid FAT32 signature.",
            ));
        }

        let fat_offset =
            (boot_sector.reserved_sector_count as u64) * (boot_sector.bytes_per_sector as u64);

        let data_region_offset = fat_offset
            + (boot_sector.num_fats as u64)
                * (boot_sector.fat_size_32 as u64)
                * (boot_sector.bytes_per_sector as u64);

        // Total number of clusters
        let total_clusters = (boot_sector.total_sectors
            - boot_sector.reserved_sector_count as u32
            - ((boot_sector.num_fats as u32 * boot_sector.fat_size_32)))
            / boot_sector.sectors_per_cluster as u32;

        Ok(FAT32 {
            boot_sector,
            total_clusters,
            fat_offset,
            data_region_offset,
        })
    }

    fn read_boot_sector(image_file: &mut File) -> io::Result<BootSector> {
        let mut buffer = [0u8; 512];
        image_file.seek(SeekFrom::Start(0))?;
        image_file.read_exact(&mut buffer)?;

        let bytes_per_sector = u16::from_le_bytes([buffer[11], buffer[12]]);
        let sectors_per_cluster = buffer[13];
        let reserved_sector_count = u16::from_le_bytes([buffer[14], buffer[15]]);
        let num_fats = buffer[16];
        let total_sectors_16 = u16::from_le_bytes([buffer[19], buffer[20]]);
        let total_sectors_32 =
            u32::from_le_bytes([buffer[32], buffer[33], buffer[34], buffer[35]]);

        let total_sectors = if total_sectors_16 != 0 {
            total_sectors_16 as u32
        } else {
            total_sectors_32
        };

        let fat_size_16 = u16::from_le_bytes([buffer[22], buffer[23]]);
        let fat_size_32 =
            u32::from_le_bytes([buffer[36], buffer[37], buffer[38], buffer[39]]);
        let fat_size = if fat_size_16 != 0 {
            fat_size_16 as u32
        } else {
            fat_size_32
        };

        let root_cluster =
            u32::from_le_bytes([buffer[44], buffer[45], buffer[46], buffer[47]]);

        let signature = u16::from_le_bytes([buffer[510], buffer[511]]);

        Ok(BootSector {
            bytes_per_sector,
            sectors_per_cluster,
            reserved_sector_count,
            num_fats,
            total_sectors,
            fat_size_32: fat_size,
            root_cluster,
            signature,
        })
    }

    pub fn cluster_to_offset(&self, cluster: u32) -> u64 {
        let cluster_num = cluster - 2;
        self.data_region_offset
            + (cluster_num as u64)
                * (self.boot_sector.sectors_per_cluster as u64)
                * (self.boot_sector.bytes_per_sector as u64)
    }

    pub fn read_directory_entries(
        &self,
        image_file: &mut File,
        mut cluster: u32,
    ) -> io::Result<Vec<DirectoryEntry>> {
        let mut entries = Vec::new();

        loop {
            let cluster_offset = self.cluster_to_offset(cluster);

            let mut offset = cluster_offset;
            let cluster_size = (self.boot_sector.sectors_per_cluster as u64)
                * (self.boot_sector.bytes_per_sector as u64);

            while offset < cluster_offset + cluster_size {
                image_file.seek(SeekFrom::Start(offset))?;
                let mut buffer = [0u8; 32];
                image_file.read_exact(&mut buffer)?;

                if buffer[0] == 0x00 {
                    // No more entries
                    return Ok(entries);
                }

                if buffer[0] == 0xE5 {
                    // Deleted entry, skip
                    offset += 32;
                    continue;
                }

                let attr = buffer[11];
                if attr == 0x0F {
                    // Long file name entry, skip
                    offset += 32;
                    continue;
                }

                let name = String::from_utf8_lossy(&buffer[0..11])
                    .trim()
                    .to_string();

                let first_cluster_high = u16::from_le_bytes([buffer[20], buffer[21]]);
                let first_cluster_low = u16::from_le_bytes([buffer[26], buffer[27]]);
                let first_cluster =
                    ((first_cluster_high as u32) << 16) | first_cluster_low as u32;
                let file_size =
                    u32::from_le_bytes([buffer[28], buffer[29], buffer[30], buffer[31]]);

                let entry = DirectoryEntry {
                    name,
                    attr,
                    first_cluster,
                    file_size,
                };

                entries.push(entry);

                offset += 32;
            }

            // Get next cluster in the chain
            cluster = self.get_next_cluster(image_file, cluster)?;

            // Check if end of cluster chain
            if cluster >= 0x0FFFFFF8 {
                break;
            }
        }

        Ok(entries)
    }

    pub fn get_next_cluster(&self, image_file: &mut File, cluster: u32) -> io::Result<u32> {
        let fat_offset = self.fat_offset + (cluster * 4) as u64;
        image_file.seek(SeekFrom::Start(fat_offset))?;
        let mut buffer = [0u8; 4];
        image_file.read_exact(&mut buffer)?;
        let next_cluster = u32::from_le_bytes(buffer) & 0x0FFFFFFF;
        Ok(next_cluster)
    }

    pub fn read_file_data(
        &self,
        image_file: &mut File,
        cluster_chain: &[u32],
        offset: u32,
        size: u32,
    ) -> io::Result<Vec<u8>> {
        let mut data = Vec::new();
        let bytes_per_cluster = self.bytes_per_cluster();

        let mut remaining_size = size;
        let current_offset = offset;
        let mut cluster_index = (current_offset / bytes_per_cluster) as usize;
        let mut cluster_offset = current_offset % bytes_per_cluster;

        while remaining_size > 0 && cluster_index < cluster_chain.len() {
            let cluster = cluster_chain[cluster_index];
            let cluster_start = self.cluster_to_offset(cluster);

            let to_read = std::cmp::min(
                remaining_size,
                bytes_per_cluster - cluster_offset,
            ) as usize;

            image_file.seek(SeekFrom::Start(
                cluster_start + cluster_offset as u64,
            ))?;

            let mut buffer = vec![0u8; to_read];
            image_file.read_exact(&mut buffer)?;
            data.extend(buffer);

            remaining_size -= to_read as u32;
            cluster_offset = 0;
            cluster_index += 1;
        }

        Ok(data)
    }

    pub fn write_file_data(
        &mut self,
        image_file: &mut File,
        cluster_chain: &mut Vec<u32>,
        offset: u32,
        data: &[u8],
    ) -> io::Result<()> {
        let bytes_per_cluster = self.bytes_per_cluster();

        let mut remaining_size = data.len() as u32;
        let current_offset = offset;
        let mut data_offset = 0usize;

        // Ensure the cluster chain is long enough
        let required_size = offset + remaining_size;
        let required_clusters = (required_size + bytes_per_cluster - 1)
            / bytes_per_cluster;

        while (cluster_chain.len() as u32) < required_clusters {
            let new_cluster = self.allocate_cluster(image_file)?;
            if let Some(last_cluster) = cluster_chain.last() {
                self.set_next_cluster(image_file, *last_cluster, new_cluster)?;
            }
            cluster_chain.push(new_cluster);
        }

        // Write data
        let mut cluster_index = (current_offset / bytes_per_cluster) as usize;
        let mut cluster_offset = current_offset % bytes_per_cluster;

        while remaining_size > 0 && cluster_index < cluster_chain.len() {
            let cluster = cluster_chain[cluster_index];
            let cluster_start = self.cluster_to_offset(cluster);

            let to_write = std::cmp::min(
                remaining_size,
                bytes_per_cluster - cluster_offset,
            ) as usize;

            image_file.seek(SeekFrom::Start(
                cluster_start + cluster_offset as u64,
            ))?;

            image_file.write_all(
                &data[data_offset..data_offset + to_write],
            )?;

            remaining_size -= to_write as u32;
            data_offset += to_write;
            cluster_offset = 0;
            cluster_index += 1;
        }

        Ok(())
    }

    pub fn get_cluster_chain(
        &self,
        image_file: &mut File,
        start_cluster: u32,
    ) -> io::Result<Vec<u32>> {
        let mut chain = Vec::new();
        let mut cluster = start_cluster;

        while cluster < 0x0FFFFFF8 && cluster != 0 {
            chain.push(cluster);
            cluster = self.get_next_cluster(image_file, cluster)?;
        }

        Ok(chain)
    }

    pub fn bytes_per_cluster(&self) -> u32 {
        self.boot_sector.bytes_per_sector as u32
            * self.boot_sector.sectors_per_cluster as u32
    }

    pub fn allocate_cluster(&mut self, image_file: &mut File) -> io::Result<u32> {
        // Search the FAT for a free cluster (0x00000000)
        for cluster in 2..self.total_clusters {
            let next_cluster = self.get_next_cluster(image_file, cluster)?;
            if next_cluster == 0x00000000 {
                // Mark cluster as end of chain
                self.set_next_cluster(image_file, cluster, 0x0FFFFFF8)?;
                return Ok(cluster);
            }
        }
        Err(io::Error::new(
            io::ErrorKind::Other,
            "No free clusters available.",
        ))
    }

    pub fn set_next_cluster(
        &mut self,
        image_file: &mut File,
        cluster: u32,
        next_cluster: u32,
    ) -> io::Result<()> {
        let fat_offset = self.fat_offset + (cluster * 4) as u64;
        image_file.seek(SeekFrom::Start(fat_offset))?;
        let next_cluster_bytes = (next_cluster & 0x0FFFFFFF).to_le_bytes();
        image_file.write_all(&next_cluster_bytes)?;

        // Update copies of FAT if necessary (omitted for simplicity)

        Ok(())
    }

    // Implemented methods

    // create_directory and related helper methods
    pub fn create_directory(
        &mut self,
        image_file: &mut File,
        parent_cluster: u32,
        dirname: &str,
    ) -> io::Result<()> {
        // Allocate a new cluster for the directory
        let new_dir_cluster = self.allocate_cluster(image_file)?;

        // Initialize the new directory with '.' and '..' entries
        self.initialize_directory(image_file, new_dir_cluster, parent_cluster)?;

        // Create a directory entry in the parent directory
        self.add_directory_entry(
            image_file,
            parent_cluster,
            dirname,
            new_dir_cluster,
            true, // is_directory
        )?;

        Ok(())
    }

    fn initialize_directory(
        &self,
        image_file: &mut File,
        dir_cluster: u32,
        parent_cluster: u32,
    ) -> io::Result<()> {
        let dir_offset = self.cluster_to_offset(dir_cluster);
        image_file.seek(SeekFrom::Start(dir_offset))?;

        // Create '.' entry
        let dot_entry = self.create_directory_entry(
            ".",
            0x10, // attr: Directory attribute
            dir_cluster,
            0, // file_size
        );
        image_file.write_all(&dot_entry)?;

        // Create '..' entry
        let dotdot_entry = self.create_directory_entry(
            "..",
            0x10,
            parent_cluster,
            0,
        );
        image_file.write_all(&dotdot_entry)?;

        // Fill the rest of the cluster with zeros
        let remaining_size = self.bytes_per_cluster() - 64; // 64 bytes for two entries
        let zero_buffer = vec![0u8; remaining_size as usize];
        image_file.write_all(&zero_buffer)?;

        Ok(())
    }

    // create_file method
    pub fn create_file(
        &mut self,
        image_file: &mut File,
        parent_cluster: u32,
        filename: &str,
    ) -> io::Result<()> {
        // No need to allocate a cluster for an empty file
        let first_cluster = 0;

        // Create a file entry in the parent directory
        self.add_directory_entry(
            image_file,
            parent_cluster,
            filename,
            first_cluster,
            false, // is_directory
        )?;

        Ok(())
    }

    // Helper method to add a directory entry
    fn add_directory_entry(
        &mut self,
        image_file: &mut File,
        dir_cluster: u32,
        name: &str,
        first_cluster: u32,
        is_directory: bool,
    ) -> io::Result<()> {
        let mut cluster = dir_cluster;

        loop {
            let cluster_offset = self.cluster_to_offset(cluster);

            let cluster_size = self.bytes_per_cluster();
            let mut offset = cluster_offset;
            while offset < cluster_offset + cluster_size as u64 {
                image_file.seek(SeekFrom::Start(offset))?;
                let mut buffer = [0u8; 32];
                image_file.read_exact(&mut buffer)?;

                if buffer[0] == 0x00 || buffer[0] == 0xE5 {
                    // Found an empty or deleted entry, can use this slot
                    image_file.seek(SeekFrom::Start(offset))?;
                    let attr = if is_directory { 0x10 } else { 0x20 };
                    let entry = self.create_directory_entry(
                        name,
                        attr,
                        first_cluster,
                        0, // file_size
                    );
                    image_file.write_all(&entry)?;
                    return Ok(());
                }

                offset += 32;
            }

            // Need to move to the next cluster in the directory
            let next_cluster = self.get_next_cluster(image_file, cluster)?;
            if next_cluster >= 0x0FFFFFF8 {
                // End of cluster chain, need to allocate a new cluster
                let new_cluster = self.allocate_cluster(image_file)?;
                self.set_next_cluster(image_file, cluster, new_cluster)?;
                cluster = new_cluster;

                // Initialize the new cluster with zeros
                let new_cluster_offset = self.cluster_to_offset(new_cluster);
                image_file.seek(SeekFrom::Start(new_cluster_offset))?;
                let zero_buffer = vec![0u8; self.bytes_per_cluster() as usize];
                image_file.write_all(&zero_buffer)?;
            } else {
                cluster = next_cluster;
            }
        }
    }

    // Helper method to create a directory entry
    fn create_directory_entry(
        &self,
        name: &str,
        attr: u8,
        first_cluster: u32,
        file_size: u32,
    ) -> [u8; 32] {
        let mut entry = [0u8; 32];
        let name_bytes = self.format_filename(name);
        entry[0..11].copy_from_slice(&name_bytes);
        entry[11] = attr;

        // First cluster high (bits 16-31)
        entry[20..22].copy_from_slice(&( ( (first_cluster >> 16) as u16 ).to_le_bytes() ));
        // First cluster low (bits 0-15)
        entry[26..28].copy_from_slice(&( (first_cluster as u16).to_le_bytes() ));
        // File size
        entry[28..32].copy_from_slice(&file_size.to_le_bytes());

        entry
    }

    // Helper method to format the filename
    fn format_filename(&self, name: &str) -> [u8; 11] {
        let mut name_bytes = [0x20u8; 11]; // Fill with spaces
        let name = name.to_uppercase();

        let (name_part, ext_part) = if let Some(pos) = name.find('.') {
            (&name[..pos], &name[pos + 1..])
        } else {
            (name.as_str(), "")
        };

        let name_part = &name_part[..std::cmp::min(8, name_part.len())];
        let ext_part = &ext_part[..std::cmp::min(3, ext_part.len())];

        name_bytes[..name_part.len()].copy_from_slice(name_part.as_bytes());
        name_bytes[8..8 + ext_part.len()].copy_from_slice(ext_part.as_bytes());

        name_bytes
    }

    // update_entry_name method
    pub fn update_entry_name(
        &mut self,
        image_file: &mut File,
        dir_cluster: u32,
        entry: &DirectoryEntry,
        new_name: &str,
    ) -> io::Result<()> {
        let mut cluster = dir_cluster;

        loop {
            let cluster_offset = self.cluster_to_offset(cluster);

            let cluster_size = self.bytes_per_cluster();
            let mut offset = cluster_offset;
            while offset < cluster_offset + cluster_size as u64 {
                image_file.seek(SeekFrom::Start(offset))?;
                let mut buffer = [0u8; 32];
                image_file.read_exact(&mut buffer)?;

                if buffer[0] == 0x00 {
                    // No more entries
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        "Directory entry not found.",
                    ));
                }

                if buffer[0] == 0xE5 {
                    // Deleted entry, skip
                    offset += 32;
                    continue;
                }

                let attr = buffer[11];
                if attr == 0x0F {
                    // Long file name entry, skip
                    offset += 32;
                    continue;
                }

                let name = String::from_utf8_lossy(&buffer[0..11])
                    .trim()
                    .to_string();

                if name == entry.name {
                    // Found the entry, update the name
                    let name_bytes = self.format_filename(new_name);
                    image_file.seek(SeekFrom::Start(offset))?;
                    image_file.write_all(&name_bytes)?;
                    return Ok(());
                }

                offset += 32;
            }

            // Move to the next cluster in the directory
            let next_cluster = self.get_next_cluster(image_file, cluster)?;
            if next_cluster >= 0x0FFFFFF8 {
                // End of cluster chain, entry not found
                break;
            } else {
                cluster = next_cluster;
            }
        }

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Directory entry not found.",
        ))
    }

    // remove_directory_entry method
    pub fn remove_directory_entry(
        &mut self,
        image_file: &mut File,
        dir_cluster: u32,
        entry: &DirectoryEntry,
    ) -> io::Result<()> {
        let mut cluster = dir_cluster;

        loop {
            let cluster_offset = self.cluster_to_offset(cluster);

            let cluster_size = self.bytes_per_cluster();
            let mut offset = cluster_offset;
            while offset < cluster_offset + cluster_size as u64 {
                image_file.seek(SeekFrom::Start(offset))?;
                let mut buffer = [0u8; 32];
                image_file.read_exact(&mut buffer)?;

                if buffer[0] == 0x00 {
                    // No more entries
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        "Directory entry not found.",
                    ));
                }

                if buffer[0] == 0xE5 {
                    // Deleted entry, skip
                    offset += 32;
                    continue;
                }

                let attr = buffer[11];
                if attr == 0x0F {
                    // Long file name entry, skip
                    offset += 32;
                    continue;
                }

                let name = String::from_utf8_lossy(&buffer[0..11])
                    .trim()
                    .to_string();

                if name == entry.name {
                    // Found the entry, mark it as deleted
                    image_file.seek(SeekFrom::Start(offset))?;
                    image_file.write_all(&[0xE5])?;
                    return Ok(());
                }

                offset += 32;
            }

            // Move to the next cluster in the directory
            let next_cluster = self.get_next_cluster(image_file, cluster)?;
            if next_cluster >= 0x0FFFFFF8 {
                // End of cluster chain, entry not found
                break;
            } else {
                cluster = next_cluster;
            }
        }

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Directory entry not found.",
        ))
    }

    // free_cluster_chain method
    pub fn free_cluster_chain(
        &mut self,
        image_file: &mut File,
        start_cluster: u32,
    ) -> io::Result<()> {
        let mut cluster = start_cluster;

        while cluster < 0x0FFFFFF8 && cluster != 0 {
            let next_cluster = self.get_next_cluster(image_file, cluster)?;
            // Mark the cluster as free (0x00000000)
            self.set_next_cluster(image_file, cluster, 0x00000000)?;
            cluster = next_cluster;
        }

        Ok(())
    }

    // update_file_size method
    pub fn update_file_size(
        &mut self,
        image_file: &mut File,
        first_cluster: u32,
        dir_cluster: u32,
        new_size: u32,
    ) -> io::Result<()> {
        let mut cluster = dir_cluster;

        loop {
            let cluster_offset = self.cluster_to_offset(cluster);

            let cluster_size = self.bytes_per_cluster();
            let mut offset = cluster_offset;
            while offset < cluster_offset + cluster_size as u64 {
                image_file.seek(SeekFrom::Start(offset))?;
                let mut buffer = [0u8; 32];
                image_file.read_exact(&mut buffer)?;

                if buffer[0] == 0x00 {
                    // No more entries
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        "File entry not found.",
                    ));
                }

                if buffer[0] == 0xE5 {
                    // Deleted entry, skip
                    offset += 32;
                    continue;
                }

                let attr = buffer[11];
                if attr == 0x0F {
                    // Long file name entry, skip
                    offset += 32;
                    continue;
                }

                let entry_first_cluster_high =
                    u16::from_le_bytes([buffer[20], buffer[21]]) as u32;
                let entry_first_cluster_low =
                    u16::from_le_bytes([buffer[26], buffer[27]]) as u32;
                let entry_first_cluster =
                    (entry_first_cluster_high << 16) | entry_first_cluster_low;

                if entry_first_cluster == first_cluster {
                    // Found the file entry, update the file size
                    image_file.seek(SeekFrom::Start(offset + 28))?;
                    image_file.write_all(&new_size.to_le_bytes())?;
                    return Ok(());
                }

                offset += 32;
            }

            // Move to the next cluster in the directory
            let next_cluster = self.get_next_cluster(image_file, cluster)?;
            if next_cluster >= 0x0FFFFFF8 {
                // End of cluster chain, entry not found
                break;
            } else {
                cluster = next_cluster;
            }
        }

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "File entry not found.",
        ))
    }
}

#[derive(Debug, Clone)]
pub struct DirectoryEntry {
    pub name: String,
    pub attr: u8,
    pub first_cluster: u32,
    pub file_size: u32,
}

impl DirectoryEntry {
    pub fn is_directory(&self) -> bool {
        self.attr & 0x10 != 0
    }

    pub fn is_file(&self) -> bool {
        !self.is_directory()
    }
}
