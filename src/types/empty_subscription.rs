use crate::type_mark::TypeMarkSubscription;
use crate::{registry, Context, Error, Pos, QueryError, Result, SubscriptionType, Type};
use futures::{stream, Stream};
use std::borrow::Cow;
use std::pin::Pin;

/// Empty subscription
///
/// Only the parameters used to construct the Schema, representing an unconfigured subscription.
#[derive(Default, Copy, Clone)]
pub struct EmptySubscription;

impl Type for EmptySubscription {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("EmptyMutation")
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        registry.create_type::<Self, _>(|_| registry::MetaType::Object {
            name: "EmptySubscription".to_string(),
            description: None,
            fields: Default::default(),
            cache_control: Default::default(),
            extends: false,
            keys: None,
        })
    }
}

impl SubscriptionType for EmptySubscription {
    fn is_empty() -> bool {
        true
    }

    fn create_field_stream<'a>(
        &'a self,
        _ctx: &'a Context<'a>,
    ) -> Pin<Box<dyn Stream<Item = Result<serde_json::Value>> + Send + 'a>>
    where
        Self: Send + Sync + 'static + Sized,
    {
        Box::pin(stream::once(async {
            Err(Error::Query {
                pos: Pos::default(),
                path: None,
                err: QueryError::NotConfiguredSubscriptions,
            })
        }))
    }
}

impl TypeMarkSubscription for EmptySubscription {}
