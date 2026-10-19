//! Plugin API shell for builds without the WebAssembly runtime.
use crate::diag::{SourceResult, StrResult, bail};
use crate::engine::Engine;
use crate::foundations::{Bytes, Func, Module, Value, cast, func, scope};
use crate::loading::DataSource;
use ecow::EcoString;
use typst_syntax::Spanned;

/// Loads a WebAssembly module (unavailable in this build).
#[func(scope)]
pub fn plugin(
    engine: &mut Engine,
    /// A WebAssembly path or bytes.
    source: Spanned<DataSource>,
) -> SourceResult<Module> {
    let _ = engine;
    bail!(source.span, "WebAssembly plugins are disabled in this build")
}

#[scope]
impl plugin {
    /// Transitions a plugin (unavailable in this build).
    #[func]
    pub fn transition(
        /// The plugin function.
        func: PluginFunc,
        /// Arguments to the function.
        #[variadic]
        arguments: Vec<Bytes>,
    ) -> StrResult<Module> {
        func.transition(arguments)
    }
}

/// An unconstructible plugin function when plugin support is disabled.
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct PluginFunc {
    name: EcoString,
}
impl PluginFunc {
    /// The function name.
    pub fn name(&self) -> &EcoString {
        &self.name
    }
    /// Calls the function.
    pub fn call(&self, _args: Vec<Bytes>) -> StrResult<Bytes> {
        bail!("WebAssembly plugins are disabled in this build")
    }
    /// Transitions the function.
    pub fn transition(&self, _args: Vec<Bytes>) -> StrResult<Module> {
        bail!("WebAssembly plugins are disabled in this build")
    }
}
cast! {
    PluginFunc,
    self => Value::Func(self.into()),
    v: Func => v.to_plugin().ok_or("expected plugin function")?.clone(),
}
