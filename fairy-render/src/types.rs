pub struct Asset {}

pub trait Content {}

pub struct Payload {
    scripts: Vec<Asset>,
    stylesheets: Vec<Asset>,
    content: Box<dyn Content>,
}
