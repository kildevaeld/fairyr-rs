use dale_http::error::Error;
use fairy_core::Environ;
use relative_path::RelativePathBuf;
use std::path::PathBuf;

pub struct RenderRequest {
    pub scripts: Vec<String>,
    pub links: Vec<String>,
    pub content: Option<String>,
}

pub trait Template: Send + Sync {
    type Error;
    fn render(&self, request: RenderRequest) -> Result<Vec<u8>, Self::Error>;
}

impl<F, E> Template for F
where
    F: Fn(RenderRequest) -> Result<Vec<u8>, E> + Send + Sync,
{
    type Error = E;

    fn render(&self, request: RenderRequest) -> Result<Vec<u8>, Self::Error> {
        (self)(request)
    }
}

pub type TemplateBox = Box<dyn Template<Error = Error>>;

impl Template for TemplateBox {
    type Error = Error;
    fn render(&self, request: RenderRequest) -> Result<Vec<u8>, Self::Error> {
        (&**self).render(request)
    }
}

struct BoxedTemplate<T> {
    template: T,
}

impl<T> Template for BoxedTemplate<T>
where
    T: Template,
    T::Error: std::error::Error + Send + Sync + 'static,
{
    type Error = Error;
    fn render(&self, request: RenderRequest) -> Result<Vec<u8>, Self::Error> {
        self.template.render(request).map_err(Error::new)
    }
}

// #[derive(Debug)]
pub struct Options {
    pub root: PathBuf,
    pub entry: RelativePathBuf,
    pub env: Environ,
    pub public: RelativePathBuf,
    pub template: TemplateBox,
}

impl Options {
    pub fn build(root: impl Into<PathBuf>) -> OptionsBuilder {
        OptionsBuilder::new(root)
    }
}

pub struct OptionsBuilder {
    root: PathBuf,
    entry: Option<RelativePathBuf>,
    env: Environ,
    public: Option<RelativePathBuf>,
    template: Option<TemplateBox>,
}

impl OptionsBuilder {
    pub fn new(root: impl Into<PathBuf>) -> OptionsBuilder {
        OptionsBuilder {
            root: root.into(),
            entry: None,
            env: Environ::default(),
            template: None,
            public: None,
        }
    }

    pub fn entry(mut self, entry: impl Into<RelativePathBuf>) -> Self {
        self.entry = Some(entry.into());
        self
    }

    pub fn template<T>(mut self, template: T) -> Self
    where
        T: Template + 'static,
        T::Error: std::error::Error + Send + Sync + 'static,
    {
        self.template = Some(Box::new(BoxedTemplate { template }));
        self
    }

    pub fn with_env(mut self, name: impl ToString, value: impl ToString) -> Self {
        self.env.insert(name.to_string(), value.to_string());
        self
    }

    pub fn public(mut self, public: impl Into<RelativePathBuf>) -> Self {
        self.public = Some(public.into());
        self
    }

    pub fn build(self) -> Result<Options, std::convert::Infallible> {
        let template = self.template.unwrap();
        let entry = self.entry.unwrap();

        let root = self.root.canonicalize().expect("invalid path");

        Ok(Options {
            root,
            entry,
            env: self.env,
            template,
            public: self
                .public
                .unwrap_or_else(|| RelativePathBuf::from("./public")),
        })
    }
}
