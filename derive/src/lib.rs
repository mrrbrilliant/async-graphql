#![allow(clippy::cognitive_complexity)]
#![forbid(unsafe_code)]

extern crate proc_macro;

mod args;
mod r#enum;
mod input_object;
mod interface;
mod merged_object;
mod merged_subscription;
mod object;
mod output_type;
mod scalar;
mod simple_object;
mod subscription;
mod union;
mod utils;

use proc_macro::TokenStream;
use syn::parse_macro_input;
use syn::{AttributeArgs, ItemImpl};

#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Object(args: TokenStream, input: TokenStream) -> TokenStream {
    let object_args = match args::Object::from_list(&parse_macro_input!(args as AttributeArgs)) {
        Ok(object_args) => object_args,
        Err(err) => return TokenStream::from(err.write_errors()),
    };
    let mut item_impl = parse_macro_input!(input as ItemImpl);
    match object::generate(&object_args, &mut item_impl) {
        Ok(expanded) => expanded,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(SimpleObject, attributes(field, graphql))]
pub fn derive_simple_object(input: TokenStream) -> TokenStream {
    let object_args =
        match args::SimpleObject::from_derive_input(parse_macro_input!(input as AttributeArgs)) {
            Ok(object_args) => object_args,
            Err(err) => return TokenStream::from(err.write_errors()),
        };
    match simple_object::generate(&object_args) {
        Ok(expanded) => expanded,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(Enum, attributes(item, graphql))]
pub fn derive_enum(input: TokenStream) -> TokenStream {
    let enum_args = match args::Enum::from_derive_input(parse_macro_input!(input as AttributeArgs))
    {
        Ok(enum_args) => enum_args,
        Err(err) => return TokenStream::from(err.write_errors()),
    };
    match r#enum::generate(&enum_args) {
        Ok(expanded) => expanded,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(InputObject, attributes(field, graphql))]
pub fn derive_input_object(input: TokenStream) -> TokenStream {
    let object_args =
        match args::InputObject::from_derive_input(parse_macro_input!(input as AttributeArgs)) {
            Ok(object_args) => object_args,
            Err(err) => return TokenStream::from(err.write_errors()),
        };
    match input_object::generate(&object_args) {
        Ok(expanded) => expanded,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(Interface, attributes(graphql))]
pub fn derive_interface(input: TokenStream) -> TokenStream {
    let interface_args =
        match args::Interface::from_derive_input(parse_macro_input!(input as AttributeArgs)) {
            Ok(interface_args) => interface_args,
            Err(err) => return TokenStream::from(err.write_errors()),
        };
    match interface::generate(&interface_args) {
        Ok(expanded) => expanded,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(Union, attributes(graphql, item))]
pub fn derive_union(input: TokenStream) -> TokenStream {
    let union_args =
        match args::Union::from_derive_input(parse_macro_input!(input as AttributeArgs)) {
            Ok(union_args) => union_args,
            Err(err) => return TokenStream::from(err.write_errors()),
        };
    match union::generate(&union_args) {
        Ok(expanded) => expanded,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Subscription(args: TokenStream, input: TokenStream) -> TokenStream {
    let object_args =
        match args::Subscription::from_list(&parse_macro_input!(args as AttributeArgs)) {
            Ok(object_args) => object_args,
            Err(err) => return TokenStream::from(err.write_errors()),
        };
    let mut item_impl = parse_macro_input!(input as ItemImpl);
    let mut item_impl = parse_macro_input!(input as ItemImpl);
    match subscription::generate(&object_args, &mut item_impl) {
        Ok(expanded) => expanded,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Scalar(args: TokenStream, input: TokenStream) -> TokenStream {
    let scalar_args = match args::Scalar::from_list(&parse_macro_input!(args as AttributeArgs)) {
        Ok(scalar_args) => scalar_args,
        Err(err) => return TokenStream::from(err.write_errors()),
    };
    let mut item_impl = parse_macro_input!(input as ItemImpl);
    match scalar::generate(&scalar_args, &mut item_impl) {
        Ok(expanded) => expanded,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(MergedObject, attributes(item, graphql))]
pub fn derive_merged_object(input: TokenStream) -> TokenStream {
    let (args, input) = match parse_derive(input.into()) {
        Ok(r) => r,
        Err(err) => return err.to_compile_error().into(),
    };
    let object_args = match args::Object::parse(parse_macro_input!(args as AttributeArgs)) {
        Ok(object_args) => object_args,
        Err(err) => return err.to_compile_error().into(),
    };
    match merged_object::generate(&object_args, &input) {
        Ok(expanded) => expanded,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(MergedSubscription, attributes(item, graphql))]
pub fn derive_merged_subscription(input: TokenStream) -> TokenStream {
    let (args, input) = match parse_derive(input.into()) {
        Ok(r) => r,
        Err(err) => return err.to_compile_error().into(),
    };
    let object_args = match args::Object::parse(parse_macro_input!(args as AttributeArgs)) {
        Ok(object_args) => object_args,
        Err(err) => return err.to_compile_error().into(),
    };
    match merged_subscription::generate(&object_args, &input) {
        Ok(expanded) => expanded,
        Err(err) => err.to_compile_error().into(),
    }
}
