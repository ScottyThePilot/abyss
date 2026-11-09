#[cfg(feature = "chrono")]
extern crate chrono;
extern crate proc_macro;
extern crate quote;
extern crate syn;

#[cfg(feature = "chrono")]
mod chrono_utils;

#[cfg(feature = "chrono")]
use chrono::{DateTime, Utc, Local, TimeZone, FixedOffset};
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

#[cfg(feature = "chrono")]
use crate::chrono_utils::*;

#[allow(unused_imports)]
use std::sync::{LazyLock, OnceLock};
#[cfg(feature = "git")]
use std::process::Command;



#[allow(unused)]
fn compile_error(message: &str) -> TokenStream {
  TokenStream::from(quote::quote!{ compile_error!(#message) })
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
  parse_macro_input!(input as syn::parse::Nothing);
  match GIT_HASH.get_or_init(|| git(&["rev-parse", "HEAD"])) {
    Ok(output) => TokenStream::from(output.trim().to_token_stream()),
    Err(error) => compile_error(error)
  }
}

#[cfg(feature = "git")]
static GIT_HASH_SHORT: OnceLock<Result<String, String>> = OnceLock::new();

/// The short 7-char hash of the current git project's last commit.
#[cfg(feature = "git")]
#[proc_macro]
pub fn git_hash_short(input: TokenStream) -> TokenStream {
  parse_macro_input!(input as syn::parse::Nothing);
  match GIT_HASH_SHORT.get_or_init(|| git(&["rev-parse", "--short", "HEAD"])) {
    Ok(output) => TokenStream::from(output.trim().to_token_stream()),
    Err(error) => compile_error(error)
  }
}

#[cfg(feature = "git")]
static GIT_BRANCH: OnceLock<Result<String, String>> = OnceLock::new();

/// The name of the current git project's branch.
#[cfg(feature = "git")]
#[proc_macro]
pub fn git_branch(input: TokenStream) -> TokenStream {
  parse_macro_input!(input as syn::parse::Nothing);
  match GIT_BRANCH.get_or_init(|| git(&["rev-parse", "--abbrev-ref", "HEAD"])) {
    Ok(output) => TokenStream::from(output.trim().to_token_stream()),
    Err(error) => compile_error(error)
  }
}

#[cfg(feature = "git")]
static GIT_ROOT: OnceLock<Result<String, String>> = OnceLock::new();

/// The root path of the current git project.
#[cfg(feature = "git")]
#[proc_macro]
pub fn git_root(input: TokenStream) -> TokenStream {
  parse_macro_input!(input as syn::parse::Nothing);
  match GIT_ROOT.get_or_init(|| git(&["rev-parse", "--show-toplevel"])) {
    Ok(output) => TokenStream::from(output.trim().to_token_stream()),
    Err(error) => compile_error(error)
  }
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

#[cfg(feature = "chrono")]
fn try_build_chrono<F, Tz, I, O>(input: TokenStream, get: impl FnOnce() -> Result<DateTime<Tz>, &'static str>, f: F) -> TokenStream
where F: FnOnce(I, syn::Ident, DateTime<Tz>) -> O, Tz: TimeZone, I: Parse, O: ToTokens {
  let input = parse_macro_input!(input as I);
  let abyss = syn::Ident::new("_abyss", Span::call_site());
  let token_stream = f(input, abyss.clone(), match get() {
    Ok(datetime) => datetime,
    Err(error) => return compile_error(error)
  });

  TokenStream::from(quote::quote!{
    {
      #[allow(unused_extern_crates, clippy::useless_attribute)]
      extern crate abyss as #abyss;
      #token_stream
    }
  })
}

#[cfg(feature = "chrono")]
fn try_build_chrono_bare<F, Tz, I, O>(input: TokenStream, get: impl FnOnce() -> Result<DateTime<Tz>, &'static str>, f: F) -> TokenStream
where F: FnOnce(I, DateTime<Tz>) -> O, Tz: TimeZone, I: Parse, O: ToTokens {
  let input = parse_macro_input!(input as I);
  let token_stream = f(input, match get() {
    Ok(datetime) => datetime,
    Err(error) => return compile_error(error)
  });

  TokenStream::from(token_stream.into_token_stream())
}

#[cfg(feature = "chrono")]
fn build_chrono<F, Tz, I, O>(input: TokenStream, get: impl FnOnce() -> DateTime<Tz>, f: F) -> TokenStream
where F: FnOnce(I, syn::Ident, DateTime<Tz>) -> O, Tz: TimeZone, I: Parse, O: ToTokens {
  try_build_chrono(input, move || Ok(get()), f)
}

#[cfg(feature = "chrono")]
fn build_chrono_bare<F, Tz, I, O>(input: TokenStream, get: impl FnOnce() -> DateTime<Tz>, f: F) -> TokenStream
where F: FnOnce(I, DateTime<Tz>) -> O, Tz: TimeZone, I: Parse, O: ToTokens {
  try_build_chrono_bare(input, move || Ok(get()), f)
}

/// The [`Utc`] [`DateTime`] corresponding to when this project was built.
#[cfg(feature = "chrono")]
#[proc_macro]
pub fn build_chrono_datetime_utc(input: TokenStream) -> TokenStream {
  build_chrono(input, get_build_chrono_datetime_utc, |syn::parse::Nothing, abyss, datetime| {
    ToTokensDateTime::<Utc>::new(abyss, datetime).to_token_stream()
  })
}

/// The [`Utc`] [`DateTime`] corresponding to when this project was built, formatted with a format string.
#[cfg(feature = "chrono")]
#[proc_macro]
pub fn build_chrono_datetime_utc_format(input: TokenStream) -> TokenStream {
  build_chrono_bare(input, get_build_chrono_datetime_utc, |input: syn::LitStr, datetime| {
    datetime.format(&input.value()).to_string()
  })
}

/// The [`Utc`] [`DateTime`] corresponding to when this project was built, in RFC 2822 format.
#[cfg(feature = "chrono")]
#[proc_macro]
pub fn build_chrono_datetime_utc_rfc2822(input: TokenStream) -> TokenStream {
  build_chrono_bare(input, get_build_chrono_datetime_utc, |syn::parse::Nothing, datetime| datetime.to_rfc2822())
}

/// The [`Utc`] [`DateTime`] corresponding to when this project was built, in RFC 3339 and ISO 8601 format.
#[cfg(feature = "chrono")]
#[proc_macro]
pub fn build_chrono_datetime_utc_rfc3339(input: TokenStream) -> TokenStream {
  build_chrono_bare(input, get_build_chrono_datetime_utc, |syn::parse::Nothing, datetime| datetime.to_rfc3339())
}

/// The local [`FixedOffset`] [`DateTime`] corresponding to when this project was built.
#[cfg(feature = "chrono")]
#[proc_macro]
pub fn build_chrono_datetime_local_fixed(input: TokenStream) -> TokenStream {
  build_chrono(input, get_build_chrono_datetime_local_fixed, |syn::parse::Nothing, abyss, datetime| {
    ToTokensDateTime::<FixedOffset>::new(abyss, datetime).to_token_stream()
  })
}

/// The local [`FixedOffset`] [`DateTime`] corresponding to when this project was built, formatted with a format string.
#[cfg(feature = "chrono")]
#[proc_macro]
pub fn build_chrono_datetime_local_fixed_format(input: TokenStream) -> TokenStream {
  build_chrono_bare(input, get_build_chrono_datetime_local_fixed, |input: syn::LitStr, datetime| {
    datetime.format(&input.value()).to_string()
  })
}

/// The local [`FixedOffset`] [`DateTime`] corresponding to when this project was built, in RFC 2822 format.
#[cfg(feature = "chrono")]
#[proc_macro]
pub fn build_chrono_datetime_local_fixed_rfc2822(input: TokenStream) -> TokenStream {
  build_chrono_bare(input, get_build_chrono_datetime_local_fixed, |syn::parse::Nothing, datetime| datetime.to_rfc2822())
}

/// The local [`FixedOffset`] [`DateTime`] corresponding to when this project was built, in RFC 3339 and ISO 8601 format.
#[cfg(feature = "chrono")]
#[proc_macro]
pub fn build_chrono_datetime_local_fixed_rfc3339(input: TokenStream) -> TokenStream {
  build_chrono_bare(input, get_build_chrono_datetime_local_fixed, |syn::parse::Nothing, datetime| datetime.to_rfc3339())
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
fn get_git_last_commit_chrono_datetime_utc() -> Result<DateTime<Utc>, &'static str> {
  get_git_last_commit_chrono_datetime_inner().as_ref().copied().map_err(String::as_str)
}

#[cfg(all(feature = "chrono", feature = "git"))]
fn get_git_last_commit_chrono_datetime_local_fixed() -> Result<DateTime<FixedOffset>, &'static str> {
  get_git_last_commit_chrono_datetime_inner().as_ref()
    .map(|datetime| datetime.with_timezone(&Local).fixed_offset())
    .map_err(String::as_str)
}

/// The [`Utc`] [`DateTime`] corresponding to when the current git project's last commit was made.
#[cfg(all(feature = "chrono", feature = "git"))]
#[proc_macro]
pub fn git_last_commit_chrono_datetime_utc(input: TokenStream) -> TokenStream {
  try_build_chrono(input, get_git_last_commit_chrono_datetime_utc, |syn::parse::Nothing, abyss, datetime| {
    ToTokensDateTime::new(abyss, datetime).to_token_stream()
  })
}

/// The [`Utc`] [`DateTime`] corresponding to when the current git project's last commit was made, formatted with a format string.
#[cfg(all(feature = "chrono", feature = "git"))]
#[proc_macro]
pub fn git_last_commit_chrono_datetime_utc_format(input: TokenStream) -> TokenStream {
  try_build_chrono_bare(input, get_git_last_commit_chrono_datetime_utc, |input: syn::LitStr, datetime| {
    datetime.format(&input.value()).to_string()
  })
}

/// The [`Utc`] [`DateTime`] corresponding to when the current git project's last commit was made, in RFC 2822 format.
#[cfg(all(feature = "chrono", feature = "git"))]
#[proc_macro]
pub fn git_last_commit_chrono_datetime_utc_rfc2822(input: TokenStream) -> TokenStream {
  try_build_chrono_bare(input, get_git_last_commit_chrono_datetime_utc, |syn::parse::Nothing, datetime| datetime.to_rfc2822())
}

/// The [`Utc`] [`DateTime`] corresponding to when the current git project's last commit was made, in RFC 3339 and ISO 8601 format.
#[cfg(all(feature = "chrono", feature = "git"))]
#[proc_macro]
pub fn git_last_commit_chrono_datetime_utc_rfc3339(input: TokenStream) -> TokenStream {
  try_build_chrono_bare(input, get_git_last_commit_chrono_datetime_utc, |syn::parse::Nothing, datetime| datetime.to_rfc3339())
}

/// The local [`FixedOffset`] [`DateTime`] corresponding to when the current git project's last commit was made.
#[cfg(all(feature = "chrono", feature = "git"))]
#[proc_macro]
pub fn git_last_commit_chrono_datetime_local_fixed(input: TokenStream) -> TokenStream {
  try_build_chrono(input, get_git_last_commit_chrono_datetime_local_fixed, |syn::parse::Nothing, abyss, datetime| {
    ToTokensDateTime::new(abyss, datetime).to_token_stream()
  })
}

/// The local [`FixedOffset`] [`DateTime`] corresponding to when the current git project's last commit was made, formatted with a format string.
#[cfg(all(feature = "chrono", feature = "git"))]
#[proc_macro]
pub fn git_last_commit_chrono_datetime_local_fixed_format(input: TokenStream) -> TokenStream {
  try_build_chrono_bare(input, get_git_last_commit_chrono_datetime_local_fixed, |input: syn::LitStr, datetime| {
    datetime.format(&input.value()).to_string()
  })
}

/// The local [`FixedOffset`] [`DateTime`] corresponding to when the current git project's last commit was made, in RFC 2822 format.
#[cfg(all(feature = "chrono", feature = "git"))]
#[proc_macro]
pub fn git_last_commit_chrono_datetime_local_fixed_rfc2822(input: TokenStream) -> TokenStream {
  try_build_chrono_bare(input, get_git_last_commit_chrono_datetime_local_fixed, |syn::parse::Nothing, datetime| datetime.to_rfc2822())
}

/// The local [`FixedOffset`] [`DateTime`] corresponding to when the current git project's last commit was made, in RFC 3339 and ISO 8601 format.
#[cfg(all(feature = "chrono", feature = "git"))]
#[proc_macro]
pub fn git_last_commit_chrono_datetime_local_fixed_rfc3339(input: TokenStream) -> TokenStream {
  try_build_chrono_bare(input, get_git_last_commit_chrono_datetime_local_fixed, |syn::parse::Nothing, datetime| datetime.to_rfc3339())
}
