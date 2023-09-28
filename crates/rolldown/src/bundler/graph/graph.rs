use rolldown_common::ModuleId;
use rustc_hash::FxHashSet;

use super::{linker::Linker, symbols::Symbols};
use crate::bundler::{
  module::module_id::ModuleVec, module_loader::ModuleLoader,
  options::normalized_input_options::NormalizedInputOptions,
};

#[derive(Default, Debug)]
pub struct Graph {
  pub modules: ModuleVec,
  pub entries: Vec<ModuleId>,
  pub sorted_modules: Vec<ModuleId>,
  pub symbols: Symbols,
}

impl Graph {
  pub async fn generate_module_graph(
    &mut self,
    input_options: &NormalizedInputOptions,
  ) -> anyhow::Result<()> {
    ModuleLoader::new(input_options, self)
      .fetch_all_modules()
      .await?;

    tracing::trace!("{:#?}", self);

    self.sort_modules();

    self.link();

    Ok(())
  }

  pub fn sort_modules(&mut self) {
    let mut stack = self
      .entries
      .iter()
      .copied()
      .map(Action::Enter)
      .rev()
      .collect::<Vec<_>>();

    let mut entered_ids: FxHashSet<ModuleId> = FxHashSet::default();
    entered_ids.shrink_to(self.modules.len());
    let mut sorted_modules = Vec::with_capacity(self.modules.len());
    let mut next_exec_order = 0;
    while let Some(action) = stack.pop() {
      let module = &mut self.modules[action.module_id()];
      match action {
        Action::Enter(id) => {
          if !entered_ids.contains(&id) {
            entered_ids.insert(id);
            stack.push(Action::Exit(id));
            stack.extend(
              module
                .import_records()
                .iter()
                .filter_map(|rec| {
                  rec
                    .resolved_module
                    .is_valid()
                    .then_some(rec.resolved_module)
                })
                .map(Action::Enter),
            );
          }
        }
        Action::Exit(id) => {
          *module.exec_order_mut() = next_exec_order;
          next_exec_order += 1;
          sorted_modules.push(id);
        }
      }
    }
    self.sorted_modules = sorted_modules;
  }

  pub fn link(&mut self) {
    Linker::new(self).link();
    self.modules.iter_mut().for_each(|module| match module {
      crate::bundler::module::module::Module::Normal(module) => {
        if module.is_symbol_for_namespace_referenced {
          module.initialize_namespace();
        }
      }
      crate::bundler::module::module::Module::External(_) => {}
    });
  }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Action {
  Enter(ModuleId),
  Exit(ModuleId),
}

impl Action {
  #[inline]
  fn module_id(&self) -> ModuleId {
    match self {
      Action::Enter(id) => *id,
      Action::Exit(id) => *id,
    }
  }
}