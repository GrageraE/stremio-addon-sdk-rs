use std::convert::TryInto;

use stremio_addon_sdk::builder::{Builder, BuilderWithHandlers};
use stremio_addon_sdk::scaffold::Scaffold;
use stremio_addon_sdk::stremio_core;
use stremio_core::runtime::TryEnvFuture;
use stremio_core::types::addon::{Manifest, ResourcePath, ResourceResponse};
use stremio_core::types::resource::{Stream, StreamSource, MetaItemPreview};
use futures::future;

fn handle_stream(resource: &ResourcePath) -> TryEnvFuture<ResourceResponse> {
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
    
    return Box::pin(future::ok(ResourceResponse::Streams {streams}));
}

fn handle_catalog(_resource: &ResourcePath) -> TryEnvFuture<ResourceResponse> {
    Box::pin(future::ok(ResourceResponse::Metas {metas: vec![
        MetaItemPreview {
            id: "tt1254207".into(),
            name: "Big buck Bunny".into(),
            poster: Some("https://image.tmdb.org/t/p/w600_and_h900_bestv2/uVEFQvFMMsg4e6yb03xOfVsDz4o.jpg".try_into().unwrap()),
            description: Some("addon test".into()),
            r#type: "movie".into(),
            ..Scaffold::default_item_preview()
        }
    ]}))
}

pub fn build(manifest: Manifest) -> BuilderWithHandlers {
     Builder::new(manifest)
        .define_catalog_handler(handle_catalog)
        .define_stream_handler(handle_stream)
        .build()
}