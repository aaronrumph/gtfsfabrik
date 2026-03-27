#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TripID {
    pub id: String,
}

impl TripID {
    pub fn new(id: &str) -> Self {
        Self { id: id.to_string() }
    }
}
