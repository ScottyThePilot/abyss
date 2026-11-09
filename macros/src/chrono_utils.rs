#[cfg(feature = "chrono")]
use chrono::{DateTime, NaiveDateTime, NaiveDate, Datelike, NaiveTime, Timelike, Utc, TimeZone, FixedOffset};
use proc_macro2::TokenStream;
use quote::ToTokens;



#[derive(Debug, Clone)]
pub struct ToTokensDateTime<Tz: TimeZone> {
  abyss: syn::Ident,
  datetime: DateTime<Tz>
}

impl<Tz: TimeZone> ToTokensDateTime<Tz> {
  pub fn new(abyss: syn::Ident, datetime: DateTime<Tz>) -> Self {
    ToTokensDateTime { abyss, datetime }
  }
}

impl ToTokens for ToTokensDateTime<Utc> {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    tokens.extend(self.to_token_stream());
  }

  fn to_token_stream(&self) -> TokenStream {
    let abyss = &self.abyss;
    let chrono = quote::quote!{ #abyss::externs::chrono };
    let tokens_naive_datetime = ToTokensNaiveDateTime::new(abyss.clone(), self.datetime.naive_utc());

    quote::quote!{
      #chrono::DateTime::<#chrono::Utc>::from_naive_utc_and_offset(
        #tokens_naive_datetime,
        #chrono::Utc
      )
    }
  }
}

impl ToTokens for ToTokensDateTime<FixedOffset> {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    tokens.extend(self.to_token_stream());
  }

  fn to_token_stream(&self) -> TokenStream {
    let abyss = &self.abyss;
    let chrono = quote::quote!{ #abyss::externs::chrono };
    let local_minus_utc = self.datetime.offset().local_minus_utc();
    let tokens_naive_datetime = ToTokensNaiveDateTime::new(abyss.clone(), self.datetime.naive_utc());

    quote::quote!{
      #chrono::DateTime::<#chrono::FixedOffset>::from_naive_utc_and_offset(
        #tokens_naive_datetime,
        #chrono::FixedOffset::west_opt(#local_minus_utc).unwrap()
      )
    }
  }
}

#[derive(Debug, Clone)]
pub struct ToTokensNaiveDateTime {
  abyss: syn::Ident,
  naive_datetime: NaiveDateTime
}

impl ToTokensNaiveDateTime {
  pub fn new(abyss: syn::Ident, naive_datetime: NaiveDateTime) -> Self {
    ToTokensNaiveDateTime { abyss, naive_datetime }
  }
}

impl ToTokens for ToTokensNaiveDateTime {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    tokens.extend(self.to_token_stream());
  }

  fn to_token_stream(&self) -> TokenStream {
    let abyss = &self.abyss;
    let chrono = quote::quote!{ #abyss::externs::chrono };
    let tokens_naive_date = ToTokensNaiveDate::new(abyss.clone(), self.naive_datetime.date());
    let tokens_naive_time = ToTokensNaiveTime::new(abyss.clone(), self.naive_datetime.time());

    quote::quote!{
      #chrono::NaiveDateTime::new(#tokens_naive_date, #tokens_naive_time)
    }
  }
}

#[derive(Debug, Clone)]
pub struct ToTokensNaiveDate {
  abyss: syn::Ident,
  naive_nate: NaiveDate
}

impl ToTokensNaiveDate {
  pub fn new(abyss: syn::Ident, naive_nate: NaiveDate) -> Self {
    ToTokensNaiveDate { abyss, naive_nate }
  }
}

impl ToTokens for ToTokensNaiveDate {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    tokens.extend(self.to_token_stream());
  }

  fn to_token_stream(&self) -> TokenStream {
    let abyss = &self.abyss;
    let chrono = quote::quote!{ #abyss::externs::chrono };
    let naive_date_days = self.naive_nate.num_days_from_ce();

    quote::quote!{
      #chrono::NaiveDate::from_num_days_from_ce_opt(#naive_date_days).unwrap()
    }
  }
}

#[derive(Debug, Clone)]
pub struct ToTokensNaiveTime {
  abyss: syn::Ident,
  naive_time: NaiveTime
}

impl ToTokensNaiveTime {
  pub fn new(abyss: syn::Ident, naive_time: NaiveTime) -> Self {
    ToTokensNaiveTime { abyss, naive_time }
  }
}

impl ToTokens for ToTokensNaiveTime {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    tokens.extend(self.to_token_stream());
  }

  fn to_token_stream(&self) -> TokenStream {
    let abyss = &self.abyss;
    let chrono = quote::quote!{ #abyss::externs::chrono };
    let naive_time_secs = self.naive_time.num_seconds_from_midnight();
    let naive_time_frac = self.naive_time.nanosecond();

    quote::quote!{
      #chrono::NaiveTime::from_num_seconds_from_midnight_opt(#naive_time_secs, #naive_time_frac).unwrap()
    }
  }
}



#[cfg(test)]
mod tests {
  use chrono::{DateTime, Utc, NaiveDateTime, NaiveDate, NaiveTime, Datelike, Timelike};

  #[test]
  fn test_roundtrips() {
    let now = Utc::now();
    test_roundtrip(now);
  }

  fn test_roundtrip(datetime: DateTime<Utc>) {
    let naive_datetime = datetime.naive_utc();
    let naive_date_days = naive_datetime.date().num_days_from_ce();
    let naive_time_secs = naive_datetime.time().num_seconds_from_midnight();
    let naive_time_frac = naive_datetime.time().nanosecond();

    let datetime2 = DateTime::<Utc>::from_naive_utc_and_offset(
      NaiveDateTime::new(
        NaiveDate::from_num_days_from_ce_opt(naive_date_days).unwrap(),
        NaiveTime::from_num_seconds_from_midnight_opt(naive_time_secs, naive_time_frac).unwrap()
      ),
      Utc
    );

    assert_eq!(datetime, datetime2);
  }
}
