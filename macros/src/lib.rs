#[allow(unused)]
mod helpers;

#[cfg(feature = "chrono")]
mod chrono_utils;

#[allow(unused_imports)]
use crate::helpers::HelperContext;

#[cfg(feature = "chrono")]
use chrono::{DateTime, Utc, Local, FixedOffset};

#[allow(unused_imports)]
use proc_macro::TokenStream;
#[allow(unused_imports)]
use proc_macro2::Span;
#[allow(unused_imports)]
use quote::ToTokens;
#[allow(unused_imports)]
use syn::parse_macro_input;
#[allow(unused_imports)]
use syn::parse::Parse;

#[allow(unused_imports)]
use std::sync::{LazyLock, OnceLock};
#[cfg(feature = "git")]
use std::process::Command;



#[allow(unused)]
fn compile_error(message: &str) -> TokenStream {
  TokenStream::from(quote::quote!{ compile_error!(#message) })
}

#[allow(unused)]
fn try_make<F, I, O>(input: TokenStream, f: F) -> TokenStream
where F: FnOnce(HelperContext, I) -> Result<O, String>, I: Parse, O: ToTokens {
  f(HelperContext::default(), parse_macro_input!(input as I))
    .map(|value| TokenStream::from(value.into_token_stream()))
    .unwrap_or_else(|error| compile_error(&error))
}

#[allow(unused)]
fn make<F, I, O>(input: TokenStream, f: F) -> TokenStream
where F: FnOnce(HelperContext, I) -> O, I: Parse, O: ToTokens {
  try_make(input, move |context, input| Ok(f(context, input)))
}

#[allow(unused)]
fn try_make_memoized<F, I, O>(input: TokenStream, store: &OnceLock<Result<O, String>>, f: F) -> TokenStream
where F: FnOnce(HelperContext, I) -> Result<O, String>, I: Parse, O: Clone + ToTokens {
  try_make(input, move |context, input| store.get_or_init(move || f(context, input)).clone())
}

#[allow(unused)]
fn make_memoized<F, I, O>(input: TokenStream, store: &OnceLock<O>, f: F) -> TokenStream
where F: FnOnce(HelperContext, I) -> O, I: Parse, O: Clone + ToTokens {
  make(input, move |context, input| store.get_or_init(move || f(context, input)).clone())
}



#[cfg(feature = "git")]
fn git(args: &[&str]) -> Result<String, String> {
  match Command::new("git").args(args).output() {
    Ok(output) => match output.status.success() {
      true => Ok(String::from_utf8_lossy(&output.stdout).into_owned()),
      false => {
        let status = output.status.code().unwrap_or(1);
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("git failed with status {status}: {stderr:?}"))
      }
    },
    Err(error) => Err(format!("git execution error: {error}"))
  }
}

#[cfg(feature = "git")]
static GIT_HASH: OnceLock<Result<String, String>> = OnceLock::new();

/// The full 40-char hash of the current git project's last commit.
#[cfg(feature = "git")]
#[proc_macro]
pub fn git_hash(input: TokenStream) -> TokenStream {
  try_make_memoized(input, &GIT_HASH, |_, _: syn::parse::Nothing| {
    Ok(git(&["rev-parse", "HEAD"])?.trim().to_owned())
  })
}

#[cfg(feature = "git")]
static GIT_HASH_SHORT: OnceLock<Result<String, String>> = OnceLock::new();

/// The short 7-char hash of the current git project's last commit.
#[cfg(feature = "git")]
#[proc_macro]
pub fn git_hash_short(input: TokenStream) -> TokenStream {
  try_make_memoized(input, &GIT_HASH_SHORT, |_, _: syn::parse::Nothing| {
    Ok(git(&["rev-parse", "--short", "HEAD"])?.trim().to_owned())
  })
}

#[cfg(feature = "git")]
static GIT_BRANCH: OnceLock<Result<String, String>> = OnceLock::new();

/// The name of the current git project's branch.
#[cfg(feature = "git")]
#[proc_macro]
pub fn git_branch(input: TokenStream) -> TokenStream {
  try_make_memoized(input, &GIT_BRANCH, |_, _: syn::parse::Nothing| {
    Ok(git(&["rev-parse", "--abbrev-ref", "HEAD"])?.trim().to_owned())
  })
}

#[cfg(feature = "git")]
static GIT_ROOT: OnceLock<Result<String, String>> = OnceLock::new();

/// The root path of the current git project.
#[cfg(feature = "git")]
#[proc_macro]
pub fn git_root(input: TokenStream) -> TokenStream {
  try_make_memoized(input, &GIT_ROOT, |_, _: syn::parse::Nothing| {
    Ok(git(&["rev-parse", "--show-toplevel"])?.trim().to_owned())
  })
}



#[cfg(feature = "chrono")]
static BUILD_CHRONO_DATETIME: LazyLock<DateTime<Utc>> = LazyLock::new(Utc::now);

#[cfg(feature = "chrono")]
fn get_build_chrono_datetime_utc() -> DateTime<Utc> {
  *BUILD_CHRONO_DATETIME
}

#[cfg(feature = "chrono")]
fn get_build_chrono_datetime_local_fixed() -> DateTime<FixedOffset> {
  BUILD_CHRONO_DATETIME.with_timezone(&Local).fixed_offset()
}

/// The [`Utc`] [`DateTime`] corresponding to when this project was built.
#[cfg(feature = "chrono")]
#[proc_macro]
pub fn build_chrono_datetime_utc(input: TokenStream) -> TokenStream {
  make(input, |context, syn::parse::Nothing| context.with(get_build_chrono_datetime_utc()))
}

/// The [`Utc`] [`DateTime`] corresponding to when this project was built, formatted with a format string.
#[cfg(feature = "chrono")]
#[proc_macro]
pub fn build_chrono_datetime_utc_format(input: TokenStream) -> TokenStream {
  make(input, |_, input: syn::LitStr| get_build_chrono_datetime_utc().format(&input.value()).to_string())
}

/// The [`Utc`] [`DateTime`] corresponding to when this project was built, in RFC 2822 format.
#[cfg(feature = "chrono")]
#[proc_macro]
pub fn build_chrono_datetime_utc_rfc2822(input: TokenStream) -> TokenStream {
  make(input, |_, syn::parse::Nothing| get_build_chrono_datetime_utc().to_rfc2822())
}

/// The [`Utc`] [`DateTime`] corresponding to when this project was built, in RFC 3339 and ISO 8601 format.
#[cfg(feature = "chrono")]
#[proc_macro]
pub fn build_chrono_datetime_utc_rfc3339(input: TokenStream) -> TokenStream {
  make(input, |_, syn::parse::Nothing| get_build_chrono_datetime_utc().to_rfc3339())
}

/// The local [`FixedOffset`] [`DateTime`] corresponding to when this project was built.
#[cfg(feature = "chrono")]
#[proc_macro]
pub fn build_chrono_datetime_local_fixed(input: TokenStream) -> TokenStream {
  make(input, |context, syn::parse::Nothing| context.with(get_build_chrono_datetime_local_fixed()))
}

/// The local [`FixedOffset`] [`DateTime`] corresponding to when this project was built, formatted with a format string.
#[cfg(feature = "chrono")]
#[proc_macro]
pub fn build_chrono_datetime_local_fixed_format(input: TokenStream) -> TokenStream {
  make(input, |_, input: syn::LitStr| get_build_chrono_datetime_local_fixed().format(&input.value()).to_string())
}

/// The local [`FixedOffset`] [`DateTime`] corresponding to when this project was built, in RFC 2822 format.
#[cfg(feature = "chrono")]
#[proc_macro]
pub fn build_chrono_datetime_local_fixed_rfc2822(input: TokenStream) -> TokenStream {
  make(input, |_, syn::parse::Nothing| get_build_chrono_datetime_local_fixed().to_rfc2822())
}

/// The local [`FixedOffset`] [`DateTime`] corresponding to when this project was built, in RFC 3339 and ISO 8601 format.
#[cfg(feature = "chrono")]
#[proc_macro]
pub fn build_chrono_datetime_local_fixed_rfc3339(input: TokenStream) -> TokenStream {
  make(input, |_, syn::parse::Nothing| get_build_chrono_datetime_local_fixed().to_rfc3339())
}



#[cfg(all(feature = "chrono", feature = "git"))]
static GIT_LAST_COMMIT_CHRONO_DATETIME: OnceLock<Result<DateTime<Utc>, String>> = OnceLock::new();

#[cfg(all(feature = "chrono", feature = "git"))]
fn get_git_last_commit_chrono_datetime_inner() -> &'static Result<DateTime<Utc>, String> {
  GIT_LAST_COMMIT_CHRONO_DATETIME.get_or_init(|| {
    git(&["log", "-1", "--format=%at", "--date=local"]).and_then(|output| {
      let timestamp_secs = output.trim().parse::<i64>()
        .map_err(|err| format!("failed to parse timestamp: {err} ({:?})", output.trim()))?;
      let datetime = DateTime::from_timestamp(timestamp_secs, 0)
        .ok_or_else(|| format!("failed to create datetime from {timestamp_secs:?}"))?;
      Ok(datetime)
    })
  })
}

#[cfg(all(feature = "chrono", feature = "git"))]
fn get_git_last_commit_chrono_datetime_utc() -> Result<DateTime<Utc>, String> {
  get_git_last_commit_chrono_datetime_inner().clone()
}

#[cfg(all(feature = "chrono", feature = "git"))]
fn get_git_last_commit_chrono_datetime_local_fixed() -> Result<DateTime<FixedOffset>, String> {
  get_git_last_commit_chrono_datetime_inner().as_ref()
    .map(|datetime| datetime.with_timezone(&Local).fixed_offset())
    .map_err(String::clone)
}

/// The [`Utc`] [`DateTime`] corresponding to when the current git project's last commit was made.
#[cfg(all(feature = "chrono", feature = "git"))]
#[proc_macro]
pub fn git_last_commit_chrono_datetime_utc(input: TokenStream) -> TokenStream {
  try_make(input, |context, syn::parse::Nothing| Ok(context.with(get_git_last_commit_chrono_datetime_utc()?)))
}

/// The [`Utc`] [`DateTime`] corresponding to when the current git project's last commit was made, formatted with a format string.
#[cfg(all(feature = "chrono", feature = "git"))]
#[proc_macro]
pub fn git_last_commit_chrono_datetime_utc_format(input: TokenStream) -> TokenStream {
  try_make(input, |_, input: syn::LitStr| Ok(get_git_last_commit_chrono_datetime_utc()?.format(&input.value()).to_string()))
}

/// The [`Utc`] [`DateTime`] corresponding to when the current git project's last commit was made, in RFC 2822 format.
#[cfg(all(feature = "chrono", feature = "git"))]
#[proc_macro]
pub fn git_last_commit_chrono_datetime_utc_rfc2822(input: TokenStream) -> TokenStream {
  try_make(input, |_, syn::parse::Nothing| Ok(get_git_last_commit_chrono_datetime_utc()?.to_rfc2822()))
}

/// The [`Utc`] [`DateTime`] corresponding to when the current git project's last commit was made, in RFC 3339 and ISO 8601 format.
#[cfg(all(feature = "chrono", feature = "git"))]
#[proc_macro]
pub fn git_last_commit_chrono_datetime_utc_rfc3339(input: TokenStream) -> TokenStream {
  try_make(input, |_, syn::parse::Nothing| Ok(get_git_last_commit_chrono_datetime_utc()?.to_rfc3339()))
}

/// The local [`FixedOffset`] [`DateTime`] corresponding to when the current git project's last commit was made.
#[cfg(all(feature = "chrono", feature = "git"))]
#[proc_macro]
pub fn git_last_commit_chrono_datetime_local_fixed(input: TokenStream) -> TokenStream {
  try_make(input, |context, syn::parse::Nothing| Ok(context.with(get_git_last_commit_chrono_datetime_local_fixed()?)))
}

/// The local [`FixedOffset`] [`DateTime`] corresponding to when the current git project's last commit was made, formatted with a format string.
#[cfg(all(feature = "chrono", feature = "git"))]
#[proc_macro]
pub fn git_last_commit_chrono_datetime_local_fixed_format(input: TokenStream) -> TokenStream {
  try_make(input, |_, input: syn::LitStr| Ok(get_git_last_commit_chrono_datetime_local_fixed()?.format(&input.value()).to_string()))
}

/// The local [`FixedOffset`] [`DateTime`] corresponding to when the current git project's last commit was made, in RFC 2822 format.
#[cfg(all(feature = "chrono", feature = "git"))]
#[proc_macro]
pub fn git_last_commit_chrono_datetime_local_fixed_rfc2822(input: TokenStream) -> TokenStream {
  try_make(input, |_, syn::parse::Nothing| Ok(get_git_last_commit_chrono_datetime_local_fixed()?.to_rfc2822()))
}

/// The local [`FixedOffset`] [`DateTime`] corresponding to when the current git project's last commit was made, in RFC 3339 and ISO 8601 format.
#[cfg(all(feature = "chrono", feature = "git"))]
#[proc_macro]
pub fn git_last_commit_chrono_datetime_local_fixed_rfc3339(input: TokenStream) -> TokenStream {
  try_make(input, |_, syn::parse::Nothing| Ok(get_git_last_commit_chrono_datetime_local_fixed()?.to_rfc3339()))
}
