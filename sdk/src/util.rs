use stremio_core::types::addon::{ResourcePath, ExtraValue};
use percent_encoding::{percent_decode, utf8_percent_encode, PATH_SEGMENT_ENCODE_SET};
use url::form_urlencoded;
use std::fmt::Display;
use std::str::FromStr;

/// The old `ParseResourceErr` from Stremio
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseResourceErr {
    WrongPrefix,
    WrongSuffix,
    InvalidLength(usize),
    DecodeErr,
}

impl Display for ParseResourceErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseResourceErr::WrongPrefix => write!(f, "ParseResourceErr: WrongPreffix"),
            ParseResourceErr::WrongSuffix => write!(f, "ParseResourceErr: WrongSuffix"),
            ParseResourceErr::InvalidLength(len) => write!(f, "ParseResourceErr: InvalidLenght: {}", len),
            ParseResourceErr::DecodeErr => write!(f, "ParseResourceErr: DecodeErr"),
        }
    }
}

impl std::error::Error for ParseResourceErr {}

/// Wraps a [ResourcePath] to get back some useful traits from the older `ResourceRef`
#[derive(Debug)]
pub struct ResourcePathWrapper {
    inner: ResourcePath
}

impl ResourcePathWrapper {
    /// Extracted from Stremio-core (commit = 3d3e1008c). Used in the [FromStr] implementation
    pub fn parse_component(s: &str) -> Result<String, ParseResourceErr> {
        Ok(percent_decode(s.as_bytes())
            .decode_utf8()
            .map_err(|_| ParseResourceErr::DecodeErr)?
            .to_string())
    }    
}

impl From<ResourcePath> for ResourcePathWrapper {
    /// Wrap a [ResourcePath]
    fn from(value: ResourcePath) -> Self {
        Self {
            inner: value
        }
    }
}

impl From<ResourcePathWrapper> for ResourcePath {
    /// Extract the [ResourcePath] inside
    fn from(value: ResourcePathWrapper) -> Self {
        value.inner
    }
}

impl Display for ResourcePathWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "/{}/{}/{}",
            &utf8_percent_encode(&self.inner.resource, PATH_SEGMENT_ENCODE_SET),
            &utf8_percent_encode(&self.inner.r#type, PATH_SEGMENT_ENCODE_SET),
            &utf8_percent_encode(&self.inner.id, PATH_SEGMENT_ENCODE_SET)
        )?;
        if !self.inner.extra.is_empty() {
            let mut extra_encoded = form_urlencoded::Serializer::new(String::new());
            for extra_value in self.inner.extra.iter() {
                extra_encoded.append_pair(&extra_value.name, &extra_value.value);
            }
            write!(f, "/{}", &extra_encoded.finish())?;
        }
        write!(f, ".json")
    }
}

impl FromStr for ResourcePathWrapper {
    type Err = ParseResourceErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with('/') {
            return Err(ParseResourceErr::WrongPrefix);
        }
        if !s.ends_with(".json") {
            return Err(ParseResourceErr::WrongSuffix);
        }
        let components: Vec<&str> = s.trim_end_matches(".json").split('/').skip(1).collect();
        match components.len() {
            3 | 4 => Ok(ResourcePath {
                resource: Self::parse_component(components[0])?,
                r#type: Self::parse_component(components[1])?,
                id: Self::parse_component(components[2])?,
                extra: components
                    .get(3)
                    .map(|e| form_urlencoded::parse(e.as_bytes()).into_owned()
                        .map(|(name, value)| ExtraValue {
                            name,
                            value
                        }).collect())
                    .unwrap_or_default(),
            }.into()),
            i => Err(ParseResourceErr::InvalidLength(i)),
        }    
    }
}
