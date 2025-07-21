use std::collections::HashMap;
use anyhow::Result;
use nxsh_core::context::ShellContext;

/// Builtin command trait duplicated to avoid circular dep.
pub trait Builtin {
    fn name(&self) -> &'static str;
    fn synopsis(&self) -> &'static str;
    fn invoke(&self, ctx: &mut ShellContext) -> anyhow::Result<()>;
}

/// Registrar passed to plugins for self-registration.
pub struct PluginRegistrar<'a> {
    builtins: HashMap<&'static str, Box<dyn Builtin + 'a>>,
}

impl<'a> PluginRegistrar<'a> {
    pub fn new() -> Self { Self { builtins: HashMap::new() } }
    pub fn register_builtin(&mut self, b: Box<dyn Builtin + 'a>) {
        self.builtins.insert(b.name(), b);
    }
    pub fn builtins(&self) -> impl Iterator<Item=&Box<dyn Builtin + 'a>> {
        self.builtins.values()
    }
} 