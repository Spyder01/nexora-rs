use core::mem::size_of;
use std::convert::TryInto;

use crate::utils::encoding::endian::endian::{write_bytes, write_u16_le, write_u64_le, read_u64_le};

pub const FILE_HEADER_MAGIC: [u8; 6] = *b"NXRv0\0";
pub const PROPERTY_NAME_MAX_SIZE: usize = 55;
pub const MAX_PROPERTIES_COUNT: usize = 120;
pub const PAGE_SIZE: usize = 4096;
pub const KB1: usize = 1024;
pub const INVALID_OFFSET: u64 = u64::MAX;

/// -------------------- Header --------------------
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NexoraHeader {
    pub footer_offset: u64,    // 8
    pub created_unix: u64,     // 8
    pub magic: [u8; 6],        // 6
    pub version: u16,          // 2
    pub flags: u16,            // 2
    pub _reserved: [u8; 4070], // 4070 + 8+8+6+2+2 = 4096
}

impl Default for NexoraHeader {
    fn default() -> Self {
        Self {
            footer_offset: INVALID_OFFSET,
            created_unix: 0,
            magic: FILE_HEADER_MAGIC,
            version: 0,
            flags: 0,
            _reserved: [0u8; 4070],
        }
    }
}

impl NexoraHeader {
    pub fn verify_magic(raw_magic: [u8; 6]) -> bool {
        raw_magic == FILE_HEADER_MAGIC
    }

    pub fn deserialize(raw_header_data: [u8; PAGE_SIZE]) -> Self {
        let mut offset = 0;

        // helper macro to grab slices safely
        macro_rules! take {
            ($len:expr) => {{
                let start = offset;
                let end = offset + $len;
                offset = end;
                &raw_header_data[start..end]
            }};
        }

        let footer_offset = u64::from_le_bytes(take!(8).try_into().unwrap());
        let created_unix = u64::from_le_bytes(take!(8).try_into().unwrap());

        let mut magic = [0u8; 6];
        magic.copy_from_slice(take!(6));

        let version = u16::from_le_bytes(take!(2).try_into().unwrap());
        let flags = u16::from_le_bytes(take!(2).try_into().unwrap());

        let mut reserved = [0u8; 4070];
        reserved.copy_from_slice(take!(4070));

        Self {
            footer_offset,
            created_unix,
            magic,
            version,
            flags,
            _reserved: reserved,
        }
    }
}

const _: () = assert!(size_of::<NexoraHeader>() == PAGE_SIZE);


/// -------------------- Offset Metadata --------------------
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct OffsetMetadataTable {
    pub nb_total_items: u64,
    pub base_chunk_offset: u64,
}
const _: () = assert!(size_of::<OffsetMetadataTable>() == 16);

/// -------------------- Footer --------------------
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NexoraFooter {
    pub name_table_offset: OffsetMetadataTable,
    pub node_schema_offset: OffsetMetadataTable,
    pub edge_schema_offset: OffsetMetadataTable,
    pub schema_properties_offset: OffsetMetadataTable,
    pub metadata_offset: OffsetMetadataTable,
    pub indices_offset: OffsetMetadataTable,
    pub nodes_offset: OffsetMetadataTable,
    pub edges_offset: OffsetMetadataTable,
    pub _reserved: [u8; 3968],
}

impl Default for NexoraFooter {
    fn default() -> Self {
        Self {
            name_table_offset: OffsetMetadataTable::default(),
            node_schema_offset: OffsetMetadataTable::default(),
            edge_schema_offset: OffsetMetadataTable::default(),
            schema_properties_offset: OffsetMetadataTable::default(),
            metadata_offset: OffsetMetadataTable::default(),
            indices_offset: OffsetMetadataTable::default(),
            nodes_offset: OffsetMetadataTable::default(),
            edges_offset: OffsetMetadataTable::default(),
            _reserved: [0u8; 3968],
        }
    }
}

impl NexoraFooter {
    pub fn deserialize(raw_footer_data: [u8; PAGE_SIZE]) -> Self {
        let mut offset = 0;

        // helper macro to grab slices safely
        macro_rules! take {
            ($len:expr) => {{
                let start = offset;
                let end = offset + $len;
                offset = end;
                &raw_footer_data[start..end]
            }};
        }

        fn parse_offset_table(bytes: &[u8]) -> OffsetMetadataTable {
            use std::convert::TryInto;
            let nb_total_items = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
            let base_chunk_offset = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
            OffsetMetadataTable {
                nb_total_items,
                base_chunk_offset,
            }
        }

        let name_table_offset = parse_offset_table(take!(16));
        let node_schema_offset = parse_offset_table(take!(16));
        let edge_schema_offset = parse_offset_table(take!(16));
        let schema_properties_offset = parse_offset_table(take!(16));
        let metadata_offset = parse_offset_table(take!(16));
        let indices_offset = parse_offset_table(take!(16));
        let nodes_offset = parse_offset_table(take!(16));
        let edges_offset = parse_offset_table(take!(16));

        let mut reserved = [0u8; 3968];
        reserved.copy_from_slice(take!(3968));

        Self {
            name_table_offset,
            node_schema_offset,
            edge_schema_offset,
            schema_properties_offset,
            metadata_offset,
            indices_offset,
            nodes_offset,
            edges_offset,
            _reserved: reserved,
        }
    }
}


const _: () = assert!(size_of::<NexoraFooter>() == PAGE_SIZE);

/// -------------------- OffsetItem --------------------
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct OffsetItem {
    pub id: u64,
    pub offset: u64,
}

impl Default for OffsetItem {
    fn default() -> Self {
        Self {
            id: 0,
            offset: INVALID_OFFSET,
        }
    }
}

const _: () = assert!(size_of::<OffsetItem>() == 16);

/// -------------------- OffsetTableChunk --------------------
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct OffsetTableChunk {
    pub nb_items: u8,
    pub _pad0: [u8; 7],
    pub previous_chunk: u64,
    pub next_chunk: u64,
    pub offset_items: [OffsetItem; 254],
    pub _reserved: [u8; 8],
}

impl Default for OffsetTableChunk {
    fn default() -> Self {
        Self {
            nb_items: 0,
            _pad0: [0u8; 7],
            previous_chunk: INVALID_OFFSET,
            next_chunk: INVALID_OFFSET,
            offset_items: [OffsetItem::default(); 254],
            _reserved: [0u8; 8],
        }
    }
}
const _: () = assert!(size_of::<OffsetTableChunk>() == PAGE_SIZE);


impl OffsetTableChunk {
    pub fn serialize(&self) -> [u8; PAGE_SIZE] {
        let mut buf: [u8; PAGE_SIZE] = [0u8; PAGE_SIZE];
        let mut offset = 0;

        macro_rules! write_slice {
            ($data:expr) => {{
                let len = $data.len();
                buf[offset..offset + len].copy_from_slice($data);
                offset += len;
            }};
        }

        buf[offset] = self.nb_items;
        offset += 1;

        write_slice!(&self._pad0);
        write_u64_le(self.previous_chunk, &mut buf[offset..offset + 8]);
        offset += 8;
        write_u64_le(self.next_chunk, &mut buf[offset..offset + 8]);
        offset += 8;

        for item in &self.offset_items {
            write_u64_le(item.id, &mut buf[offset..offset + 8]);
            offset += 8;
            write_u64_le(item.offset, &mut buf[offset..offset + 8]);
            offset += 8;
        }

        write_slice!(&self._reserved);

        assert_eq!(offset, PAGE_SIZE, "OffsetTableChunk serialization size mismatch");

        buf
    }

    pub fn deserialize(buf: &[u8; PAGE_SIZE]) -> Self {
        let mut offset = 0;

        // ---- nb_items ----
        let nb_items = buf[offset];
        offset += 1;

        // ---- pad0 ----
        let mut pad0 = [0u8; 7];
        pad0.copy_from_slice(&buf[offset..offset + 7]);
        offset += 7;

        // ---- previous_chunk ----
        let previous_chunk = read_u64_le(buf, offset).unwrap();
        offset += 8;

        // ---- next_chunk ----
        let next_chunk = read_u64_le(buf, offset).unwrap();
        offset += 8;

        // ---- offset_items ----
        let mut offset_items = [OffsetItem::default(); 254];
        for item in &mut offset_items {
            let id = read_u64_le(buf, offset).unwrap();
            offset += 8;

            let item_offset = read_u64_le(buf, offset).unwrap();
            offset += 8;

            *item = OffsetItem { id, offset: item_offset };
        }

        // ---- reserved ----
        let mut reserved = [0u8; 8];
        reserved.copy_from_slice(&buf[offset..offset + 8]);
        offset += 8;

        assert_eq!(
            offset,
            PAGE_SIZE,
            "OffsetTableChunk deserialization did not consume full buffer"
        );

        Self {
            nb_items,
            _pad0: pad0,
            previous_chunk,
            next_chunk,
            offset_items,
            _reserved: reserved,
        }
    }
}

/// -------------------- Name --------------------
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Name {
    pub id: u64,
    pub size: u8,
    pub value: [u8; PROPERTY_NAME_MAX_SIZE],
}

impl Default for Name {
    fn default() -> Self {
        Self {
            id: 0,
            size: 0,
            value: [0u8; PROPERTY_NAME_MAX_SIZE],
        }
    }
}
const _: () = assert!(size_of::<Name>() == 64);

/// -------------------- PropertyType --------------------
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyType {
    Int8 = 0,
    Int16,
    Int32,
    Int64,
    Float8,
    Float16,
    Float32,
    Float64,
    String32,
    String64,
    String512,
    Page,
    Bool,
    InvalidType,
}

impl Default for PropertyType {
    fn default() -> Self {
        PropertyType::InvalidType
    }
}

/// -------------------- PropertyDefinition --------------------
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PropertyDefinition {
    pub name_id: u64,
    pub r#type: u8,
    pub optional: u8,
    pub _reserved: [u8; 6],
}

impl Default for PropertyDefinition {
    fn default() -> Self {
        Self {
            name_id: 0,
            r#type: PropertyType::InvalidType as u8,
            optional: 0,
            _reserved: [0u8; 6],
        }
    }
}
const _: () = assert!(size_of::<PropertyDefinition>() == 16);

/// -------------------- NodeSchema --------------------
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NodeSchema {
    pub id: u64,
    pub property_count: u16,
    pub _pad: [u8; 6],
    pub properties: [u64; MAX_PROPERTIES_COUNT],
    pub _reserved: [u8; 48],
}

impl Default for NodeSchema {
    fn default() -> Self {
        Self {
            id: 0,
            property_count: 0,
            _pad: [0u8; 6],
            properties: [0u64; MAX_PROPERTIES_COUNT],
            _reserved: [0u8; 48],
        }
    }
}
const _: () = assert!(size_of::<NodeSchema>() == KB1);

/// -------------------- EdgeSchema --------------------
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct EdgeSchema {
    pub id: u64,
    pub property_count: u16,
    pub _pad: [u8; 6],
    pub properties: [u64; MAX_PROPERTIES_COUNT],
    pub _reserved: [u8; 48],
}

impl Default for EdgeSchema {
    fn default() -> Self {
        Self {
            id: 0,
            property_count: 0,
            _pad: [0u8; 6],
            properties: [0u64; MAX_PROPERTIES_COUNT],
            _reserved: [0u8; 48],
        }
    }
}
const _: () = assert!(size_of::<EdgeSchema>() == KB1);

/// -------------------- Node --------------------
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Node {
    pub id: u64,
    pub schema_id: u64,
    pub property_values: [u64; MAX_PROPERTIES_COUNT],
    pub _reserved: [u8; 48],
}

impl Default for Node {
    fn default() -> Self {
        Self {
            id: 0,
            schema_id: 0,
            property_values: [0u64; MAX_PROPERTIES_COUNT],
            _reserved: [0u8; 48],
        }
    }
}
const _: () = assert!(size_of::<Node>() == KB1);

/// -------------------- Edge --------------------
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Edge {
    pub id: u64,
    pub schema_id: u64,
    pub source_id: u64,
    pub destination_id: u64,
    pub property_values: [u64; MAX_PROPERTIES_COUNT],
    pub _reserved: [u8; 32],
}

impl Default for Edge {
    fn default() -> Self {
        Self {
            id: 0,
            schema_id: 0,
            source_id: 0,
            destination_id: 0,
            property_values: [0u64; MAX_PROPERTIES_COUNT],
            _reserved: [0u8; 32],
        }
    }
}
const _: () = assert!(size_of::<Edge>() == KB1);

/// -------------------- NexoraFile --------------------
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NexoraFile {
    pub header: NexoraHeader,
    pub footer: NexoraFooter,
}

impl Default for NexoraFile {
    fn default() -> Self {
        // Header: offset 0
        let mut header = NexoraHeader {
            footer_offset: INVALID_OFFSET, // footer starts after header
            created_unix: 0,
            magic: FILE_HEADER_MAGIC,
            version: 0,
            flags: 0,
            _reserved: [0u8; 4070],
        };

        // Compute offsets for other sections (all defaults start sequentially)
        let mut offset = PAGE_SIZE as u64; // immediately after header

        let name_table_offset = OffsetMetadataTable {
            nb_total_items: 0,
            base_chunk_offset: offset,
        };
        offset += PAGE_SIZE as u64;

        let node_schema_offset = OffsetMetadataTable {
            nb_total_items: 0,
            base_chunk_offset: offset,
        };
        offset += PAGE_SIZE as u64;

        let edge_schema_offset = OffsetMetadataTable {
            nb_total_items: 0,
            base_chunk_offset: offset,
        };
        offset += PAGE_SIZE as u64;

        let schema_properties_offset = OffsetMetadataTable {
            nb_total_items: 0,
            base_chunk_offset: offset,
        };
        offset += PAGE_SIZE as u64;

        let metadata_offset = OffsetMetadataTable {
            nb_total_items: 0,
            base_chunk_offset: offset,
        };
        offset += PAGE_SIZE as u64;

        let indices_offset = OffsetMetadataTable {
            nb_total_items: 0,
            base_chunk_offset: offset,
        };
        offset += PAGE_SIZE as u64;

        let nodes_offset = OffsetMetadataTable {
            nb_total_items: 0,
            base_chunk_offset: offset,
        };
        offset += PAGE_SIZE as u64;

        let edges_offset = OffsetMetadataTable {
            nb_total_items: 0,
            base_chunk_offset: offset,
        };
        offset += PAGE_SIZE as u64;

        header.footer_offset = offset;

        let footer = NexoraFooter {
            name_table_offset,
            node_schema_offset,
            edge_schema_offset,
            schema_properties_offset,
            metadata_offset,
            indices_offset,
            nodes_offset,
            edges_offset,
            _reserved: [0u8; 3968],
        };

        Self { header, footer }
    }

}
const _: () = assert!(size_of::<NexoraFile>() == PAGE_SIZE * 2);


impl NexoraFile {
    pub fn serialize(&self) -> [u8; PAGE_SIZE * 10] {
        let mut buf: [u8; PAGE_SIZE * 10] = [0u8; PAGE_SIZE * 10];
        let mut offset = 0;

        write_u64_le(self.header.footer_offset, &mut buf[offset..offset + 8]);
        offset += 8;
        write_u64_le(self.header.created_unix, &mut buf[offset..offset + 8]);
        offset += 8;
        write_bytes(&self.header.magic, &mut buf[offset..offset + self.header.magic.len()]);
        offset += self.header.magic.len();
        write_u16_le(self.header.version, &mut buf[offset..offset + 2]);
        offset += 2;
        write_u16_le(self.header.flags, &mut buf[offset..offset + 2]);
        offset += 2;
        write_bytes(&self.header._reserved, &mut buf[offset..offset + self.header._reserved.len()]);
        offset += self.header._reserved.len();
        
        for _ in 0..8 {
            let chunk = OffsetTableChunk::default().serialize();
            buf[offset..offset + PAGE_SIZE].copy_from_slice(&chunk);
            offset += PAGE_SIZE;
        }

        macro_rules! write_offset_table {
            ($ot:expr) => {
                write_u64_le($ot.nb_total_items, &mut buf[offset..offset + 8]);
                offset += 8;
                write_u64_le($ot.base_chunk_offset, &mut buf[offset..offset + 8]);
                offset += 8;
            };
        }

        write_offset_table!(self.footer.name_table_offset);
        write_offset_table!(self.footer.node_schema_offset);
        write_offset_table!(self.footer.edge_schema_offset);
        write_offset_table!(self.footer.schema_properties_offset);
        write_offset_table!(self.footer.metadata_offset);
        write_offset_table!(self.footer.indices_offset);
        write_offset_table!(self.footer.nodes_offset);
        write_offset_table!(self.footer.edges_offset);

        write_bytes(&self.footer._reserved, &mut buf[offset..offset + self.footer._reserved.len()]);
        offset += self.footer._reserved.len();

        assert_eq!(offset, PAGE_SIZE * 10, "NexoraFile serialization size mismatch");

        buf
    }
}
