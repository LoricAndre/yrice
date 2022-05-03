#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub value: String,
}

impl Variable {
    pub fn new(name: String, value: String) -> Variable {
        Variable { name, value }
    }
}
