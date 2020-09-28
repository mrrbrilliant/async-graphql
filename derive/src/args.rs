use crate::utils::{get_rustdoc, parse_default, parse_default_with};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, AttributeArgs, Error, Lit, Meta, MetaList, NestedMeta, Result, Type};

pub struct CacheControl {
    pub public: bool,
    pub max_age: usize,
}

impl Default for CacheControl {
    fn default() -> Self {
        Self {
            public: true,
            max_age: 0,
        }
    }
}

impl CacheControl {
    pub fn parse(ls: &MetaList) -> Result<Self> {
        let mut cache_control = Self {
            public: true,
            max_age: 0,
        };

        for meta in &ls.nested {
            match meta {
                NestedMeta::Meta(Meta::NameValue(nv)) => {
                    if nv.path.is_ident("max_age") {
                        if let Lit::Int(n) = &nv.lit {
                            match n.base10_parse::<usize>() {
                                Ok(n) => cache_control.max_age = n,
                                Err(err) => {
                                    return Err(Error::new_spanned(&nv.lit, err));
                                }
                            }
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Attribute 'max_age' must be integer.",
                            ));
                        }
                    }
                }
                NestedMeta::Meta(Meta::Path(p)) => {
                    if p.is_ident("public") {
                        cache_control.public = true;
                    } else if p.is_ident("private") {
                        cache_control.public = false;
                    }
                }
                _ => {}
            }
        }

        Ok(cache_control)
    }
}

pub struct Object {
    pub internal: bool,
    pub name: Option<String>,
    pub desc: Option<String>,
    pub cache_control: CacheControl,
    pub extends: bool,
}

impl Object {
    pub fn parse(args: AttributeArgs) -> Result<Self> {
        let mut internal = false;
        let mut name = None;
        let mut desc = None;
        let mut cache_control = CacheControl::default();
        let mut extends = false;

        for arg in args {
            match arg {
                NestedMeta::Meta(Meta::Path(p)) if p.is_ident("internal") => {
                    internal = true;
                }
                NestedMeta::Meta(Meta::Path(p)) if p.is_ident("extends") => {
                    extends = true;
                }
                NestedMeta::Meta(Meta::NameValue(nv)) => {
                    if nv.path.is_ident("name") {
                        if let syn::Lit::Str(lit) = nv.lit {
                            name = Some(lit.value());
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Attribute 'name' should be a string.",
                            ));
                        }
                    } else if nv.path.is_ident("desc") {
                        if let syn::Lit::Str(lit) = nv.lit {
                            desc = Some(lit.value());
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Attribute 'desc' should be a string.",
                            ));
                        }
                    }
                }
                NestedMeta::Meta(Meta::List(ls)) => {
                    if ls.path.is_ident("cache_control") {
                        cache_control = CacheControl::parse(&ls)?;
                    }
                }
                _ => {}
            }
        }

        Ok(Self {
            internal,
            name,
            desc,
            cache_control,
            extends,
        })
    }
}

pub struct Argument {
    pub name: Option<String>,
    pub desc: Option<String>,
    pub default: bool,
    pub default_value: Option<Lit>,
    pub default_with: Option<LitStr>,
    pub validator: Option<MetaList>,
    pub key: bool, // for entity
}

impl Argument {
    pub fn parse(attrs: &[Attribute]) -> Result<Self> {
        let mut name = None;
        let mut desc = None;
        let mut default = false;
        let mut default_value = None;
        let mut default_with = None;
        let mut validator = None;
        let mut key = false;

        for attr in attrs {
            match attr.parse_meta()? {
                Meta::List(ls) if ls.path.is_ident("arg") => {
                    for meta in &ls.nested {
                        if let NestedMeta::Meta(Meta::Path(p)) = meta {
                            if p.is_ident("default") {
                                default = true;
                            } else if p.is_ident("key") {
                                key = true;
                            }
                        } else if let NestedMeta::Meta(Meta::NameValue(nv)) = meta {
                            if nv.path.is_ident("name") {
                                if let syn::Lit::Str(lit) = &nv.lit {
                                    name = Some(lit.value());
                                } else {
                                    return Err(Error::new_spanned(
                                        &nv.lit,
                                        "Attribute 'name' should be a string.",
                                    ));
                                }
                            } else if nv.path.is_ident("desc") {
                                if let syn::Lit::Str(lit) = &nv.lit {
                                    desc = Some(lit.value());
                                } else {
                                    return Err(Error::new_spanned(
                                        &nv.lit,
                                        "Attribute 'desc' should be a string.",
                                    ));
                                }
                            } else if nv.path.is_ident("default_value") {
                                default_value = Some(nv.lit.clone());
                            } else if nv.path.is_ident("default_with") {
                                default_with = Some(parse_default_with(&nv.lit)?);
                            }
                        }
                    }

                    validator = Some(ls);
                }
                _ => {}
            }
        }

        Ok(Self {
            name,
            desc,
            default,
            default_value,
            default_with,
            validator,
            key,
        })
    }
}

pub struct Field {
    pub name: Option<String>,
    pub desc: Option<String>,
    pub deprecation: Option<String>,
    pub cache_control: CacheControl,
    pub external: bool,
    pub provides: Option<String>,
    pub requires: Option<String>,
    pub owned: bool,
    pub guard: Option<MetaList>,
    pub post_guard: Option<MetaList>,
}

impl Field {
    pub fn parse(attrs: &[Attribute]) -> Result<Option<Self>> {
        let mut name = None;
        let mut desc = None;
        let mut deprecation = None;
        let mut cache_control = CacheControl::default();
        let mut external = false;
        let mut provides = None;
        let mut requires = None;
        let mut owned = false;
        let mut guard = None;
        let mut post_guard = None;

        for attr in attrs {
            match attr.parse_meta()? {
                Meta::List(ls) if ls.path.is_ident("field") => {
                    guard = parse_guards(crate_name, &ls)?;
                    post_guard = parse_post_guards(crate_name, &ls)?;
                    for meta in &ls.nested {
                        match meta {
                            NestedMeta::Meta(Meta::Path(p)) if p.is_ident("skip") => {
                                return Ok(None);
                            }
                            NestedMeta::Meta(Meta::Path(p)) if p.is_ident("external") => {
                                external = true;
                            }
                            NestedMeta::Meta(Meta::Path(p)) if p.is_ident("owned") => {
                                owned = true;
                            }
                            NestedMeta::Meta(Meta::Path(p)) if p.is_ident("ref") => {
                                return Err(Error::new_spanned(
                                    &p,
                                    "Attribute `ref` is no longer supported. By default, all fields resolver return borrowed value. If you want to return ownership value, use `owned` attribute.",
                                ));
                            }
                            NestedMeta::Meta(Meta::NameValue(nv)) => {
                                if nv.path.is_ident("name") {
                                    if let syn::Lit::Str(lit) = &nv.lit {
                                        name = Some(lit.value());
                                    } else {
                                        return Err(Error::new_spanned(
                                            &nv.lit,
                                            "Attribute 'name' should be a string.",
                                        ));
                                    }
                                } else if nv.path.is_ident("desc") {
                                    if let syn::Lit::Str(lit) = &nv.lit {
                                        desc = Some(lit.value());
                                    } else {
                                        return Err(Error::new_spanned(
                                            &nv.lit,
                                            "Attribute 'desc' should be a string.",
                                        ));
                                    }
                                } else if nv.path.is_ident("deprecation") {
                                    if let syn::Lit::Str(lit) = &nv.lit {
                                        deprecation = Some(lit.value());
                                    } else {
                                        return Err(Error::new_spanned(
                                            &nv.lit,
                                            "Attribute 'deprecation' should be a string.",
                                        ));
                                    }
                                } else if nv.path.is_ident("provides") {
                                    if let syn::Lit::Str(lit) = &nv.lit {
                                        provides = Some(lit.value());
                                    } else {
                                        return Err(Error::new_spanned(
                                            &nv.lit,
                                            "Attribute 'provides' should be a string.",
                                        ));
                                    }
                                } else if nv.path.is_ident("requires") {
                                    if let syn::Lit::Str(lit) = &nv.lit {
                                        requires = Some(lit.value());
                                    } else {
                                        return Err(Error::new_spanned(
                                            &nv.lit,
                                            "Attribute 'requires' should be a string.",
                                        ));
                                    }
                                }
                            }
                            NestedMeta::Meta(Meta::List(ls)) => {
                                if ls.path.is_ident("cache_control") {
                                    cache_control = CacheControl::parse(ls)?;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        if desc.is_none() {
            desc = get_rustdoc(attrs)?;
        }

        Ok(Some(Self {
            name,
            desc,
            deprecation,
            cache_control,
            external,
            provides,
            requires,
            owned,
            guard,
            post_guard,
        }))
    }
}

pub struct Enum {
    pub internal: bool,
    pub name: Option<String>,
    pub desc: Option<String>,
    pub remote: Option<String>,
}

impl Enum {
    pub fn parse(args: AttributeArgs) -> Result<Self> {
        let mut internal = false;
        let mut name = None;
        let mut desc = None;
        let mut remote = None;

        for arg in args {
            match arg {
                NestedMeta::Meta(Meta::Path(p)) if p.is_ident("internal") => {
                    internal = true;
                }
                NestedMeta::Meta(Meta::NameValue(nv)) => {
                    if nv.path.is_ident("name") {
                        if let syn::Lit::Str(lit) = nv.lit {
                            name = Some(lit.value());
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Attribute 'name' should be a string.",
                            ));
                        }
                    } else if nv.path.is_ident("desc") {
                        if let syn::Lit::Str(lit) = nv.lit {
                            desc = Some(lit.value());
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Attribute 'desc' should be a string.",
                            ));
                        }
                    } else if nv.path.is_ident("remote") {
                        if let syn::Lit::Str(lit) = nv.lit {
                            remote = Some(lit.value());
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Attribute 'remote' should be a string.",
                            ));
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(Self {
            internal,
            name,
            desc,
            remote,
        })
    }
}

pub struct EnumItem {
    pub name: Option<String>,
    pub desc: Option<String>,
    pub deprecation: Option<String>,
}

impl EnumItem {
    pub fn parse(attrs: &[Attribute]) -> Result<Self> {
        let mut name = None;
        let mut desc = None;
        let mut deprecation = None;

        for attr in attrs {
            if attr.path.is_ident("item") {
                if let Meta::List(args) = attr.parse_meta()? {
                    for meta in args.nested {
                        if let NestedMeta::Meta(Meta::NameValue(nv)) = meta {
                            if nv.path.is_ident("name") {
                                if let syn::Lit::Str(lit) = nv.lit {
                                    name = Some(lit.value());
                                } else {
                                    return Err(Error::new_spanned(
                                        &nv.lit,
                                        "Attribute 'name' should be a string.",
                                    ));
                                }
                            } else if nv.path.is_ident("desc") {
                                if let syn::Lit::Str(lit) = nv.lit {
                                    desc = Some(lit.value());
                                } else {
                                    return Err(Error::new_spanned(
                                        &nv.lit,
                                        "Attribute 'desc' should be a string.",
                                    ));
                                }
                            } else if nv.path.is_ident("deprecation") {
                                if let syn::Lit::Str(lit) = nv.lit {
                                    deprecation = Some(lit.value());
                                } else {
                                    return Err(Error::new_spanned(
                                        &nv.lit,
                                        "Attribute 'deprecation' should be a string.",
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        if desc.is_none() {
            desc = get_rustdoc(attrs)?;
        }

        Ok(Self {
            name,
            desc,
            deprecation,
        })
    }
}

pub struct UnionItem {
    pub flatten: bool,
}

impl UnionItem {
    pub fn parse(attrs: &[Attribute]) -> Result<Self> {
        let mut flatten = false;

        for attr in attrs {
            if attr.path.is_ident("item") {
                if let Meta::List(args) = attr.parse_meta()? {
                    for meta in args.nested {
                        if let NestedMeta::Meta(Meta::Path(p)) = meta {
                            if p.is_ident("flatten") {
                                flatten = true;
                            }
                        }
                    }
                }
            }
        }

        Ok(Self { flatten })
    }
}

pub struct InputField {
    pub name: Option<String>,
    pub desc: Option<String>,
    pub default: bool,
    pub default_value: Option<Lit>,
    pub default_with: Option<LitStr>,
    pub validator: Option<MetaList>,
    pub flatten: bool,
}

impl InputField {
    pub fn parse(attrs: &[Attribute]) -> Result<Self> {
        let mut name = None;
        let mut desc = None;
        let mut default = false;
        let mut default_value = None;
        let mut default_with = None;
        let mut validator = quote! { None };
        let mut flatten = false;

        for attr in attrs {
            if attr.path.is_ident("field") {
                if let Meta::List(args) = &attr.parse_meta()? {
                    for meta in &args.nested {
                        match meta {
                            NestedMeta::Meta(Meta::Path(p)) if p.is_ident("skip") => {
                                return Err(Error::new_spanned(
                                    meta,
                                    "Fields on InputObject are not allowed to be skipped",
                                ));
                            }
                            NestedMeta::Meta(Meta::Path(p)) if p.is_ident("default") => {
                                default = true;
                            }
                            NestedMeta::Meta(Meta::Path(p)) if p.is_ident("flatten") => {
                                flatten = true;
                            }
                            NestedMeta::Meta(Meta::NameValue(nv)) => {
                                if nv.path.is_ident("name") {
                                    if let syn::Lit::Str(lit) = &nv.lit {
                                        name = Some(lit.value());
                                    } else {
                                        return Err(Error::new_spanned(
                                            &nv.lit,
                                            "Attribute 'name' should be a string.",
                                        ));
                                    }
                                } else if nv.path.is_ident("desc") {
                                    if let syn::Lit::Str(lit) = &nv.lit {
                                        desc = Some(lit.value());
                                    } else {
                                        return Err(Error::new_spanned(
                                            &nv.lit,
                                            "Attribute 'desc' should be a string.",
                                        ));
                                    }
                                } else if nv.path.is_ident("default_value") {
                                    default_value = Some(nv.lit.clone());
                                } else if nv.path.is_ident("default_with") {
                                    default_with = Some(nv.lit.clone());
                                }
                            }
                            _ => {}
                        }
                    }

                    validator = parse_validator(crate_name, &args)?;
                }
            }
        }

        if desc.is_none() {
            desc = get_rustdoc(attrs)?;
        }

        Ok(Self {
            name,
            desc,
            default,
            default_value,
            default_with,
            validator,
            flatten,
        })
    }
}

pub struct InputObject {
    pub internal: bool,
    pub name: Option<String>,
    pub desc: Option<String>,
}

impl InputObject {
    pub fn parse(args: AttributeArgs) -> Result<Self> {
        let mut internal = false;
        let mut name = None;
        let mut desc = None;

        for arg in args {
            match arg {
                NestedMeta::Meta(Meta::Path(p)) if p.is_ident("internal") => {
                    internal = true;
                }
                NestedMeta::Meta(Meta::NameValue(nv)) => {
                    if nv.path.is_ident("name") {
                        if let syn::Lit::Str(lit) = nv.lit {
                            name = Some(lit.value());
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Attribute 'name' should be a string.",
                            ));
                        }
                    } else if nv.path.is_ident("desc") {
                        if let syn::Lit::Str(lit) = nv.lit {
                            desc = Some(lit.value());
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Attribute 'desc' should be a string.",
                            ));
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(Self {
            internal,
            name,
            desc,
        })
    }
}

pub struct InterfaceFieldArgument {
    pub name: String,
    pub desc: Option<String>,
    pub ty: Type,
    pub default: Option<MetaList>,
}

impl InterfaceFieldArgument {
    pub fn parse(ls: &MetaList) -> Result<Self> {
        let mut name = None;
        let mut desc = None;
        let mut ty = None;
        let mut default = None;

        for meta in &ls.nested {
            if let NestedMeta::Meta(Meta::Path(p)) = meta {
                if p.is_ident("default") {
                    default = Some(quote! { Default::default() });
                }
            } else if let NestedMeta::Meta(Meta::NameValue(nv)) = meta {
                if nv.path.is_ident("name") {
                    if let syn::Lit::Str(lit) = &nv.lit {
                        name = Some(lit.value());
                    } else {
                        return Err(Error::new_spanned(
                            &nv.lit,
                            "Attribute 'name' should be a string.",
                        ));
                    }
                } else if nv.path.is_ident("desc") {
                    if let syn::Lit::Str(lit) = &nv.lit {
                        desc = Some(lit.value());
                    } else {
                        return Err(Error::new_spanned(
                            &nv.lit,
                            "Attribute 'desc' should be a string.",
                        ));
                    }
                } else if nv.path.is_ident("type") {
                    if let syn::Lit::Str(lit) = &nv.lit {
                        if let Ok(ty2) = syn::parse_str::<syn::Type>(&lit.value()) {
                            ty = Some(ty2);
                        } else {
                            return Err(Error::new_spanned(&lit, "Expect type"));
                        }
                    } else {
                        return Err(Error::new_spanned(
                            &nv.lit,
                            "Attribute 'type' should be a string.",
                        ));
                    }
                } else if nv.path.is_ident("default") {
                    default = Some(parse_default(&nv.lit)?);
                } else if nv.path.is_ident("default_with") {
                    default = Some(parse_default_with(&nv.lit)?);
                }
            }
        }

        if name.is_none() {
            return Err(Error::new_spanned(ls, "Missing name"));
        }

        if ty.is_none() {
            return Err(Error::new_spanned(ls, "Missing type"));
        }

        Ok(Self {
            name: name.unwrap(),
            desc,
            ty: ty.unwrap(),
            default,
        })
    }
}

pub struct InterfaceField {
    pub name: String,
    pub method: Option<String>,
    pub desc: Option<String>,
    pub ty: Type,
    pub args: Vec<InterfaceFieldArgument>,
    pub deprecation: Option<String>,
    pub external: bool,
    pub provides: Option<String>,
    pub requires: Option<String>,
}

impl InterfaceField {
    pub fn parse(ls: &MetaList) -> Result<Self> {
        let mut name = None;
        let mut method = None;
        let mut desc = None;
        let mut ty = None;
        let mut args = Vec::new();
        let mut deprecation = None;
        let mut external = false;
        let mut provides = None;
        let mut requires = None;

        for meta in &ls.nested {
            match meta {
                NestedMeta::Meta(Meta::Path(p)) if p.is_ident("external") => {
                    external = true;
                }
                NestedMeta::Meta(Meta::NameValue(nv)) => {
                    if nv.path.is_ident("name") {
                        if let syn::Lit::Str(lit) = &nv.lit {
                            name = Some(lit.value());
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Attribute 'name' should be a string.",
                            ));
                        }
                    } else if nv.path.is_ident("method") {
                        if let syn::Lit::Str(lit) = &nv.lit {
                            method = Some(lit.value());
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Attribute 'method' should be a string.",
                            ));
                        }
                    } else if nv.path.is_ident("desc") {
                        if let syn::Lit::Str(lit) = &nv.lit {
                            desc = Some(lit.value());
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Attribute 'desc' should be a string.",
                            ));
                        }
                    } else if nv.path.is_ident("type") {
                        if let syn::Lit::Str(lit) = &nv.lit {
                            if let Ok(ty2) = syn::parse_str::<syn::Type>(&lit.value()) {
                                ty = Some(ty2);
                            } else {
                                return Err(Error::new_spanned(&lit, "Expect type"));
                            }
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Attribute 'type' should be a string.",
                            ));
                        }
                    } else if nv.path.is_ident("deprecation") {
                        if let syn::Lit::Str(lit) = &nv.lit {
                            deprecation = Some(lit.value());
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Attribute 'deprecation' should be a string.",
                            ));
                        }
                    } else if nv.path.is_ident("provides") {
                        if let syn::Lit::Str(lit) = &nv.lit {
                            provides = Some(lit.value());
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Attribute 'provides' should be a string.",
                            ));
                        }
                    } else if nv.path.is_ident("requires") {
                        if let syn::Lit::Str(lit) = &nv.lit {
                            requires = Some(lit.value());
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Attribute 'requires' should be a string.",
                            ));
                        }
                    }
                }
                NestedMeta::Meta(Meta::List(ls)) if ls.path.is_ident("arg") => {
                    args.push(InterfaceFieldArgument::parse(ls)?);
                }
                _ => {}
            }
        }

        if name.is_none() {
            return Err(Error::new_spanned(ls, "Missing name"));
        }

        if ty.is_none() {
            return Err(Error::new_spanned(ls, "Missing type"));
        }

        Ok(Self {
            name: name.unwrap(),
            method,
            desc,
            ty: ty.unwrap(),
            args,
            deprecation,
            external,
            requires,
            provides,
        })
    }
}

pub struct Interface {
    pub internal: bool,
    pub name: Option<String>,
    pub desc: Option<String>,
    pub fields: Vec<InterfaceField>,
    pub extends: bool,
}

impl Interface {
    pub fn parse(args: AttributeArgs) -> Result<Self> {
        let mut internal = false;
        let mut name = None;
        let mut desc = None;
        let mut fields = Vec::new();
        let mut extends = false;

        for arg in args {
            match arg {
                NestedMeta::Meta(Meta::Path(p)) if p.is_ident("internal") => {
                    internal = true;
                }
                NestedMeta::Meta(Meta::Path(p)) if p.is_ident("extends") => {
                    extends = true;
                }
                NestedMeta::Meta(Meta::NameValue(nv)) => {
                    if nv.path.is_ident("name") {
                        if let syn::Lit::Str(lit) = nv.lit {
                            name = Some(lit.value());
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Attribute 'name' should be a string.",
                            ));
                        }
                    } else if nv.path.is_ident("desc") {
                        if let syn::Lit::Str(lit) = nv.lit {
                            desc = Some(lit.value());
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Attribute 'desc' should be a string.",
                            ));
                        }
                    }
                }
                NestedMeta::Meta(Meta::List(ls)) if ls.path.is_ident("field") => {
                    fields.push(InterfaceField::parse(&ls)?);
                }
                _ => {}
            }
        }

        Ok(Self {
            internal,
            name,
            desc,
            fields,
            extends,
        })
    }
}

pub struct Scalar {
    pub internal: bool,
    pub name: Option<String>,
    pub desc: Option<String>,
}

impl Scalar {
    pub fn parse(args: AttributeArgs) -> Result<Self> {
        let mut internal = false;
        let mut name = None;
        let mut desc = None;

        for arg in args {
            match arg {
                NestedMeta::Meta(Meta::Path(p)) => {
                    if p.is_ident("internal") {
                        internal = true;
                    }
                }
                NestedMeta::Meta(Meta::NameValue(nv)) => {
                    if nv.path.is_ident("name") {
                        if let syn::Lit::Str(lit) = nv.lit {
                            name = Some(lit.value());
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Attribute 'name' should be a string.",
                            ));
                        }
                    } else if nv.path.is_ident("desc") {
                        if let syn::Lit::Str(lit) = nv.lit {
                            desc = Some(lit.value());
                        } else {
                            return Err(Error::new_spanned(
                                &nv.lit,
                                "Attribute 'desc' should be a string.",
                            ));
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(Self {
            internal,
            name,
            desc,
        })
    }
}

pub struct Entity {}

impl Entity {
    pub fn parse(attrs: &[Attribute]) -> Result<Option<Self>> {
        for attr in attrs {
            match attr.parse_meta()? {
                Meta::List(ls) if ls.path.is_ident("entity") => {
                    return Ok(Some(Self {}));
                }
                Meta::Path(p) if p.is_ident("entity") => {
                    return Ok(Some(Self {}));
                }
                _ => {}
            }
        }

        Ok(None)
    }
}
