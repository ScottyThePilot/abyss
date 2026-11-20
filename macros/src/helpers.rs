use proc_macro2::{TokenStream, Span};
use quote::ToTokens;



#[derive(Debug, Clone)]
pub struct HelperContext {
  abyss: syn::Ident
}

impl HelperContext {
  pub const fn new(abyss: syn::Ident) -> Self {
    HelperContext { abyss }
  }

  pub fn get_extern(&self, name: &str) -> syn::Path {
    syn::Path {
      leading_colon: Some(<syn::Token![::]>::default()),
      segments: syn::punctuated::Punctuated::from_iter([
        syn::PathSegment::from(self.abyss.clone()),
        syn::PathSegment::from(syn::Ident::new("externs", Span::call_site())),
        syn::PathSegment::from(syn::Ident::new(name, Span::call_site()))
      ])
    }
  }

  pub fn with<T: ToTokensHelper>(self, value: T) -> Helper<T> {
    Helper::new(self, value)
  }
}

impl Default for HelperContext {
  fn default() -> Self {
    Self::new(syn::Ident::new("abyss", Span::call_site()))
  }
}



#[derive(Debug, Clone)]
pub struct Helper<T> {
  context: HelperContext,
  value: T
}

impl<T: ToTokensHelper> Helper<T> {
  pub const fn new(context: HelperContext, value: T) -> Self {
    Helper { context, value }
  }
}

impl<T: ToTokensHelper> ToTokens for Helper<T> {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    tokens.extend(self.to_token_stream());
  }

  fn to_token_stream(&self) -> TokenStream {
    self.value.to_token_stream(&self.context)
  }
}



pub trait ToTokensHelper: Sized {
  fn to_token_stream(&self, context: &HelperContext) -> TokenStream;
}

impl<T: ToTokensHelper> ToTokensHelper for &T {
  fn to_token_stream(&self, context: &HelperContext) -> TokenStream {
    T::to_token_stream(self, context)
  }
}
