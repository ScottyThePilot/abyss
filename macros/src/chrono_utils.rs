use crate::helpers::{HelperContext, ToTokensHelper};

use chrono::{DateTime, NaiveDateTime, NaiveDate, Datelike, NaiveTime, Timelike, Utc, FixedOffset};
use proc_macro2::TokenStream;



impl ToTokensHelper for DateTime<Utc> {
  fn to_token_stream(&self, context: &HelperContext) -> TokenStream {
    let chrono = context.get_extern("chrono");
    let tokens_naive_datetime = context.clone().with(self.naive_utc());

    quote::quote!{
      #chrono::DateTime::<#chrono::Utc>::from_naive_utc_and_offset(
        #tokens_naive_datetime,
        #chrono::Utc
      )
    }
  }
}

impl ToTokensHelper for DateTime<FixedOffset> {
  fn to_token_stream(&self, context: &HelperContext) -> TokenStream {
    let chrono = context.get_extern("chrono");
    let local_minus_utc = self.offset().local_minus_utc();
    let tokens_naive_datetime = context.clone().with(self.naive_utc());

    quote::quote!{
      #chrono::DateTime::<#chrono::FixedOffset>::from_naive_utc_and_offset(
        #tokens_naive_datetime,
        #chrono::FixedOffset::west_opt(#local_minus_utc).unwrap()
      )
    }
  }
}

impl ToTokensHelper for NaiveDateTime {
  fn to_token_stream(&self, context: &HelperContext) -> TokenStream {
    let chrono = context.get_extern("chrono");
    let tokens_naive_date = context.clone().with(self.date());
    let tokens_naive_time = context.clone().with(self.time());

    quote::quote!{
      #chrono::NaiveDateTime::new(#tokens_naive_date, #tokens_naive_time)
    }
  }
}

impl ToTokensHelper for NaiveDate {
  fn to_token_stream(&self, context: &HelperContext) -> TokenStream {
    let chrono = context.get_extern("chrono");
    let naive_date_days = self.num_days_from_ce();

    quote::quote!{
      #chrono::NaiveDate::from_num_days_from_ce_opt(#naive_date_days).unwrap()
    }
  }
}

impl ToTokensHelper for NaiveTime {
  fn to_token_stream(&self, context: &HelperContext) -> TokenStream {
    let chrono = context.get_extern("chrono");
    let naive_time_secs = self.num_seconds_from_midnight();
    let naive_time_frac = self.nanosecond();

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
