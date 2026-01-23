use ulid::Ulid;

#[derive(Debug, Clone)]
pub struct Host(pub String);

#[derive(Debug, Clone)]
pub struct RunId(pub(crate) String);

impl std::fmt::Display for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl RunId {
    pub fn new() -> Self {
        Self(Ulid::new().to_string())
    }

    pub fn from_string(s: String) -> Self {
        Self(s)
    }
}

impl Default for RunId {
    fn default() -> Self {
        Self::new()
    }
}
