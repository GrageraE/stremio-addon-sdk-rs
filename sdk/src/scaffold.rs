use std::iter::FromIterator;

use stremio_core::types::addon::Manifest;
use semver::Version;
use serde_json;

pub struct Scaffold;
impl Scaffold {
    pub fn default_manifest() -> Manifest {
        Manifest {
            id: String::default(),
            name: String::default(),
            version: Version::new(0, 0, 1),
            resources: Vec::default(),
            types: Vec::default(),
            catalogs: Vec::default(),
            contact_email: Option::default(),
            background: Option::default(),
            logo: Option::default(),
            id_prefixes: Option::default(),
            description: Option::default(),
            addon_catalogs: Vec::default(),
            behavior_hints: serde_json::map::Map::default() // Default::default()
        }
    }

    pub fn set_behavior_hints(adult: bool, p2p: bool, configurable: bool, configuration_required: bool)
        -> serde_json::map::Map<String, serde_json::Value> 
    {
        serde_json::map::Map::from_iter([("adult".into(), adult.into()),
                                        ("p2p".into(), p2p.into()),
                                        ("configurable".into(), configurable.into()),
                                        ("configurationRequired".into(), configuration_required.into())])  
    }
}