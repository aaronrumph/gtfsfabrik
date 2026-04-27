// OSM PBF file types and structs

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
    ObsoleteBZip2(Bytes),
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
    pub bbox: Option<HeaderBBox>,

    pub required_features: Vec<String>,
    pub optional_features: Vec<String>,

    pub writing_program: Option<String>,
    pub source: Option<String>,

    pub osmosis_replication_timestamp: Option<i64>,
    pub osmosis_replication_sequence_number: Option<i64>,
    pub osmosis_replication_base_url: Option<String>,
}

pub struct HeaderBBox {
    pub left: i64,
    pub right: i64,
    pub top: i64,
    pub bottom: i64,
}

/// Metadata about the object
pub struct Info {
    pub version: Option<i32>,
    pub timestamp: Option<i64>,
    pub changeset: Option<i64>,
    pub uid: Option<i32>,
    pub user_sid: Option<u32>,

    pub visible: Option<bool>,
}

pub struct Node {
    pub id: i64,
    pub keys: Vec<u32>,
    pub vals: Vec<u32>,
    pub info: Option<Info>,
    pub lat: i64,
    pub lon: i64,
}

pub struct DenseInfo {
    pub version: Vec<i32>,
    pub timestamp: Vec<i64>,
    pub changeset: Vec<i64>,
    pub uid: Vec<i32>,
    pub user_sid: Vec<i32>,
    pub visible: Vec<bool>,
}

pub struct DenseNodes {
    pub id: Vec<i64>,
    pub denseinfo: Option<DenseInfo>,
    pub lat: Vec<i64>,
    pub lon: Vec<i64>,

    pub keys_vals: Vec<i32>,
}

pub struct Way {
    pub id: i64,
    pub keys: Vec<u32>,
    pub vals: Vec<u32>,
    pub info: Option<Info>,
    pub refs: Vec<i64>,
    pub lat: Vec<i64>,
    pub lon: Vec<i64>,
}

// MemberType enum used in Relation!
pub enum MemberType {
    Node,
    Way,
    Relation,
}

pub struct Relation {
    pub id: i64,
    pub keys: Vec<u32>,
    pub vals: Vec<u32>,

    pub info: Option<Info>,

    // Parallel Aarrays
    pub roles_sid: Vec<i32>,
    pub memids: Vec<i64>,
    pub types: Vec<MemberType>,
}

pub struct ChangeSet {
    pub id: i64,
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
    pub granularity: Option<i32>, // default = 100

    pub lat_offset: Option<i64>, // default = 0
    pub lon_offset: Option<i64>, // default = 0

    pub date_granularity: Option<i32>, // default = 1000
}

pub struct StringTable {
    // all strings, etc are stored in a string table
    pub strings: Vec<Bytes>,
}
