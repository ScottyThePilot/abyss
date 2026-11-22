//! A set of logger helper traits for easier error handling while logging.

use bytemuck::{TransparentWrapper, TransparentWrapperAlloc};
use log::{Log, Metadata, Record, SetLoggerError};

use std::convert::Infallible;
use std::fmt;
use std::io;
use std::io::prelude::*;
use std::sync::mpsc::{Sender, SendError};
use std::sync::Mutex;



pub trait OutputExt: Output {
  fn as_dyn_log(&self) -> &dyn Log
  where Self: Sized, Self::Error: fmt::Display;
  fn into_dyn_log(self: Box<Self>) -> Box<dyn Log>
  where Self: Sized + 'static, Self::Error: fmt::Display;

  fn apply(self) -> Result<(), SetLoggerError>
  where Self: Sized + 'static, Self::Error: fmt::Display;
  fn apply_static_ref(&'static self) -> Result<(), SetLoggerError>
  where Self: Sized + 'static, Self::Error: fmt::Display;
  fn apply_boxed(self: Box<Self>) -> Result<(), SetLoggerError>
  where Self: Sized + 'static, Self::Error: fmt::Display;

  fn with_predicate<P>(self, predicate: P) -> OutputPredicate<Self, P>
  where Self: Sized, P: Fn(&Metadata<'_>) -> bool + Send + Sync;
}

impl<O: Output> OutputExt for O {
  fn as_dyn_log(&self) -> &dyn Log
  where Self: Sized, Self::Error: fmt::Display {
    LogOutput::wrap_ref(self)
  }

  fn into_dyn_log(self: Box<Self>) -> Box<dyn Log>
  where Self: Sized + 'static, Self::Error: fmt::Display {
    LogOutput::wrap_box(self)
  }

  fn apply(self) -> Result<(), SetLoggerError>
  where Self: Sized + 'static, Self::Error: fmt::Display {
    Box::new(self).apply_boxed()
  }

  fn apply_static_ref(&'static self) -> Result<(), SetLoggerError>
  where Self: Sized + 'static, Self::Error: fmt::Display {
    log::set_logger(self.as_dyn_log())
  }

  fn apply_boxed(self: Box<Self>) -> Result<(), SetLoggerError>
  where Self: Sized + 'static, Self::Error: fmt::Display {
    log::set_boxed_logger(self.into_dyn_log())
  }

  fn with_predicate<P>(self, predicate: P) -> OutputPredicate<Self, P>
  where Self: Sized, P: Fn(&Metadata<'_>) -> bool + Send + Sync {
    OutputPredicate::new(self, predicate)
  }
}

pub trait OutputMutExt: OutputMut {
  fn into_output(self) -> Mutex<Self> where Self: Sized;

  fn with_predicate<P>(self, predicate: P) -> OutputPredicate<Self, P>
  where Self: Sized, P: FnMut(&Metadata<'_>) -> bool + Send + Sync;
}

impl<O: OutputMut> OutputMutExt for O {
  fn into_output(self) -> Mutex<Self> where Self: Sized {
    Mutex::new(self)
  }

  fn with_predicate<P>(self, predicate: P) -> OutputPredicate<Self, P>
  where Self: Sized, P: FnMut(&Metadata<'_>) -> bool + Send + Sync {
    OutputPredicate::new_mut(self, predicate)
  }
}



pub trait Output: Send + Sync {
  type Error;

  fn enabled(&self, metadata: &Metadata) -> bool;
  fn log(&self, record: &Record) -> Result<(), Self::Error>;
  fn flush(&self) -> Result<(), Self::Error>;
}

impl<O: OutputMut + ?Sized> Output for Mutex<O> {
  type Error = O::Error;

  fn enabled(&self, metadata: &Metadata) -> bool {
    self.lock().unwrap_or_else(|l| l.into_inner()).enabled(metadata)
  }

  fn log(&self, record: &Record) -> Result<(), Self::Error> {
    self.lock().unwrap_or_else(|l| l.into_inner()).log(record)
  }

  fn flush(&self) -> Result<(), Self::Error> {
    self.lock().unwrap_or_else(|l| l.into_inner()).flush()
  }
}

pub trait OutputMut: Send + Sync {
  type Error;

  fn enabled(&mut self, metadata: &Metadata) -> bool;
  fn log(&mut self, record: &Record) -> Result<(), Self::Error>;
  fn flush(&mut self) -> Result<(), Self::Error>;
}

macro_rules! impl_output_deref {
  (<$O:ident> $Type:ty) => (
    impl<$O: Output + ?Sized> Output for $Type {
      type Error = $O::Error;

      fn enabled(&self, metadata: &Metadata) -> bool {
        $O::enabled(self, metadata)
      }

      fn log(&self, record: &Record) -> Result<(), Self::Error> {
        $O::log(self, record)
      }

      fn flush(&self) -> Result<(), Self::Error> {
        $O::flush(self)
      }
    }
  );
}

impl_output_deref!(<O> &O);
impl_output_deref!(<O> &mut O);
impl_output_deref!(<O> Box<O>);
impl_output_deref!(<O> std::sync::Arc<O>);

macro_rules! impl_output_iter {
  (<$O:ident> $Type:ty) => (
    impl<$O: Output> Output for $Type {
      type Error = $O::Error;

      fn enabled(&self, metadata: &Metadata) -> bool {
        self.into_iter().any(|output| $O::enabled(output, metadata))
      }

      fn log(&self, record: &Record) -> Result<(), Self::Error> {
        self.into_iter().try_for_each(|output| $O::log(output, record))
      }

      fn flush(&self) -> Result<(), Self::Error> {
        self.into_iter().try_for_each(|output| $O::flush(output))
      }
    }
  );
}

impl_output_iter!(<O> [O]);
impl_output_iter!(<O> Vec<O>);

macro_rules! impl_output_deref_mut {
  (<$O:ident> $Type:ty) => (
    impl<$O: OutputMut + ?Sized> OutputMut for $Type {
      type Error = $O::Error;

      fn enabled(&mut self, metadata: &Metadata) -> bool {
        $O::enabled(self, metadata)
      }

      fn log(&mut self, record: &Record) -> Result<(), Self::Error> {
        $O::log(self, record)
      }

      fn flush(&mut self) -> Result<(), Self::Error> {
        $O::flush(self)
      }
    }
  );
}

impl_output_deref_mut!(<O> &mut O);
impl_output_deref_mut!(<O> Box<O>);

macro_rules! impl_output_iter_mut {
  (<$O:ident> $Type:ty) => (
    impl<$O: OutputMut> OutputMut for $Type {
      type Error = $O::Error;

      fn enabled(&mut self, metadata: &Metadata) -> bool {
        self.into_iter().any(|output| $O::enabled(output, metadata))
      }

      fn log(&mut self, record: &Record) -> Result<(), Self::Error> {
        self.into_iter().try_for_each(|output| $O::log(output, record))
      }

      fn flush(&mut self) -> Result<(), Self::Error> {
        self.into_iter().try_for_each(|output| $O::flush(output))
      }
    }
  );
}

impl_output_iter_mut!(<O> [O]);
impl_output_iter_mut!(<O> Vec<O>);



#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputWrite<W: ?Sized>(pub W);

unsafe impl<W: ?Sized> TransparentWrapper<W> for OutputWrite<W> {}

impl<W: Send + Sync + ?Sized> OutputMut for OutputWrite<W> where W: Write {
  type Error = io::Error;

  fn enabled(&mut self, _: &Metadata) -> bool {
    true
  }

  fn log(&mut self, record: &Record) -> Result<(), Self::Error> {
    write_record(&mut self.0, record)?;
    self.0.flush()?;
    Ok(())
  }

  fn flush(&mut self) -> Result<(), Self::Error> {
    self.0.flush()?;
    Ok(())
  }
}

impl<W: Send + Sync + ?Sized> Output for OutputWrite<W> where for<'w> &'w W: Write {
  type Error = io::Error;

  fn enabled(&self, _: &Metadata) -> bool {
    true
  }

  fn log(&self, record: &Record) -> Result<(), Self::Error> {
    write_record(&mut &self.0, record)?;
    (&mut &self.0).flush()?;
    Ok(())
  }

  fn flush(&self) -> Result<(), Self::Error> {
    (&mut &self.0).flush()?;
    Ok(())
  }
}



macro_rules! impl_std_log_output {
  ($Stream:ty) => (
    impl OutputMut for $Stream {
      type Error = io::Error;

      fn enabled(&mut self, _: &Metadata) -> bool {
        true
      }

      fn log(&mut self, record: &Record) -> Result<(), Self::Error> {
        write_record(self, record)
      }

      fn flush(&mut self) -> Result<(), Self::Error> {
        Write::flush(self)
      }
    }

    impl Output for $Stream {
      type Error = io::Error;

      fn enabled(&self, _: &Metadata) -> bool {
        true
      }

      fn log(mut self: &Self, record: &Record) -> Result<(), Self::Error> {
        write_record(&mut self, record)
      }

      fn flush(mut self: &Self) -> Result<(), Self::Error> {
        Write::flush(&mut self)
      }
    }
  );
}

impl_std_log_output!(io::Stdout);
impl_std_log_output!(io::Stderr);



#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Panic;

impl Output for Panic {
  type Error = Infallible;

  fn enabled(&self, _: &Metadata) -> bool {
    true
  }

  fn log(&self, record: &Record) -> Result<(), Self::Error> {
    panic!("{}", record.args());
  }

  fn flush(&self) -> Result<(), Self::Error> {
    Ok(())
  }
}

impl OutputMut for Panic {
  type Error = Infallible;

  fn enabled(&mut self, _: &Metadata) -> bool {
    true
  }

  fn log(&mut self, record: &Record) -> Result<(), Self::Error> {
    panic!("{}", record.args());
  }

  fn flush(&mut self) -> Result<(), Self::Error> {
    Ok(())
  }
}



impl Output for Sender<String> {
  type Error = SendError<String>;

  fn enabled(&self, _: &Metadata) -> bool {
    true
  }

  fn log(&self, record: &Record) -> Result<(), Self::Error> {
    self.send(format!("{}\n", record.args()))
  }

  fn flush(&self) -> Result<(), Self::Error> {
    Ok(())
  }
}



fn write_record<W: Write + ?Sized>(write: &mut W, record: &Record) -> io::Result<()> {
  writeln!(write, "{}", record.args())
}



#[derive(Debug, Clone, Copy)]
pub struct OutputPredicate<O, P> {
  pub output: O,
  pub predicate: P
}

impl<O, P> OutputPredicate<O, P> {
  pub const fn new(output: O, predicate: P) -> Self
  where O: Output, P: Fn(&Metadata<'_>) -> bool + Send + Sync {
    OutputPredicate { output, predicate }
  }

  pub const fn new_mut(output: O, predicate: P) -> Self
  where O: OutputMut, P: FnMut(&Metadata<'_>) -> bool + Send + Sync {
    OutputPredicate { output, predicate }
  }
}

impl<O: Output, P: Fn(&Metadata<'_>) -> bool + Send + Sync> Output for OutputPredicate<O, P> {
  type Error = O::Error;

  fn enabled(&self, metadata: &Metadata) -> bool {
    self.output.enabled(metadata) && (self.predicate)(metadata)
  }

  fn log(&self, record: &Record) -> Result<(), Self::Error> {
    self.output.log(record)
  }

  fn flush(&self) -> Result<(), Self::Error> {
    self.output.flush()
  }
}

impl<O: OutputMut, P: FnMut(&Metadata<'_>) -> bool + Send + Sync> OutputMut for OutputPredicate<O, P> {
  type Error = O::Error;

  fn enabled(&mut self, metadata: &Metadata) -> bool {
    self.output.enabled(metadata) && (self.predicate)(metadata)
  }

  fn log(&mut self, record: &Record) -> Result<(), Self::Error> {
    self.output.log(record)
  }

  fn flush(&mut self) -> Result<(), Self::Error> {
    self.output.flush()
  }
}



#[derive(Debug, Clone, Copy)]
pub struct OutputMapError<O, F> {
  pub output: O,
  pub map_error: F
}

impl<O, F> OutputMapError<O, F> {
  pub const fn new<E>(output: O, map_error: F) -> Self
  where O: Output, F: Fn(O::Error) -> E + Send + Sync {
    OutputMapError { output, map_error }
  }

  pub const fn new_mut<E>(output: O, map_error: F) -> Self
  where O: OutputMut, F: FnMut(O::Error) -> E + Send + Sync {
    OutputMapError { output, map_error }
  }
}

impl<O: Output, F: Fn(O::Error) -> E + Send + Sync, E> Output for OutputMapError<O, F> {
  type Error = E;

  fn enabled(&self, metadata: &Metadata) -> bool {
    self.output.enabled(metadata)
  }

  fn log(&self, record: &Record) -> Result<(), Self::Error> {
    self.output.log(record).map_err(&self.map_error)
  }

  fn flush(&self) -> Result<(), Self::Error> {
    self.output.flush().map_err(&self.map_error)
  }
}

impl<O: OutputMut, F: FnMut(O::Error) -> E + Send + Sync, E> OutputMut for OutputMapError<O, F> {
  type Error = E;

  fn enabled(&mut self, metadata: &Metadata) -> bool {
    self.output.enabled(metadata)
  }

  fn log(&mut self, record: &Record) -> Result<(), Self::Error> {
    self.output.log(record).map_err(&mut self.map_error)
  }

  fn flush(&mut self) -> Result<(), Self::Error> {
    self.output.flush().map_err(&mut self.map_error)
  }
}



#[repr(transparent)]
#[derive(Debug)]
pub struct LogOutput<O: ?Sized>(pub O);

unsafe impl<O: ?Sized> TransparentWrapper<O> for LogOutput<O> {}

impl<O: Output<Error = E> + ?Sized, E: fmt::Display> Log for LogOutput<O> {
  fn enabled(&self, metadata: &Metadata) -> bool {
    self.0.enabled(metadata)
  }

  fn log(&self, record: &Record) {
    fallback_log(record, || self.0.log(record));
  }

  fn flush(&self) {
    fallback_flush(|| self.0.flush());
  }
}



#[repr(transparent)]
#[derive(Debug)]
pub struct LogOutputInfallible<O: ?Sized>(pub O);

unsafe impl<O: ?Sized> TransparentWrapper<O> for LogOutputInfallible<O> {}

impl<O: Output<Error = Infallible> + ?Sized> Log for LogOutputInfallible<O> {
  fn enabled(&self, metadata: &Metadata) -> bool {
    Output::enabled(&self.0, metadata)
  }

  fn log(&self, record: &Record) {
    Output::log(&self.0, record)
      .unwrap_or_else(|i| match i {})
  }

  fn flush(&self) {
    Output::flush(&self.0)
      .unwrap_or_else(|i| match i {})
  }
}



impl<E: fmt::Display> Log for dyn Output<Error = E> {
  fn enabled(&self, metadata: &Metadata) -> bool {
    LogOutput::wrap_ref(self).enabled(metadata)
  }

  fn log(&self, record: &Record) {
    LogOutput::wrap_ref(self).log(record);
  }

  fn flush(&self) {
    LogOutput::wrap_ref(self).flush();
  }
}



fn fallback_log<E: fmt::Display>(record: &Record, f: impl FnOnce() -> Result<(), E>) {
  f().unwrap_or_else(|error| panic!("error while logging:\n- contents: {}\n- record: {:?}\n- error: {}", record.args(), record, error))
}

fn fallback_flush<E: fmt::Display>(f: impl FnOnce() -> Result<(), E>) {
  f().unwrap_or_else(|error| panic!("error while flushing logger:\n- error: {}", error));
}
