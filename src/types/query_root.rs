use crate::model::{__Schema, __Type};
use crate::parser::types::Field;
use crate::resolver_utils::{resolve_object, ObjectType};
use crate::{
    registry, Any, Context, ContextSelectionSet, Error, OutputValueType, Positioned, QueryError,
    Result, SimpleObject, Type,
};

use crate::type_mark::TypeMarkObject;
use indexmap::map::IndexMap;
use std::borrow::Cow;

/// Federation service
#[derive(SimpleObject)]
#[graphql(internal, name = "_Service")]
struct Service {
    sdl: Option<String>,
}

pub(crate) struct QueryRoot<T> {
    pub(crate) inner: T,
    pub(crate) disable_introspection: bool,
}

impl<T: Type> Type for QueryRoot<T> {
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        let schema_type = __Schema::create_type_info(registry);
        let root = T::create_type_info(registry);
        if let Some(registry::MetaType::Object { fields, .. }) =
            registry.types.get_mut(T::type_name().as_ref())
        {
            fields.insert(
                "__schema".to_string(),
                registry::MetaField {
                    name: "__schema".to_string(),
                    description: Some("Access the current type schema of this server."),
                    args: Default::default(),
                    ty: schema_type,
                    deprecation: None,
                    cache_control: Default::default(),
                    external: false,
                    requires: None,
                    provides: None,
                },
            );

            fields.insert(
                "__type".to_string(),
                registry::MetaField {
                    name: "__type".to_string(),
                    description: Some("Request the type information of a single type."),
                    args: {
                        let mut args = IndexMap::new();
                        args.insert(
                            "name",
                            registry::MetaInputValue {
                                name: "name",
                                description: None,
                                ty: "String!".to_string(),
                                default_value: None,
                                validator: None,
                            },
                        );
                        args
                    },
                    ty: "__Type".to_string(),
                    deprecation: None,
                    cache_control: Default::default(),
                    external: false,
                    requires: None,
                    provides: None,
                },
            );
        }
        root
    }
}

#[async_trait::async_trait]
impl<T: ObjectType + Send + Sync> ObjectType for QueryRoot<T> {
    async fn resolve_field(&self, ctx: &Context<'_>) -> Result<serde_json::Value> {
        if ctx.item.node.name.node == "__schema" {
            if self.disable_introspection {
                return Err(Error::Query {
                    pos: ctx.item.pos,
                    path: ctx
                        .path_node
                        .as_ref()
                        .and_then(|path| serde_json::to_value(path).ok()),
                    err: QueryError::FieldNotFound {
                        field_name: ctx.item.node.name.to_string(),
                        object: Self::type_name().to_string(),
                    },
                });
            }

            let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
            return OutputValueType::resolve(
                &__Schema {
                    registry: &ctx.schema_env.registry,
                },
                &ctx_obj,
                ctx.item,
            )
            .await;
        } else if ctx.item.node.name.node == "__type" {
            let type_name: String = ctx.param_value("name", None)?;
            let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
            return OutputValueType::resolve(
                &ctx.schema_env
                    .registry
                    .types
                    .get(&type_name)
                    .map(|ty| __Type::new_simple(&ctx.schema_env.registry, ty)),
                &ctx_obj,
                ctx.item,
            )
            .await;
        } else if ctx.item.node.name.node == "_entities" {
            let representations: Vec<Any> = ctx.param_value("representations", None)?;
            let mut res = Vec::new();
            for item in representations {
                res.push(self.inner.find_entity(ctx, &item.0).await?);
            }
            return Ok(res.into());
        } else if ctx.item.node.name.node == "_service" {
            let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
            return OutputValueType::resolve(
                &Service {
                    sdl: Some(ctx.schema_env.registry.export_sdl(true)),
                },
                &ctx_obj,
                ctx.item,
            )
            .await;
        }

        self.inner.resolve_field(ctx).await
    }
}

#[async_trait::async_trait]
impl<T: ObjectType + Send + Sync> OutputValueType for QueryRoot<T> {
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        _field: &Positioned<Field>,
    ) -> Result<serde_json::Value> {
        resolve_object(ctx, self).await
    }
}

impl<T: ObjectType + Send + Sync> TypeMarkObject for QueryRoot<T> {}
