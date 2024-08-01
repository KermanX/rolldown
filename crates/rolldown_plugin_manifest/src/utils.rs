use pathdiff::diff_paths;
use rolldown_common::OutputChunk;

pub fn get_chunk_original_file_name(chunk: OutputChunk, root: String) -> anyhow::Result<String> {
  if let Some(module_id) = chunk.facade_module_id {
    get_relative_path(root, module_id.to_string()).and_then(|p| Ok(p.replace('\0', "")))
  } else {
    get_basename(chunk.filename.to_string()).and_then(|p| Ok(format!("\0{p}")))
  }
}

fn get_basename(file_name: String) -> anyhow::Result<String> {
  std::path::Path::new(file_name.as_str())
    .file_name()
    .and_then(|name| name.to_str())
    .and_then(|name| Some(name.to_string()))
    .ok_or(anyhow::anyhow!("Failed to get basename"))
}

fn normalize_path(id: String) -> String {
  id.replace("\\", "/")
}

fn get_relative_path(root: String, target: String) -> anyhow::Result<String> {
  let root = normalize_path(root);
  let target = normalize_path(target);
  diff_paths(&target, &root)
    .ok_or_else(|| anyhow::anyhow!("Failed to get relative path"))
    .map(|p| p.to_string_lossy().to_string())
}
