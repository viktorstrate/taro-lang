#[derive(Debug, Clone)]
pub enum Comment<'a> {
    Line(&'a str),
    Block(&'a str),
}
