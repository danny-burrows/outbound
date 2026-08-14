#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Item {
    pub(crate) position: (i64, i64),
    pub(crate) id: String,
}

impl Item {
    pub(crate) const fn new(id: String, position: (i64, i64)) -> Self {
        Self { position, id }
    }
}
