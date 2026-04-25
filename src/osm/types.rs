// OSM PBF file types and structs

use crate::utils::errors::OSMErorr;

pub type Bytes = Vec<u8>;

pub enum BlobType {
    OSMHeader,
    OSMData,
    Other(String),
}

/// 'Type' field (required) replaced with kind
/// First four bytes: Length of BlobHeader
/// BlobHeader type: first in file is 'OSMHeader', others 'OSMData'
/// Next BlobHeader
/// Finaly Blob
pub struct OSMBlock {
    pub length_of_header: u32,
    pub blob_header_type: BlobType,
    pub blob_header: BlobHeader,
    pub blob: Blob,
}

/// A struct representing a BlobHeader.
pub struct BlobHeader {
    pub kind: String, // Can't call it type like official format
    pub index_data: Bytes,
    pub data_size: i32,
}

// TODO: docs
pub struct Blob {
    // "The uncompressed length of a Blob should be less than 16 MiB (16 * 1024 * 1024
    // Bytes) and must be less than 32 MiB" https://wiki.openstreetmap.org/wiki/PBF_Format#Low_level_encoding
    pub raw_size: Option<i32>,
    // must contain at least one of the BlobData types
    pub data: BlobData,
}

/// The various forms OSM data can take depending on compression. Mutually exclusive (one PBF file
/// must contain ONLY one)
pub enum BlobData {
    Raw(Bytes),
    Zlib(Bytes),
    Lzma(Bytes),
    ObseleteBZip2(Bytes),
    Lz4(Bytes),
    Zstd(Bytes),
}

impl Blob {
    /// Constructor for OSM PBF blob
    pub fn new(raw_size: Option<i32>, data: BlobData) -> Self {
        Self { raw_size, data }
    }
}

// TODO: docs
pub struct HeaderBlock {
    header_bbox: HeaderBBox,

    required_features: Vec<String>,
    optional_features: Vec<String>,

    writing_program: Option<String>,
    source: Option<String>,

    osmosis_replication_timestamp: Option<i64>,
    osmosis_replication_sequence_number: Option<i64>,
    osmosis_replication_base_url: Option<String>,
}

pub struct HeaderBBox {
    left: i64,
    right: i64,
    top: i64,
    bottom: i64,
}

/// Metadata about the object
pub struct Info {
    version: Option<i32>,
    timestamp: Option<i64>,
    changeset: Option<i64>,
    uid: Option<i32>,
    user_sid: Option<u32>,

    visible: Option<Bool>,
}

pub struct Node {
    // TODO: node struct impl
    id: i64,
    keys: Vec<u32>,
    vals: Vec<u32>,
    info: Option<Info>,
    lat: i64,
    lon: i64,
}

pub struct DenseInfo {
    // TODO: Dense info
}
pub struct DenseNodes {
    id: Vec<i64>,
    denseinfo: Option<DenseInfo>,
    // TODO: finish
}

pub struct Way {
    id: u64,
    keys: Vec<u32>,
    vals: Vec<u32>,
    info: Option<Info>,
    refs: Vec<i64>,
    lat: Vec<i64>,
    lon: Vec<i64>,
}

pub struct Relation {
    id: u64,
    keys: Vec<u32>,
    vals: Vec<u32>,

    info: Option<Info>,

    // Parallel Aarrays
    roles_sid: Vec<i32>,
    memids: Vec<i64>,
    types: Vec<MemberTypes>,
}

pub struct ChangeSet {
    // TODO: figure out what this even is and what to do with it
}

/// 6 kinds of primitive groups so using enum
pub enum PrimitiveGroup {
    Nodes(Vec<Node>),
    DenseNodes(DenseNodes),
    Ways(Vec<Way>),
    Relations(Vec<Relation>),
    ChangeSets(Vec<ChangeSet>),
}

pub struct PrimitiveBlock {
    pub string_table: StringTable,
    pub primitive_groups: Vec<PrimitiveGroup>,
    pub granularity: i32,
    // TODO: Figure out types for these
    // pub lat_offset:
    // pub lon_offset:
}

pub struct StringTable {
    // all strings, etc are stored in a string table
    pub strings: Vec<Bytes>,
}
