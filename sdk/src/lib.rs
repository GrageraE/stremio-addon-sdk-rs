pub mod builder;
pub mod server;
pub mod landing_template;
pub mod router;
pub mod helpers;
// pub mod export {
//     pub mod serverless {
//         pub mod now {
//             pub use now_lambda::Request;
//             pub use now_lambda::IntoResponse;
//             pub use now_lambda::error::NowError;
//         }
//     }
// }

/// Functions to build defaults of some types in Stremio-core
pub mod scaffold;
/// Types extracted from old versions of Stremio-core (commit = 3d3e1008c), useful for the SDK
pub mod util;

// Re-export stremio-core
pub use stremio_core;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use stremio_core::{runtime::EnvError, types::addon::*};

    struct TestHandler;

    #[async_trait::async_trait]
    impl builder::Handler for TestHandler {
        async fn reply(&self, _resource: &ResourcePath) -> Result<ResourceResponse, EnvError> {
            Ok(ResourceResponse::Streams { streams: vec![] })
        }
    }

    #[test]
    #[should_panic]
    fn builder_panics_if_no_handlers_attached() {
        builder::Builder::new(scaffold::Scaffold::default_manifest()).build();
    }

    #[test]
    #[should_panic]
    fn builder_panics_if_no_resources_defined_for_handler() {
        let handler = TestHandler;

        builder::Builder::new(scaffold::Scaffold::default_manifest())
            .define_stream_handler(Arc::new(handler))
            .build();
    }

    #[test]
    #[should_panic]
    fn builder_panics_if_no_handlers_defined_for_resource() {
        let handler = TestHandler;
        let manifest = Manifest {
            resources: vec![ManifestResource::Short("meta".into()), ManifestResource::Short("stream".into())],
            ..scaffold::Scaffold::default_manifest()
        };
        builder::Builder::new(manifest)
            .define_stream_handler(Arc::new(handler))
            .build();
    }
}