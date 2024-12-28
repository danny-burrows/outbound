#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Item {
    pub position: (i64, i64),
    pub id: String,
}

impl Item {
    pub fn new(id: String, position: (i64, i64)) -> Item {
        Item { id, position }
    }
}
