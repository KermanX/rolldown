use rolldown_common::{
  Chunk, ChunkKind, ModuleId, NormalizedBundlerOptions, PreRenderedChunk, RenderedModule,
  RollupRenderedChunk,
};
use rustc_hash::FxHashMap;

use crate::{chunk_graph::ChunkGraph, stages::link_stage::LinkStageOutput};

use self::render_chunk_exports::get_chunk_export_names;

pub mod collect_render_chunk_imports;
pub mod deconflict_chunk_symbols;
pub mod determine_use_strict;
pub mod finalize_chunks;
pub mod render_chunk_exports;

pub fn generate_pre_rendered_chunk(
  chunk: &Chunk,
  graph: &LinkStageOutput,
  output_options: &NormalizedBundlerOptions,
) -> PreRenderedChunk {
  PreRenderedChunk {
    name: chunk.name.clone().expect("should have name"),
    is_entry: matches!(&chunk.kind, ChunkKind::EntryPoint { is_user_defined, .. } if *is_user_defined),
    is_dynamic_entry: matches!(&chunk.kind, ChunkKind::EntryPoint { is_user_defined, .. } if !*is_user_defined),
    facade_module_id: match &chunk.kind {
      ChunkKind::EntryPoint { module, .. } => {
        Some(graph.module_table.modules[*module].id().to_string().into())
      }
      ChunkKind::Common => None,
    },
    module_ids: chunk
      .modules
      .iter()
      .map(|id| graph.module_table.modules[*id].id().to_string().into())
      .collect(),
    exports: get_chunk_export_names(chunk, graph, output_options),
  }
}

pub fn generate_rendered_chunk(
  chunk: &Chunk,
  graph: &LinkStageOutput,
  output_options: &NormalizedBundlerOptions,
  render_modules: FxHashMap<ModuleId, RenderedModule>,
  chunk_graph: &ChunkGraph,
) -> RollupRenderedChunk {
  let pre_rendered_chunk = generate_pre_rendered_chunk(chunk, graph, output_options);
  RollupRenderedChunk {
    name: pre_rendered_chunk.name,
    is_entry: pre_rendered_chunk.is_entry,
    is_dynamic_entry: pre_rendered_chunk.is_dynamic_entry,
    facade_module_id: pre_rendered_chunk.facade_module_id,
    module_ids: pre_rendered_chunk.module_ids,
    exports: pre_rendered_chunk.exports,
    filename: chunk
      .preliminary_filename
      .as_deref()
      .expect("should have preliminary_filename")
      .clone(),
    modules: render_modules,
    imports: chunk
      .cross_chunk_imports
      .iter()
      .map(|id| {
        chunk_graph.chunks[*id]
          .preliminary_filename
          .as_deref()
          .expect("should have preliminary_filename")
          .clone()
      })
      .collect(),
    dynamic_imports: chunk
      .cross_chunk_dynamic_imports
      .iter()
      .map(|id| {
        chunk_graph.chunks[*id]
          .preliminary_filename
          .as_deref()
          .expect("should have preliminary_filename")
          .clone()
      })
      .collect(),
  }
}
