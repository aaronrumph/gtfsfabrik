#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RouteID {
    id: String,
}

impl RouteID {
    pub fn new(id: &str) -> Self {
        Self { id: id.to_string() }
    }
}

