use crate::types::ManifestChunk;
use dashmap::DashMap;
use rolldown_common::{Output, OutputChunk};
use rolldown_plugin::{HookNoopReturn, Plugin, SharedPluginContext};
use rustc_hash::FxBuildHasher;
use std::borrow::Cow;
use std::future::Future;
use std::rc::Rc;

#[derive(Debug)]
pub struct ManifestPlugin {
  pub manifest: DashMap<String, ManifestChunk, FxBuildHasher>,
  pub output_count: Rc<usize>,
}

impl ManifestPlugin {
  pub fn new() -> Self {
    Self { manifest: DashMap::default(), output_count: Rc::new(0usize) }
  }
}

impl Plugin for ManifestPlugin {
  fn name(&self) -> Cow<'static, str> {
    "rolldown:manifest".into()
  }

  fn generate_bundle(&self, _ctx: &SharedPluginContext, _bundle: &mut Vec<Output>, _is_write: bool) -> HookNoopReturn {
    todo!()
  }
}
