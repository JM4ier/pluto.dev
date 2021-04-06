#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PageKind {
    Post,
    Tag,
}

impl PageKind {
    fn name(&self) -> &str {
        match self {
            Self::Post => "post",
            Self::Tag => "tag",
        }
    }
    pub fn url_of(&self, item: &str) -> String {
        format!("/{}/{}.html", self.name(), item)
    }
    pub fn path_of(&self, item: &str) -> String {
        format!("html{}", self.url_of(item))
    }
    pub fn dir(&self) -> String {
        format!("html/{}/", self.name())
    }
    pub fn kinds() -> Vec<Self> {
        vec![Self::Post, Self::Tag]
    }
}

pub trait Linkable {
    fn link(&self) -> String;
}
