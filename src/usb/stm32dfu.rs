/*
Enumerating the STM32 DFU reveals the following

Bus 001 Device 006: ID 0483:df11 STMicroelectronics STM Device in DFU Mode
Device Descriptor:
  bLength                18
  bDescriptorType         1
  bcdUSB               1.00
  bDeviceClass            0
  bDeviceSubClass         0
  bDeviceProtocol         0
  bMaxPacketSize0        64
  idVendor           0x0483 STMicroelectronics
  idProduct          0xdf11 STM Device in DFU Mode
  bcdDevice           22.00
  iManufacturer           1 STMicroelectronics
  iProduct                2 STM32  BOOTLOADER
  iSerial                 3 348435943539
  bNumConfigurations      1
  Configuration Descriptor:
    bLength                 9
    bDescriptorType         2
    wTotalLength       0x0036
    bNumInterfaces          1
    bConfigurationValue     1
    iConfiguration          0
    bmAttributes         0xc0
      Self Powered
    MaxPower              100mA
    Interface Descriptor:
      bLength                 9
      bDescriptorType         4
      bInterfaceNumber        0
      bAlternateSetting       0
      bNumEndpoints           0
      bInterfaceClass       254 Application Specific Interface
      bInterfaceSubClass      1 Device Firmware Update
      bInterfaceProtocol      2
      iInterface              4 @Internal Flash  /0x08000000/04*016Kg,01*064Kg,03*128Kg
    Interface Descriptor:
      bLength                 9
      bDescriptorType         4
      bInterfaceNumber        0
      bAlternateSetting       1
      bNumEndpoints           0
      bInterfaceClass       254 Application Specific Interface
      bInterfaceSubClass      1 Device Firmware Update
      bInterfaceProtocol      2
      iInterface              5 @Option Bytes  /0x1FFFC000/01*016 e
    Interface Descriptor:
      bLength                 9
      bDescriptorType         4
      bInterfaceNumber        0
      bAlternateSetting       2
      bNumEndpoints           0
      bInterfaceClass       254 Application Specific Interface
      bInterfaceSubClass      1 Device Firmware Update
      bInterfaceProtocol      2
      iInterface              6 @OTP Memory /0x1FFF7800/01*512 e,01*016 e
    Interface Descriptor:
      bLength                 9
      bDescriptorType         4
      bInterfaceNumber        0
      bAlternateSetting       3
      bNumEndpoints           0
      bInterfaceClass       254 Application Specific Interface
      bInterfaceSubClass      1 Device Firmware Update
      bInterfaceProtocol      2
      iInterface              7 @Device Feature/0xFFFF0000/01*004 e
      Device Firmware Upgrade Interface Descriptor:
        bLength                             9
        bDescriptorType                    33
        bmAttributes                       11
          Will Detach
          Manifestation Intolerant
          Upload Supported
          Download Supported
        wDetachTimeout                    255 milliseconds
        wTransferSize                    2048 bytes
        bcdDFUVersion                   1.1a


Interface strings are formatted as follows:
"@<MemoryAreaName>/<BaseAddress>/<MemoryLayout>[/<BaseAddress>/<MemoryLayout>[/...]]"

Although not normal, there can be any number of banks

The MemoryLayout is as follows, a.e a comma separated list of PageSectors
"<PageSector>[,<PageSector>[,...]]"

PageSection have the following format
"<PageCount>*<PageSize><SizeType><Unknown>"

SizeType is either:
- "M" - Mebi, or 1024 * 1024
- "K" - Kibi, or 1024
- " " - Just bytes

The last char indicates the accessibility of the memory
– a (0x41): Readable
– b (0x42): Erasable
– c (0x43): Readable and Erasabled
- d (0x44): Writeable
– e (0x45): Readable and Writeable
– f (0x46): Erasable and Writeable
– g (0x47): Readable, Erasable and Writeable

*/

use crate::util::memory::{Accessibility, Bank, MemoryMap, Sector};
use crate::util::parse;

#[derive(Debug)]
pub enum DefParseError {
    InvalidStartChar,
    InvalidBankPartCount,
    AddressParseError,
    InvalidSectorDefinition,
}

// [@Internal Flash  /0x08000000/04*016Kg,01*064Kg,03*128Kg]

pub fn parse_memory_layout_string(ifstring: &str) -> Result<MemoryMap, DefParseError> {
    // Split the string by slash
    let mut ifstrparts = ifstring.split('/');

    // Get the first part as the name, and return error if it's start char is not correct
    let namestr_part = ifstrparts
        .next()
        .ok_or(DefParseError::InvalidBankPartCount)?;
    if !namestr_part.starts_with('@') {
        return Err(DefParseError::InvalidStartChar);
    }

    // Then iterate Banks
    let mut bank_list: Vec<Bank> = Vec::new();
    let mut bank_index = 0;

    // Iterate until we break
    loop {
        // If there are no more parts, we are done parsing here
        let donecheck = ifstrparts.next();
        if donecheck.is_none() {
            break;
        }

        // Get the base address string
        let base_address_parsed = parse::usize_from_string(donecheck.unwrap());
        if base_address_parsed.is_err() {
            return Err(DefParseError::AddressParseError);
        }

        // Parse the base address and sector layout
        let base_address = base_address_parsed.unwrap();
        let sector_layouts_string = ifstrparts
            .next()
            .ok_or(DefParseError::InvalidBankPartCount)?;

        let mut sector_list: Vec<Sector> = Vec::new();
        let mut sector_address = base_address;
        let mut sector_index = 0;

        // The sector layout is comma separated, so let's split it
        for sector_layout in sector_layouts_string.split(',') {
            // Parse the sector
            let sector = parse_sector_layout(sector_index, sector_address, sector_layout)?;

            // Move address to the first address after the sector
            sector_index += sector.block_count;
            sector_address += sector.total_size();

            // Push to list
            sector_list.push(sector);
        }

        // Finally create the bank from the vectors, and push it to the bank list
        let bank = Bank::from_sectors(bank_index, sector_list);
        bank_list.push(bank);

        // Increase the bank index
        bank_index += 1;
    }

    // Get the name string
    let namestr = namestr_part[1..].trim();

    // Return a dummy OK result
    Ok(MemoryMap::new(namestr, bank_list))
}

// 04*016Kg  01*064Kg  03*128Kg]

/// Parse a layout string into a sector
fn parse_sector_layout(
    sector_index: usize,
    sector_address: usize,
    layoutstr: &str,
) -> Result<Sector, DefParseError> {
    // The position of descriptor chars
    let dix = layoutstr.len() - 2;

    // Split into two parts, and then extract them
    let mut layout_parts = layoutstr[..dix].split('*');
    let def_chars = &layoutstr[dix..];

    // Get the block count string and block size strings
    let block_count_str = layout_parts
        .next()
        .ok_or(DefParseError::InvalidSectorDefinition)?
        .trim();
    let block_sizen_str = layout_parts
        .next()
        .ok_or(DefParseError::InvalidSectorDefinition)?
        .trim();

    // Parse the block count and size
    let block_count = block_count_str
        .parse::<usize>()
        .or_else(|_| Err(DefParseError::InvalidSectorDefinition))?;
    let mut block_size = block_sizen_str
        .parse::<usize>()
        .or_else(|_| Err(DefParseError::InvalidSectorDefinition))?;

    // Get the size multiplier char, and  parse it
    let size_multiplier_char = def_chars
        .chars()
        .nth(0)
        .ok_or(DefParseError::InvalidSectorDefinition)?;
    let access_type = def_chars
        .chars()
        .nth(1)
        .ok_or(DefParseError::InvalidSectorDefinition)?;
    block_size *= match size_multiplier_char {
        'M' => 1024 * 1024,
        'K' => 1024,
        ' ' => 0,
        _ => return Err(DefParseError::InvalidSectorDefinition),
    };

    // The last char defines the accessibility as follows
    let access = match access_type {
        // – a (0x41): Readable
        'a' => Accessibility::READ,
        // – b (0x42): Erasable
        'b' => Accessibility::ERASE,
        // – c (0x43): Readable and Erasabled
        'c' => Accessibility::READ_ERASE,
        // – d (0x44): Writable
        'd' => Accessibility::WRITE,
        // – e (0x45): Readable and Writeable
        'e' => Accessibility::READ_WRITE,
        // – f (0x46): Erasable and Writeable
        'f' => Accessibility::WRITE_ERASE,
        // – g (0x47): Readable, Erasable and Writeable
        'g' => Accessibility::READ_WRITE_ERASE,
        // If not, return error
        _ => return Err(DefParseError::InvalidSectorDefinition),
    };

    // Finally return the sector, and simply assume it's read write erase
    Ok(Sector::new(
        sector_index,
        sector_address,
        block_count,
        block_size,
        access,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_memory_definition_string() {
        let defstr = "@Internal Flash  /0x08000000/04*016Kg,01*064Kg,03*128Kg";
        let memmap_result = parse_memory_layout_string(defstr);

        if memmap_result.is_err() {
            print!(
                "Parse failed, which should not happen: {:?}\n",
                memmap_result.unwrap_err()
            );
            assert!(true);
            return;
        }

        // Get the memory map
        let memmap = memmap_result.unwrap();
        print!("Memory map: {}", memmap);

        // Assert the content of the map
        assert_eq!("Internal Flash", memmap.name);
        assert_eq!(1, memmap.banks().len());

        // Borrow a reference to the
        let bank = &memmap.banks()[0];

        // Check the bank properties
        assert_eq!(0x8000000, bank.address);
        assert_eq!(0, bank.index);

        // Get a reference to the sectors
        let sectors = bank.sectors();

        // Sectors 0-3
        assert_eq!(0, sectors[0].index);
        assert_eq!(0x8000000, sectors[0].address);
        assert_eq!(0x10000, sectors[0].total_size());

        // Sector 4
        assert_eq!(4, sectors[1].index);
        assert_eq!(0x8010000, sectors[1].address);
        assert_eq!(0x10000, sectors[1].total_size());

        // Sectors 5-7
        assert_eq!(5, sectors[2].index);
        assert_eq!(0x8020000, sectors[2].address);
        assert_eq!(0x60000, sectors[2].total_size());
    }
}
