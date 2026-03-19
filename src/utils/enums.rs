use clap::ValueEnum;

#[derive(Debug, Clone, ValueEnum)]
pub enum GeoScope {
    Place,
    County,
    Msa,
    Csa,
}
