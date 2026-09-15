use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error(transparent)]
    Xml(#[from] quick_xml::Error),

    #[error("attribute: {0}")]
    Attr(#[from] quick_xml::events::attributes::AttrError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),

    #[error(transparent)]
    FromUtf8(#[from] std::string::FromUtf8Error),

    #[error("xslt: {0}")]
    Xslt(String),

    #[error("rdf: {0}")]
    Rdf(String),

    #[error("index: {0}")]
    Index(String),
}

impl From<oxrdfio::RdfParseError> for Error {
    fn from(e: oxrdfio::RdfParseError) -> Self {
        Error::Rdf(e.to_string())
    }
}

impl From<oxiri::IriParseError> for Error {
    fn from(e: oxiri::IriParseError) -> Self {
        Error::Rdf(e.to_string())
    }
}
