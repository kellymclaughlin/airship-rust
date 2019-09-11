//! # derive `Webmachine` for structs and enums
//!
//! This allows airship users to derive a default `Webmachine` implementation
//! for structs or to combine multiple resources that each implement the
//! `Webmachine` trait into a single application into an `enum` and derive the
//! `Webmachine` implementation for that `enum`. The airship `RoutingSpec` is
//! specified using a `std::Vec`, but rust collections may only contain a items
//! of a single concrete type. The use of an `enum` to represent all of the
//! airship application's resources provides that one concrete type and using
//! `#[derive(Webmachine)]` with the `enum` avoids any extra boilerplate in the
//! application.

extern crate proc_macro;

use proc_macro::TokenStream;

use quote::quote;
use syn;
use syn::{Ident, Variant};
use syn::punctuated::Punctuated;
use syn::token::Comma;

#[proc_macro_derive(Webmachine)]
pub fn webmachine_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_webmachine(&ast)
}

fn impl_webmachine(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let data = &ast.data;

    match data {
        syn::Data::Enum(enum_data) => {
            let variants = &enum_data.variants;
            let gen = impl_webmachine_enum_variants(name, variants);
            gen.into()
        },
        syn::Data::Struct(_struct_data) => {
            let gen = quote! {
                impl Webmachine for #name {}
            };
            gen.into()
        },
        _ => panic!("#[derive(Webmachine)] only supports struct and enum types")
    }

}

fn impl_webmachine_enum_variants(
    name: &syn::Ident,
    variants: &Punctuated<Variant, Comma>
) -> proc_macro2::TokenStream
{
    let allow_missing_post_variants = impl_allow_missing_post(name, variants);
    let allowed_methods_variants = impl_allowed_methods(name, variants);
    let content_types_accepted_variants = impl_content_types_accepted(name, variants);
    let content_types_provided_variants = impl_content_types_provided(name, variants);
    let delete_completed_variants = impl_delete_completed(name, variants);
    let delete_resource_variants = impl_delete_resource(name, variants);
    let entity_too_large_variants = impl_entity_too_large(name, variants);
    let forbidden_variants = impl_forbidden(name, variants);
    let generate_etag_variants = impl_generate_etag(name, variants);
    let implemented_variants = impl_implemented(name, variants);
    let is_authorized_variants = impl_is_authorized(name, variants);
    let is_conflict_variants = impl_is_conflict(name, variants);
    let known_content_type_variants = impl_known_content_type(name, variants);
    let last_modified_variants = impl_last_modified(name, variants);
    let language_available_variants = impl_language_available(name, variants);
    let malformed_request_variants = impl_malformed_request(name, variants);
    let moved_permanently_variants = impl_moved_permanently(name, variants);
    let moved_temporarily_variants = impl_moved_temporarily(name, variants);
    let multiple_choices_variants = impl_multiple_choices(name, variants);
    let patch_content_types_accepted_variants = impl_patch_content_types_accepted(name, variants);
    let previously_existed_variants = impl_previously_existed(name, variants);
    let process_post_variants = impl_process_post(name, variants);
    let resource_exists_variants = impl_resource_exists(name, variants);
    let service_available_variants = impl_service_available(name, variants);
    let uri_too_long_variants = impl_uri_too_long(name, variants);
    let valid_content_headers_variants = impl_valid_content_headers(name, variants);

    quote! {
        impl Webmachine for #name {
            #allow_missing_post_variants

            #allowed_methods_variants

            #content_types_accepted_variants

            #content_types_provided_variants

            #delete_completed_variants

            #delete_resource_variants

            #entity_too_large_variants

            #forbidden_variants

            #generate_etag_variants

            #implemented_variants

            #is_authorized_variants

            #is_conflict_variants

            #known_content_type_variants

            #last_modified_variants

            #language_available_variants

            #malformed_request_variants

            #moved_permanently_variants

            #moved_temporarily_variants

            #multiple_choices_variants

            #patch_content_types_accepted_variants

            #previously_existed_variants

            #process_post_variants

            #resource_exists_variants

            #service_available_variants

            #uri_too_long_variants

            #valid_content_headers_variants
        }
    }
}


fn impl_webmachine_enum_variant(name: &Ident, callback_method: &proc_macro2::TokenStream, trailing_args: &proc_macro2::TokenStream, variant: &Variant) -> proc_macro2::TokenStream {
    let id = &variant.ident;
    match variant.fields {
        syn::Fields::Unnamed(ref fields) => {
            match fields.unnamed.len() {
                0 => {
                    panic!("#[derive(Webmachine)] does not support tuple variants with no fields")

                }
                1 => {
                    quote! {
                        #name::#id(ref inner) => {
                            airship::resource::Webmachine::#callback_method(inner#trailing_args)
                        }
                    }
                }
                _ => {
                    panic!("#[derive(Webmachine)] does not support tuple variants with more than one \
                            fields")
                }
            }
        }
        _ => panic!("#[derive(Webmachine)] works only with unnamed variants"),
    }
}

fn impl_allow_missing_post(
    name: &syn::Ident,
    variants: &Punctuated<Variant, Comma>
) -> proc_macro2::TokenStream
{
    let callback_method = quote! {
        allow_missing_post
    };
    let trailing_args = quote! {};
    let variants = variants
        .iter()
        .map(|variant| impl_webmachine_enum_variant(name, &callback_method, &trailing_args, variant));

    quote! {
        fn allow_missing_post(&self) -> bool {
            match *self {
                #(#variants)*
            }
        }
    }
}

fn impl_allowed_methods(
    name: &syn::Ident,
    variants: &Punctuated<Variant, Comma>
) -> proc_macro2::TokenStream
{
    let callback_method = quote! {
        allowed_methods
    };
    let trailing_args = quote! {};
    let variants = variants
        .iter()
        .map(|variant| impl_webmachine_enum_variant(name, &callback_method, &trailing_args, variant));

    quote! {
        fn allowed_methods(&self) -> Vec<Method> {
            match *self {
                #(#variants)*
            }
        }
    }
}

fn impl_content_types_accepted(
    name: &syn::Ident,
    variants: &Punctuated<Variant, Comma>
) -> proc_macro2::TokenStream
{
    let callback_method = quote! {
        content_types_accepted
    };
    let trailing_args = quote! {};
    let variants = variants
        .iter()
        .map(|variant| impl_webmachine_enum_variant(name, &callback_method, &trailing_args, variant));

    quote! {
        fn content_types_accepted(&self) -> Vec<(Mime, fn(&Request))> {
            match *self {
                #(#variants)*
            }
        }
    }
}

fn impl_content_types_provided(
    name: &syn::Ident,
    variants: &Punctuated<Variant, Comma>
) -> proc_macro2::TokenStream
{
    let callback_method = quote! {
        content_types_provided
    };
    let trailing_args = quote! {};
    let variants = variants
        .iter()
        .map(|variant| impl_webmachine_enum_variant(name, &callback_method, &trailing_args, variant));

    quote! {
        fn content_types_provided(&self) -> Vec<(Mime, fn(&Request) -> Body)> {
            match *self {
                #(#variants)*
            }
        }
    }
}

fn impl_delete_completed(
    name: &syn::Ident,
    variants: &Punctuated<Variant, Comma>
) -> proc_macro2::TokenStream
{
    let callback_method = quote! {
        delete_completed
    };
    let trailing_args = quote! {};
    let variants = variants
        .iter()
        .map(|variant| impl_webmachine_enum_variant(name, &callback_method, &trailing_args, variant));

    quote! {
        fn delete_completed(&self) -> bool {
            match *self {
                #(#variants)*
            }
        }
    }
}

fn impl_delete_resource(
    name: &syn::Ident,
    variants: &Punctuated<Variant, Comma>
) -> proc_macro2::TokenStream
{
    let callback_method = quote! {
        delete_resource
    };
    let trailing_args = quote! {
        , req
    };
    let variants = variants
        .iter()
        .map(|variant| impl_webmachine_enum_variant(name, &callback_method, &trailing_args, variant));

    quote! {
        fn delete_resource(&self, req: &Request) -> bool {
            match *self {
                #(#variants)*
            }
        }
    }
}

fn impl_entity_too_large(
    name: &syn::Ident,
    variants: &Punctuated<Variant, Comma>
) -> proc_macro2::TokenStream
{
    let callback_method = quote! {
        entity_too_large
    };
    let trailing_args = quote! {
        , req
    };
    let variants = variants
        .iter()
        .map(|variant| impl_webmachine_enum_variant(name, &callback_method, &trailing_args, variant));

    quote! {
        fn entity_too_large(&self, req: &Request) -> bool {
            match *self {
                #(#variants)*
            }
        }
    }
}

fn impl_forbidden(
    name: &syn::Ident,
    variants: &Punctuated<Variant, Comma>
) -> proc_macro2::TokenStream
{
    let callback_method = quote! {
        forbidden
    };
    let trailing_args = quote! {
        , req
    };
    let variants = variants
        .iter()
        .map(|variant| impl_webmachine_enum_variant(name, &callback_method, &trailing_args, variant));

    quote! {
        fn forbidden(&self, req: &Request) -> bool {
            match *self {
                #(#variants)*
            }
        }
    }
}

fn impl_generate_etag(
    name: &syn::Ident,
    variants: &Punctuated<Variant, Comma>
) -> proc_macro2::TokenStream
{
    let callback_method = quote! {
        generate_etag
    };
    let trailing_args = quote! {
        , req
    };
    let variants = variants
        .iter()
        .map(|variant| impl_webmachine_enum_variant(name, &callback_method, &trailing_args, variant));

    quote! {
        fn generate_etag(&self, req: &Request) -> Option<hyper::header::EntityTag> {
            match *self {
                #(#variants)*
            }
        }
    }
}

fn impl_implemented(
    name: &syn::Ident,
    variants: &Punctuated<Variant, Comma>
) -> proc_macro2::TokenStream
{
    let callback_method = quote! {
        implemented
    };
    let trailing_args = quote! {};
    let variants = variants
        .iter()
        .map(|variant| impl_webmachine_enum_variant(name, &callback_method, &trailing_args, variant));

    quote! {
        fn implemented(&self) -> bool {
            match *self {
                #(#variants)*
            }
        }
    }
}

fn impl_is_authorized(
    name: &syn::Ident,
    variants: &Punctuated<Variant, Comma>
) -> proc_macro2::TokenStream
{
    let callback_method = quote! {
        is_authorized
    };
    let trailing_args = quote! {
        , req
    };
    let variants = variants
        .iter()
        .map(|variant| impl_webmachine_enum_variant(name, &callback_method, &trailing_args, variant));

    quote! {
        fn is_authorized(&self, req: &Request) -> bool {
            match *self {
                #(#variants)*
            }
        }
    }
}

fn impl_is_conflict(
    name: &syn::Ident,
    variants: &Punctuated<Variant, Comma>
) -> proc_macro2::TokenStream
{
    let callback_method = quote! {
        is_conflict
    };
    let trailing_args = quote! {};
    let variants = variants
        .iter()
        .map(|variant| impl_webmachine_enum_variant(name, &callback_method, &trailing_args, variant));

    quote! {
        fn is_conflict(&self) -> bool {
            match *self {
                #(#variants)*
            }
        }
    }
}

fn impl_known_content_type(
    name: &syn::Ident,
    variants: &Punctuated<Variant, Comma>
) -> proc_macro2::TokenStream
{
    let callback_method = quote! {
        known_content_type
    };
    let trailing_args = quote! {
        , req
    };
    let variants = variants
        .iter()
        .map(|variant| impl_webmachine_enum_variant(name, &callback_method, &trailing_args, variant));

    quote! {
        fn known_content_type(&self, req: &Request) -> bool {
            match *self {
                #(#variants)*
            }
        }
    }
}

fn impl_last_modified(
    name: &syn::Ident,
    variants: &Punctuated<Variant, Comma>
) -> proc_macro2::TokenStream
{
    let callback_method = quote! {
        last_modified
    };
    let trailing_args = quote! {};
    let variants = variants
        .iter()
        .map(|variant| impl_webmachine_enum_variant(name, &callback_method, &trailing_args, variant));

    quote! {
        fn last_modified(&self) -> Option<hyper::header::HttpDate> {
            match *self {
                #(#variants)*
            }
        }
    }
}

fn impl_language_available(
    name: &syn::Ident,
    variants: &Punctuated<Variant, Comma>
) -> proc_macro2::TokenStream
{
    let callback_method = quote! {
        language_available
    };
    let trailing_args = quote! {
        , accept_lang_header
    };
    let variants = variants
        .iter()
        .map(|variant| impl_webmachine_enum_variant(name, &callback_method, &trailing_args, variant));

    quote! {
        fn language_available<H: hyper::header::Header>(&self, accept_lang_header: &H) -> bool {
            match *self {
                #(#variants)*
            }
        }
    }
}

fn impl_malformed_request(
    name: &syn::Ident,
    variants: &Punctuated<Variant, Comma>
) -> proc_macro2::TokenStream
{
    let callback_method = quote! {
        malformed_request
    };
    let trailing_args = quote! {
        , req
    };
    let variants = variants
        .iter()
        .map(|variant| impl_webmachine_enum_variant(name, &callback_method, &trailing_args, variant));

    quote! {
        fn malformed_request(&self, req: &Request) -> bool {
            match *self {
                #(#variants)*
            }
        }
    }
}

fn impl_moved_permanently(
    name: &syn::Ident,
    variants: &Punctuated<Variant, Comma>
) -> proc_macro2::TokenStream
{
    let callback_method = quote! {
        moved_permanently
    };
    let trailing_args = quote! {};
    let variants = variants
        .iter()
        .map(|variant| impl_webmachine_enum_variant(name, &callback_method, &trailing_args, variant));

    quote! {
        fn moved_permanently(&self) -> Option<String> {
            match *self {
                #(#variants)*
            }
        }
    }
}

fn impl_moved_temporarily(
    name: &syn::Ident,
    variants: &Punctuated<Variant, Comma>
) -> proc_macro2::TokenStream
{
    let callback_method = quote! {
        moved_temporarily
    };
    let trailing_args = quote! {};
    let variants = variants
        .iter()
        .map(|variant| impl_webmachine_enum_variant(name, &callback_method, &trailing_args, variant));

    quote! {
        fn moved_temporarily(&self) -> Option<String> {
            match *self {
                #(#variants)*
            }
        }
    }
}

fn impl_multiple_choices(
    name: &syn::Ident,
    variants: &Punctuated<Variant, Comma>
) -> proc_macro2::TokenStream
{
    let callback_method = quote! {
        multiple_choices
    };
    let trailing_args = quote! {};
    let variants = variants
        .iter()
        .map(|variant| impl_webmachine_enum_variant(name, &callback_method, &trailing_args, variant));

    quote! {
        fn multiple_choices(&self) -> bool {
            match *self {
                #(#variants)*
            }
        }
    }
}

fn impl_patch_content_types_accepted(
    name: &syn::Ident,
    variants: &Punctuated<Variant, Comma>
) -> proc_macro2::TokenStream
{
    let callback_method = quote! {
        patch_content_types_accepted
    };
    let trailing_args = quote! {};
    let variants = variants
        .iter()
        .map(|variant| impl_webmachine_enum_variant(name, &callback_method, &trailing_args, variant));

    quote! {
        fn patch_content_types_accepted(&self) -> Vec<(Mime, fn(&Request))> {
            match *self {
                #(#variants)*
            }
        }
    }
}

fn impl_previously_existed(
    name: &syn::Ident,
    variants: &Punctuated<Variant, Comma>
) -> proc_macro2::TokenStream
{
    let callback_method = quote! {
        previously_existed
    };
    let trailing_args = quote! {};
    let variants = variants
        .iter()
        .map(|variant| impl_webmachine_enum_variant(name, &callback_method, &trailing_args, variant));

    quote! {
        fn previously_existed(&self) -> bool {
            match *self {
                #(#variants)*
            }
        }
    }
}

fn impl_process_post(
    name: &syn::Ident,
    variants: &Punctuated<Variant, Comma>
) -> proc_macro2::TokenStream
{
    let callback_method = quote! {
        process_post
    };
    let trailing_args = quote! {
        , req
    };
    let variants = variants
        .iter()
        .map(|variant| impl_webmachine_enum_variant(name, &callback_method, &trailing_args, variant));

    quote! {
        fn process_post(&self, req: &Request) -> airship::resource::PostResponse {
            match *self {
                #(#variants)*
            }
        }
    }
}

fn impl_resource_exists(
    name: &syn::Ident,
    variants: &Punctuated<Variant, Comma>
) -> proc_macro2::TokenStream
{
    let callback_method = quote! {
        resource_exists
    };
    let trailing_args = quote! {};
    let variants = variants
        .iter()
        .map(|variant| impl_webmachine_enum_variant(name, &callback_method, &trailing_args, variant));

    quote! {
        fn resource_exists(&self) -> bool {
            match *self {
                #(#variants)*
            }
        }
    }
}

fn impl_service_available(
    name: &syn::Ident,
    variants: &Punctuated<Variant, Comma>
) -> proc_macro2::TokenStream
{
    let callback_method = quote! {
        service_available
    };
    let trailing_args = quote! {};
    let variants = variants
        .iter()
        .map(|variant| impl_webmachine_enum_variant(name, &callback_method, &trailing_args, variant));

    quote! {
        fn service_available(&self) -> bool {
            match *self {
                #(#variants)*
            }
        }
    }
}

fn impl_uri_too_long(
    name: &syn::Ident,
    variants: &Punctuated<Variant, Comma>
) -> proc_macro2::TokenStream
{
    let callback_method = quote! {
        uri_too_long
    };
    let trailing_args = quote! {
        , uri
    };
    let variants = variants
        .iter()
        .map(|variant| impl_webmachine_enum_variant(name, &callback_method, &trailing_args, variant));

    quote! {
        fn uri_too_long(&self, uri: &hyper::Uri) -> bool {
            match *self {
                #(#variants)*
            }
        }
    }
}

fn impl_valid_content_headers(
    name: &syn::Ident,
    variants: &Punctuated<Variant, Comma>
) -> proc_macro2::TokenStream
{
    let callback_method = quote! {
        valid_content_headers
    };
    let trailing_args = quote! {
        , req
    };
    let variants = variants
        .iter()
        .map(|variant| impl_webmachine_enum_variant(name, &callback_method, &trailing_args, variant));

    quote! {
        fn valid_content_headers(&self, req: &Request) -> bool {
            match *self {
                #(#variants)*
            }
        }
    }
}
