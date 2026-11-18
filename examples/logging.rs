extern crate abyss;

use abyss::logging::*;
use abyss::externs::log::{trace, debug, info, warn, error};

fn main() {
  log::set_max_level(log::STATIC_MAX_LEVEL);

  Rich::new()
    .with_format(|callback, message, record| {
      callback.finish(format_args!(
        "[{level}]({target}) {message}",
        level = record.level(),
        target = record.target()
      ))
    })
    .with_level(log::LevelFilter::Trace)
    .with_child(std::io::stdout())
    .apply()
    .expect("error");

  println!("{:?} {:?}", log::STATIC_MAX_LEVEL, log::max_level());

  trace!("this is a trace log");
  debug!("this is a debug log");
  info!("this is an info log");
  warn!("this is a warn log");
  error!("this is an error log");
}
