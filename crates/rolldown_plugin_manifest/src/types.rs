#[derive(Debug)]
pub struct ManifestChunk {
  pub src: Option<String>,
  pub file: String,
  pub css: Option<Vec<String>>,
  pub assets: Option<Vec<String>>,
  pub is_entry: bool,
  pub name: Option<String>,
  pub is_dynamic_entry: bool,
  pub imports: Option<Vec<String>>,
  pub dynamic_imports: Option<Vec<String>>,
}
