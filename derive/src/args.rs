use darling::ast::{Data, Fields};
use darling::util::Ignored;
use darling::{FromDeriveInput, FromField, FromMeta, FromVariant};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Attribute, AttributeArgs, Error, Generics, Ident, Lit, LitStr, Meta, MetaList, NestedMeta,
    Result, Type, Visibility,
};

#[derive(FromMeta)]
#[darling(default)]
pub struct CacheControl {
    #[darling(default = "default_cache_control_public")]
    pub public: bool,
    pub max_age: usize,
}

fn default_cache_control_public() -> bool {
    true
}

#[derive(FromField)]
#[darling(default, attributes(graphql), forward_attrs(doc))]
pub struct SimpleObjectField {
    pub ident: Option<Ident>,
    pub ty: Type,
    pub vis: Visibility,
    pub attrs: Vec<Attribute>,

    pub skip: bool,
    pub name: Option<String>,
    pub deprecation: Option<String>,
    pub owned: bool,
    pub cache_control: CacheControl,
    pub external: bool,
    pub provides: Option<String>,
    pub requires: Option<String>,
    pub guard: Option<MetaList>,
    pub post_guard: Option<MetaList>,
}

#[derive(FromDeriveInput)]
#[darling(default, from_ident, attributes(graphql), forward_attrs(doc))]
pub struct SimpleObject {
    pub ident: Ident,
    pub generics: Generics,
    pub attrs: Vec<Attribute>,
    pub data: Data<Ignored, SimpleObjectField>,

    pub internal: bool,
    pub name: Option<String>,
    pub cache_control: CacheControl,
    pub extends: bool,
}

#[derive(FromMeta)]
#[darling(default)]
pub struct Argument {
    pub name: Option<String>,
    pub desc: Option<String>,
    pub default: bool,
    pub default_value: Option<Lit>,
    pub default_with: Option<LitStr>,
    pub validator: Option<MetaList>,
    pub key: bool, // for entity
}

#[derive(FromMeta)]
#[darling(default)]
pub struct Object {
    pub internal: bool,
    pub name: Option<String>,
    pub cache_control: CacheControl,
    pub extends: bool,
}

#[derive(FromMeta)]
#[darling(default)]
pub struct ObjectField {
    pub skip: bool,
    pub entity: bool,
    pub name: Option<String>,
    pub deprecation: Option<String>,
    pub cache_control: CacheControl,
    pub external: bool,
    pub provides: Option<String>,
    pub requires: Option<String>,
    pub guard: Option<MetaList>,
    pub post_guard: Option<MetaList>,
}

#[derive(FromMeta)]
#[darling(default, allow_unknown_fields)]
pub struct ObjectFieldWrapper {
    pub graphql: ObjectField,
}

#[derive(FromDeriveInput)]
#[darling(default, from_ident, attributes(graphql), forward_attrs(doc))]
pub struct Enum {
    pub ident: Ident,
    pub generics: Generics,
    pub attrs: Vec<Attribute>,
    pub data: Data<EnumItem, Ignored>,

    pub internal: bool,
    pub name: Option<String>,
    pub remote: Option<String>,
}

#[derive(FromVariant)]
#[darling(default, from_ident, attributes(graphql), forward_attrs(doc))]
pub struct EnumItem {
    pub ident: Ident,
    pub attrs: Vec<Attribute>,
    pub fields: Fields<Ignored>,

    pub name: Option<String>,
    pub deprecation: Option<String>,
}

#[derive(FromDeriveInput)]
#[darling(default, from_ident, attributes(graphql), forward_attrs(doc))]
pub struct Union {
    pub ident: Ident,
    pub generics: Generics,
    pub attrs: Vec<Attribute>,
    pub data: Data<UnionItem, Ignored>,
}

#[derive(FromVariant)]
#[darling(default, from_ident, attributes(graphql))]
pub struct UnionItem {
    pub ident: Ident,

    pub flatten: bool,
}

#[derive(FromField)]
#[darling(default, from_ident, attributes(graphql), forward_attrs(doc))]
pub struct InputObjectField {
    pub ident: Option<Ident>,
    pub ty: Type,
    pub attrs: Vec<Attribute>,

    pub name: Option<String>,
    pub default: bool,
    pub default_value: Option<Lit>,
    pub default_with: Option<LitStr>,
    pub validator: Option<MetaList>,
    pub flatten: bool,
}

#[derive(FromDeriveInput)]
#[darling(default, from_ident, attributes(graphql), forward_attrs(doc))]
pub struct InputObject {
    pub ident: Ident,
    pub generics: Generics,
    pub attrs: Vec<Attribute>,
    pub data: Data<Ignored, InputObjectField>,

    pub internal: bool,
    pub name: Option<String>,
}

#[derive(FromMeta)]
pub struct InterfaceFieldArgument {
    pub name: String,
    pub desc: Option<String>,
    pub ty: LitStr,
    #[darling(default)]
    pub default: bool,
    pub default_value: Option<Lit>,
    pub default_with: Option<LitStr>,
}

#[derive(FromMeta)]
pub struct InterfaceField {
    pub name: String,
    pub method: Option<String>,
    pub desc: Option<String>,
    pub ty: Type,
    #[darling(default)]
    pub args: Vec<InterfaceFieldArgument>,
    pub deprecation: Option<String>,
    #[darling(default)]
    pub external: bool,
    pub provides: Option<String>,
    pub requires: Option<String>,
}

#[derive(FromDeriveInput)]
#[darling(default, from_ident, attributes(graphql), forward_attrs(doc))]
pub struct Interface {
    pub ident: Ident,
    pub generics: Generics,
    pub attrs: Vec<Attribute>,
    pub data: Data<Ignored, InputObjectField>,

    pub internal: bool,
    pub name: Option<String>,
    pub desc: Option<String>,
    pub fields: Vec<InterfaceField>,
    pub extends: bool,
}

#[derive(FromMeta)]
#[darling(default)]
pub struct Scalar {
    pub internal: bool,
    pub name: Option<String>,
    pub desc: Option<String>,
}

#[derive(FromMeta)]
#[darling(default)]
pub struct Subscription {
    pub internal: bool,
    pub name: Option<String>,
}

#[derive(FromMeta)]
#[darling(default)]
pub struct SubscriptionField {
    pub skip: bool,
    pub name: Option<String>,
    pub deprecation: Option<String>,
    pub guard: Option<MetaList>,
    pub post_guard: Option<MetaList>,
}

#[derive(FromMeta)]
#[darling(default, allow_unknown_fields)]
pub struct SubscriptionFieldWrapper {
    pub graphql: SubscriptionField,
}

#[derive(FromField)]
#[darling(default, from_ident, attributes(graphql), forward_attrs(doc))]
pub struct MergedObjectField {
    pub ident: Option<Ident>,
    pub ty: Type,
}

#[derive(FromDeriveInput)]
#[darling(default, from_ident, attributes(graphql), forward_attrs(doc))]
pub struct MergedObject {
    pub ident: Ident,
    pub generics: Generics,
    pub attrs: Vec<Attribute>,
    pub data: Data<Ignored, MergedObjectField>,

    pub internal: bool,
    pub name: Option<String>,
    pub cache_control: CacheControl,
    pub extends: bool,
}

#[derive(FromField)]
#[darling(default, from_ident, attributes(graphql), forward_attrs(doc))]
pub struct MergedSubscriptionField {
    pub ident: Option<Ident>,
    pub ty: Type,
}

#[derive(FromDeriveInput)]
#[darling(default, from_ident, attributes(graphql), forward_attrs(doc))]
pub struct MergedSubscription {
    pub ident: Ident,
    pub generics: Generics,
    pub attrs: Vec<Attribute>,
    pub data: Data<Ignored, MergedSubscriptionField>,

    pub internal: bool,
    pub name: Option<String>,
}
