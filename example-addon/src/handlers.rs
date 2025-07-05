use std::convert::TryInto;

use stremio_addon_sdk::builder::{Builder, BuilderWithHandlers, Handler};
use stremio_addon_sdk::scaffold::Scaffold;
use stremio_addon_sdk::stremio_core;
use stremio_addon_sdk::stremio_core::runtime::EnvError;
use stremio_core::types::addon::{Manifest, ResourcePath, ResourceResponse};
use stremio_core::types::resource::{Stream, StreamSource, MetaItemPreview};
use std::sync::Arc;

struct StreamsHandler;

#[async_trait::async_trait]
impl Handler for StreamsHandler {
    async fn reply(&self, resource: &ResourcePath) -> Result<ResourceResponse, EnvError> {
        let mut streams = vec![];
        if resource.r#type.eq("movie") && resource.id.eq("tt1254207") {
            streams.push(Stream {
                name: Some("[RUST TEST] Big buck bunny".into()),
                description: Some("[RUST TEST] Big buck bunny".into()),
                source: StreamSource::Url {
                    url: "http://distribution.bbb3d.renderfarming.net/video/mp4/bbb_sunflower_1080p_30fps_normal.mp4".try_into().unwrap()
                },
                behavior_hints: Default::default(),
                thumbnail: None,
                subtitles: vec![],
            });
        }
        dbg!(&streams);
        
        Ok(ResourceResponse::Streams {streams})
    }
}

struct CatalogHandler;

#[async_trait::async_trait]
impl Handler for CatalogHandler {
    async fn reply(&self, _: &ResourcePath) -> Result<ResourceResponse, EnvError> {
        Ok(ResourceResponse::Metas { metas: vec![
            MetaItemPreview {
                id: "tt1254207".into(),
                name: "Big buck Bunny".into(),
                poster: Some("https://image.tmdb.org/t/p/w600_and_h900_bestv2/uVEFQvFMMsg4e6yb03xOfVsDz4o.jpg".try_into().unwrap()),
                description: Some("addon test".into()),
                r#type: "movie".into(),
                ..Scaffold::default_item_preview()
            }
        ]})
    }
}

pub fn build(manifest: Manifest) -> BuilderWithHandlers {
    let catalog_handler = Arc::new(CatalogHandler);
    let streams_handler = Arc::new(StreamsHandler);

     Builder::new(manifest)
        .define_catalog_handler(catalog_handler)
        .define_stream_handler(streams_handler)
        .build()
}