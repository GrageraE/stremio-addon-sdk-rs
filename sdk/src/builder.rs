use stremio_core::types::addon::{Manifest, ManifestResource, ResourcePath, ResourceResponse};
use stremio_core::runtime::EnvError;
use crate::util::ResourcePathWrapper;
use std::str::FromStr;
use std::sync::Arc;

// type Handler = dyn Fn(&ResourcePath) -> TryEnvFuture<ResourceResponse> + Send + Sync + 'static;
// type RouterFut = Box<dyn Future<Item=ResourceResponse, Error=RouterErr>>;
// type RouterFut = Pin<Box<dyn Future<Output = Result<ResourceResponse, EnvError>> + Send>>;

// #[derive(Debug)]
// pub enum RouterErr {
//     NotFound,
//     Handler(Box<dyn Error>),
//     Parse(ParseResourceErr)
// }
// impl Error for RouterErr {}
// impl std::fmt::Display for RouterErr {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "RouterError")
//     }
// }

/// A handler
#[async_trait::async_trait]
pub trait Handler {
    /// An async function which replies to Stremio
    async fn reply(&self, resource: &ResourcePath) -> Result<ResourceResponse, EnvError>;
}

/// Async version of the trait [AddonTransport]
#[allow(async_fn_in_trait)]
pub trait AsyncAddonTransport {
    async fn resource(&self, path: &ResourcePath) -> Result<ResourceResponse, EnvError>;
    async fn manifest(&self) -> Manifest;
}

/// Sendable [Handler]
type HandlerSS = dyn Handler + Send + Sync;

#[allow(async_fn_in_trait)]
pub trait AddonRouter {
    fn get_manifest(&self) -> &Manifest;
    async fn route(&self, path: &str) -> Result<ResourceResponse, EnvError>;
}

// Base: just serving the manifest
#[derive(Clone)]
pub struct AddonBase {
    manifest: Manifest
}
impl AddonRouter for AddonBase {
    fn get_manifest(&self) -> &Manifest {
        &self.manifest
    }

    async fn route(&self, _: &str) -> Result<ResourceResponse, EnvError> {
        Err(EnvError::Other("Not found".into()))
    }
}

// WithHandler: attach a handler
#[derive(Clone)]
pub struct WithHandler<T> 
where T: AddonRouter {
    base: T,
    pub match_prefix: String,
    handler: Arc<HandlerSS>
}

impl<T> AddonRouter for WithHandler<T> 
where T: AddonRouter {
    fn get_manifest(&self) -> &Manifest {
        self.base.get_manifest()
    }

    async fn route(&self, path: &str) -> Result<ResourceResponse, EnvError> {
        if path.starts_with(&self.match_prefix) {
            let res = match ResourcePathWrapper::from_str(path) {
                Ok(r) => r.into(),
                Err(parse_error) 
                    => {return Err(EnvError::Other(format!("Parse Error: {}", parse_error)));}
            };
            return self.handler.reply(&res).await;
        }
        self.base.route(&path).await
    }
}

impl<T> AsyncAddonTransport for WithHandler<T> 
where T: AddonRouter {
    async fn manifest(&self) -> Manifest {
        self.get_manifest().clone()
    }
    async fn resource(&self, req: &ResourcePath) -> Result<ResourceResponse, EnvError> {
        self.route(&ResourcePathWrapper::from(req.clone()).to_string()).await
    }
}


// Builder: constructs a new builder that implements WithHandler
pub struct Builder;
impl Builder {
    pub fn new(manifest: Manifest) -> BuilderWithHandlers {
        // typestate, two different types for the builder, when we attach handlers
        // so that we cannot build before that
        BuilderWithHandlers {
            handlers: vec![],
            base: AddonBase { manifest }
        }
    }
}

/// BuilderWithHandlers: builder with handlers attached
#[derive(Clone)]
pub struct BuilderWithHandlers {
    base: AddonBase,
    pub handlers: Vec<WithHandler<AddonBase>>
}
impl BuilderWithHandlers {
    fn handle_resource(&mut self, resource_name: &str, handler: Arc<HandlerSS>) -> &mut Self 
    {
        if self.handlers.iter().any(|h| self.prefix_to_name(&h.match_prefix) == resource_name) {
            panic!("handler for resource {} is already defined!", resource_name);
        }
        self.handlers.push(WithHandler {
            base: self.base.clone(),
            match_prefix: format!("/{}/", resource_name),
            handler
        });
        self
    }
    pub fn define_stream_handler(&mut self, handler: Arc<HandlerSS>) -> &mut Self 
    {
        self.handle_resource("stream", handler)
    }
    pub fn define_meta_handler(&mut self, handler: Arc<HandlerSS>) -> &mut Self 
    {
        self.handle_resource("meta", handler)
    }
    pub fn define_catalog_handler(&mut self, handler: Arc<HandlerSS>) -> &mut Self {
        self.handle_resource("catalog", handler)
    }
    pub fn define_subtitles_handler(&mut self, handler: Arc<HandlerSS>) -> &mut Self 
    {
        self.handle_resource("subtitles", handler)
    }
    pub async fn handle(&self, path: &str) -> Option<ResourceResponse> {
        // get requested resource
        let resource = match ResourcePathWrapper::from_str(path) {
            Ok(r) => r.into(),
            Err(_) => return None
        };
        dbg!(&resource);

        // find correct handler for this resource
        let handler = match self.handlers.iter().find(|&item| path.starts_with(&item.match_prefix)) {
            Some(x) => x,
            _ => return None
        };
        
        // execute the handler
        let resource_response = match handler.resource(&resource).await {
            Ok(r) => r,
            Err(_) => return None
        };

        Some(resource_response)
    }
    fn prefix_to_name(&self, prefix: &String) -> String {
        prefix.replace("/", "")
    }
    fn validate(&self) -> Vec<String> {
        let mut errors: Vec<String> = Vec::new();
        let manifest = self.base.get_manifest();

        if self.handlers.len() == 0 {
            errors.push("at least one handler must be defined".into());
        }
        
        // get all handlers that are declared in the maifest
        let mut handlers_in_manifest: Vec<String> = Vec::new();
        if manifest.catalogs.len() > 0 {
            handlers_in_manifest.push("catalog".into());
        }
        for resource in &manifest.resources {
            // NOTE: resource.name() should probably be public in stremio-core, making this code unnecessary
            match resource {
                ManifestResource::Short(n) => handlers_in_manifest.push(n.to_string()),
                ManifestResource::Full { name, .. } => handlers_in_manifest.push(name.to_string()),
            }
        }
        
        // check if defined handlers are also specified in the manifest
        for defined_handler in &self.handlers {
            if !handlers_in_manifest.iter().any(|r| r.to_string() == self.prefix_to_name(&defined_handler.match_prefix)) {
                if defined_handler.match_prefix == "/catalog/" {
                    errors.push("manifest.catalogs is empty, catalog handler will never be called".into());
                }
                else {
                    errors.push(format!("manifest.resources does not contain: {}", self.prefix_to_name(&defined_handler.match_prefix)));
                }
            }
        }

        // check if handlers that are specified in the manifest are also defined
        for handler in handlers_in_manifest {
            if !self.handlers.iter().any(|r| handler == self.prefix_to_name(&r.match_prefix)) {
                errors.push(format!("manifest definition requires handler for {}, but it is not provided", handler));
            }
        }

        return errors;
        
    }
    pub fn build(&self) -> Self {
        let errors = self.validate();
        if errors.len() > 0 {
            panic!("\n--failed to build addon interface-- \n{}", errors.join("\n"));
        }
        self.clone()
    }
}
