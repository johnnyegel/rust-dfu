/// Defines a model for mapping out memory

use core::fmt;
use bitflags::bitflags;


bitflags! {
    /// Defines the access types
    pub struct Accessibility: u32 {
        /// Page is readable
        const Read = 0b0001;
        /// Page is writable
        const Write = 0b0010;
        /// Combination of Read and Write
        const ReadWrite = Self::Read.bits | Self::Write.bits;
        /// Erasable (supports block erase)
        const Erase = 0b0100;
        /// Supports all modes. Typically used to define Flash Memory blocks
        const ReadWriteErase = Self::ReadWrite.bits | Self::Erase.bits;
    }
}


/// A memory area defines a set of memory banks, which in turn contains 
/// Banks with sectors consisting of pages
pub struct MemoryMap<'a> {
    /// The name of the memory map
    pub name: &'a str,
    /// Array of banks in the memory map
    banks: Vec<Bank>
}

/// A bank is a set of sectors, defined from some base address
/// The bank also have an index which can be used to define sections
/// with overlapping address space, but located in different banks.
pub struct Bank {
    /// The bank index
    pub index: usize,
    /// The base address of the bank
    pub address: usize,
    /// List of sectors
    sectors: Vec<Sector>
}

/// A sector is a continous section of memory, consisting of several blocks
pub struct Sector {
    /// The index of the first block in the sector
    pub index: usize,

    /// The base address of the sector
    pub address: usize,

    /// Defines the number of blocks in the sector
    pub block_count: usize,
    
    /// Defines the size of each sector block
    pub block_size: usize,

    /// Defines the accessibility of the sector blocks
    pub access: Accessibility
}

/// Implement a memory map
impl<'a> MemoryMap<'a> {

    /// Creates a new memory map, containing the given banks
    pub fn new(name: &'a str, banks: Vec<Bank>) -> Self {
        MemoryMap {
            name: name,
            banks: banks
        }
    }
}

impl<'a> fmt::Display for MemoryMap<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write out the memory map name
        write!(f, "Memory Map [{}]:\n", self.name)?;

        // Iterate the banks 
        for bank in self.banks {
            write!(f, "- {}\n", bank)?;
        }

        Ok(())
    }
}

/// Implement methods for bank
impl Bank {

    /// Creates a new bank from the given parameters
    pub fn new(index: usize, address: usize, sectors: Vec<Sector>) -> Self {
        Bank {
            index: index,
            address: address,
            sectors: sectors
        }
    }

    /// Creates a new bank using the first sector as the base address
    pub fn from_sectors(index: usize, sectors: Vec<Sector>) -> Self {
        // Determine address, but set it to 0 if there are no sectors
        let address = if sectors.len() > 0 {
            sectors[0].address
        }
        else {
            0
        };

        // Create new sector
        Self::new(index, address, sectors)
    }

}

impl fmt::Display for Bank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")?;
        Ok(())
    }
}

/// Sector implementation
impl Sector {

    /// Creates a new free standing sector from the given parameters
    pub fn new(index: usize, address: usize, block_count: usize, block_size: usize, access: Accessibility) -> Self {
        Sector {
            index: index,
            address: address,
            block_count: block_count,
            block_size: block_size,
            access: access
        }
    }

    /// Creates the next sector, direct in continuation for the current one:
    /// - The index will be the current plus the block count
    /// - The address will directly continue after the current address plus the block count times their size
    pub fn next(&self, block_count: usize, block_size: usize, access: Accessibility) -> Self {
        // Calculate the index
        let index = self.index + self.block_count;
        let address = self.address + (self.block_count * self.block_size);

        Self::new(index, address, block_count, block_size, access)
    }


    /// Checks if the given access flags are supported by the sector
    pub fn is_accessible(&self, access: Accessibility) -> bool {
        // Check if bitwise and of given accessbits and the memory areas
        // access bits
        access & self.access == access
    }

    /// Returns the total size in bytes of all the blocks in the sector
    pub fn total_size(&self) -> usize {
        self.block_count * self.block_size
    }

}

