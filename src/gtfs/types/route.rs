#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RouteID {
    pub id: String,
}

impl RouteID {
    pub fn new(id: &str) -> Self {
        Self { id: id.to_string() }
    }
}
