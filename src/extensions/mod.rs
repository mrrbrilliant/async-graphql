//! Extensions for schema

#[cfg(feature = "apollo_tracing")]
mod apollo_tracing;
#[cfg(feature = "log")]
mod logger;
#[cfg(feature = "tracing")]
mod tracing;

use crate::context::{QueryPathNode, ResolveId};
use crate::{FieldResult, QueryEnv, Result, SchemaEnv, Variables};

#[cfg(feature = "apollo_tracing")]
pub use self::apollo_tracing::ApolloTracing;
#[cfg(feature = "log")]
pub use self::logger::Logger;
#[cfg(feature = "tracing")]
pub use self::tracing::Tracing;
use crate::parser::types::ExecutableDocument;
use crate::Error;
use serde_json::Value;
use std::any::{Any, TypeId};

pub(crate) type BoxExtension = Box<dyn Extension>;

#[doc(hidden)]
pub struct Extensions(pub(crate) Vec<BoxExtension>);

/// Parameters for `Extension::resolve_field_start`
pub struct ResolveInfo<'a> {
    /// Because resolver is concurrent, `Extension::resolve_field_start` and `Extension::resolve_field_end` are
    /// not strictly ordered, so each pair is identified by an id.
    pub resolve_id: ResolveId,

    /// Current path node, You can go through the entire path.
    pub path_node: &'a QueryPathNode<'a>,

    /// Parent type
    pub parent_type: &'a str,

    /// Current return type, is qualified name.
    pub return_type: &'a str,

    #[doc(hidden)]
    pub schema_env: &'a SchemaEnv,

    #[doc(hidden)]
    pub query_env: &'a QueryEnv,
}

impl<'a> ResolveInfo<'a> {
    /// Gets the global data defined in the `Context` or `Schema`.
    ///
    /// If both `Schema` and `Query` have the same data type, the data in the `Query` is obtained.
    ///
    /// # Errors
    ///
    /// Returns a `FieldError` if the specified type data does not exist.
    pub fn data<D: Any + Send + Sync>(&self) -> FieldResult<&D> {
        self.data_opt::<D>()
            .ok_or_else(|| format!("Data `{}` does not exist.", std::any::type_name::<D>()).into())
    }

    /// Gets the global data defined in the `Context` or `Schema`.
    ///
    /// # Panics
    ///
    /// It will panic if the specified data type does not exist.
    pub fn data_unchecked<D: Any + Send + Sync>(&self) -> &D {
        self.data_opt::<D>()
            .unwrap_or_else(|| panic!("Data `{}` does not exist.", std::any::type_name::<D>()))
    }

    /// Gets the global data defined in the `Context` or `Schema` or `None` if the specified type data does not exist.
    pub fn data_opt<D: Any + Send + Sync>(&self) -> Option<&D> {
        self.query_env
            .ctx_data
            .get(&TypeId::of::<D>())
            .or_else(|| self.schema_env.data.get(&TypeId::of::<D>()))
            .and_then(|d| d.downcast_ref::<D>())
    }
}

/// Represents a GraphQL extension
#[allow(unused_variables)]
pub trait Extension: Sync + Send + 'static {
    /// If this extension needs to output data to query results, you need to specify a name.
    fn name(&self) -> Option<&'static str> {
        None
    }

    /// Called at the begin of the parse.
    fn parse_start(&mut self, query_source: &str, variables: &Variables) {}

    /// Called at the end of the parse.
    fn parse_end(&mut self, document: &ExecutableDocument) {}

    /// Called at the begin of the validation.
    fn validation_start(&mut self) {}

    /// Called at the end of the validation.
    fn validation_end(&mut self) {}

    /// Called at the begin of the execution.
    fn execution_start(&mut self) {}

    /// Called at the end of the execution.
    fn execution_end(&mut self) {}

    /// Called at the begin of the resolve field.
    fn resolve_start(&mut self, info: &ResolveInfo<'_>) {}

    /// Called at the end of the resolve field.
    fn resolve_end(&mut self, info: &ResolveInfo<'_>) {}

    /// Called when an error occurs.
    fn error(&mut self, err: &Error) {}

    /// Get the results
    fn result(&mut self) -> Option<serde_json::Value> {
        None
    }
}

pub(crate) trait ErrorLogger {
    fn log_error(self, extensions: &spin::Mutex<Extensions>) -> Self;
}

impl<T> ErrorLogger for Result<T> {
    fn log_error(self, extensions: &spin::Mutex<Extensions>) -> Self {
        if let Err(err) = &self {
            extensions.lock().error(err);
        }
        self
    }
}

impl Extension for Extensions {
    fn parse_start(&mut self, query_source: &str, variables: &Variables) {
        self.0
            .iter_mut()
            .for_each(|e| e.parse_start(query_source, variables));
    }

    fn parse_end(&mut self, document: &ExecutableDocument) {
        self.0.iter_mut().for_each(|e| e.parse_end(document));
    }

    fn validation_start(&mut self) {
        self.0.iter_mut().for_each(|e| e.validation_start());
    }

    fn validation_end(&mut self) {
        self.0.iter_mut().for_each(|e| e.validation_end());
    }

    fn execution_start(&mut self) {
        self.0.iter_mut().for_each(|e| e.execution_start());
    }

    fn execution_end(&mut self) {
        self.0.iter_mut().for_each(|e| e.execution_end());
    }

    fn resolve_start(&mut self, info: &ResolveInfo<'_>) {
        self.0.iter_mut().for_each(|e| e.resolve_start(info));
    }

    fn resolve_end(&mut self, resolve_id: &ResolveInfo<'_>) {
        self.0.iter_mut().for_each(|e| e.resolve_end(resolve_id));
    }

    fn error(&mut self, err: &Error) {
        self.0.iter_mut().for_each(|e| e.error(err));
    }

    fn result(&mut self) -> Option<Value> {
        if !self.0.is_empty() {
            let value = self
                .0
                .iter_mut()
                .filter_map(|e| {
                    if let Some(name) = e.name() {
                        e.result().map(|res| (name.to_string(), res))
                    } else {
                        None
                    }
                })
                .collect::<serde_json::Map<_, _>>();
            if value.is_empty() {
                None
            } else {
                Some(value.into())
            }
        } else {
            None
        }
    }
}
