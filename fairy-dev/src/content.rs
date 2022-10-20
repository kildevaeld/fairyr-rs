use swc_common::sync::Lrc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Content(Lrc<Vec<u8>>);

impl Content {
    pub fn new(bytes: Vec<u8>) -> Content {
        Content(Lrc::new(bytes))
    }

    pub fn to_bytes(self) -> Vec<u8> {
        self.0.as_ref().clone()
    }

    pub fn to_string(self) -> anyhow::Result<String> {
        Ok(String::from_utf8(self.to_bytes())?)
    }
}

impl std::ops::Deref for Content {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
