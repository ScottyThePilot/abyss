//! An [`Output`] logger with a rich set of options and filters,
//! and can delegate to a number of other child [`Output`]s.
//!
//! Some code was adapted from or took inspiration from `fern` `0.7.1`, under the MIT license.

use super::output::Output;

use log::{Level, LevelFilter, Metadata, Record, RecordBuilder};

use std::collections::HashMap;
use std::ops::{Index, IndexMut};
use std::borrow::Cow;
use std::fmt;



pub struct Rich<E> {
  formatter: Option<Box<Formatter<E>>>,
  level_default: LevelFilter,
  levels: HashMap<Cow<'static, str>, LevelFilter>,
  filters: Vec<Box<Filter>>,
  children: Vec<Box<dyn Output<Error = E>>>
}

impl<E> Rich<E> {
  pub fn new() -> Self {
    Rich {
      formatter: None,
      level_default: LevelFilter::Trace,
      levels: HashMap::new(),
      filters: Vec::new(),
      children: Vec::new()
    }
  }

  pub fn with_format<F>(mut self, formatter: F) -> Self
  where F: Fn(FormatterCallback<'_, E>, &fmt::Arguments<'_>, &Record<'_>) -> Result<(), E> + Send + Sync + 'static {
    self.formatter = Some(Box::new(formatter));
    self
  }

  pub fn with_level(mut self, level: LevelFilter) -> Self {
    self.level_default = level;
    self
  }

  pub fn with_level_for<M>(mut self, module: M, level: LevelFilter) -> Self
  where M: Into<Cow<'static, str>> {
    self.levels.insert(module.into(), level);
    self
  }

  pub fn with_filter<F>(mut self, filter: F) -> Self
  where F: Fn(&Metadata<'_>) -> bool + Send + Sync + 'static {
    self.filters.push(Box::new(filter));
    self
  }

  pub fn with_children_boxed<I>(mut self, children: I) -> Self
  where I: IntoIterator<Item = Box<dyn Output<Error = E>>> {
    self.children.extend(children);
    self
  }

  pub fn with_child_boxed(mut self, child: Box<dyn Output<Error = E>>) -> Self {
    self.children.push(child);
    self
  }

  pub fn with_child<O>(self, child: O) -> Self
  where O: Output<Error = E> + 'static {
    self.with_child_boxed(Box::new(child))
  }

  fn find_module(&self, metadata: &Metadata) -> LevelFilter {
    find_module(&self.levels, metadata.target()).unwrap_or(self.level_default)
  }

  fn enabled_shallow(&self, metadata: &Metadata) -> bool {
    metadata.level() <= self.find_module(metadata) && self.filters.iter().all(|filter| filter(metadata))
  }

  fn enabled_deep(&self, metadata: &Metadata) -> bool {
    self.enabled_shallow(metadata) && self.children.iter().any(|child| child.enabled(metadata))
  }
}

impl<E> fmt::Debug for Rich<E> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Rich")
      .field("formatter", &self.formatter.as_ref().map(|_| format_args!("..")))
      .field("level_default", &self.level_default)
      .field("levels", &self.levels)
      .field("filters", &[format_args!("..")])
      .field("children", &[format_args!("..")])
      .finish()
  }
}

impl<E> Output for Rich<E> {
  type Error = E;

  fn enabled(&self, metadata: &Metadata) -> bool {
    self.enabled_deep(metadata)
  }

  fn log(&self, record: &Record) -> Result<(), Self::Error> {
    if self.enabled_shallow(record.metadata()) {
      let mut finished = false;
      if let Some(ref format) = self.formatter {
        (format)(
          FormatterCallback {
            finished: &mut finished,
            rich: self,
            record
          },
          record.args(),
          record
        )?;
      };

      if !finished {
        self.children.log(record)?;
      };
    };

    Ok(())
  }

  fn flush(&self) -> Result<(), Self::Error> {
    self.children.flush()
  }
}

pub type Filter = dyn Fn(&Metadata<'_>) -> bool + Send + Sync + 'static;

pub type Formatter<E> = dyn Fn(FormatterCallback<'_, E>, &fmt::Arguments<'_>, &Record<'_>) -> Result<(), E> + Send + Sync + 'static;

#[derive(Debug)]
pub struct FormatterCallback<'a, E> {
  finished: &'a mut bool,
  rich: &'a Rich<E>,
  record: &'a Record<'a>
}

impl<'a, E> FormatterCallback<'a, E> {
  pub fn finish(self, formatted_message: fmt::Arguments<'_>) -> Result<(), E> {
    let record = RecordBuilder::new()
      .args(formatted_message)
      .metadata(self.record.metadata().clone())
      .level(self.record.level())
      .target(self.record.target())
      .module_path(self.record.module_path())
      .file(self.record.file())
      .line(self.record.line())
      .build();

    self.rich.children.log(&record)?;

    *self.finished = true;

    Ok(())
  }
}



fn find_module(modules: &HashMap<Cow<'static, str>, LevelFilter>, module: &str) -> Option<LevelFilter> {
  std::iter::successors(Some(module), |&module| Some(module.rsplit_once("::")?.0))
    .find_map(|module| modules.get(module).copied())
}



#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct LevelsConfig<T> {
  pub error: T,
  pub warn: T,
  pub info: T,
  pub debug: T,
  pub trace: T
}

impl<T> LevelsConfig<T> {
  pub const fn splat(value: T) -> Self where T: Copy {
    LevelsConfig {
      error: value,
      warn: value,
      info: value,
      debug: value,
      trace: value
    }
  }

  pub const fn get(&self, variant: Level) -> &T {
    match variant {
      Level::Error => &self.error,
      Level::Warn => &self.warn,
      Level::Info => &self.info,
      Level::Debug => &self.debug,
      Level::Trace => &self.trace
    }
  }

  pub const fn get_mut(&mut self, variant: Level) -> &mut T {
    match variant {
      Level::Error => &mut self.error,
      Level::Warn => &mut self.warn,
      Level::Info => &mut self.info,
      Level::Debug => &mut self.debug,
      Level::Trace => &mut self.trace
    }
  }

  pub const fn set(&mut self, variant: Level, value: T) -> T {
    std::mem::replace(self.get_mut(variant), value)
  }
}

impl<T> Index<Level> for LevelsConfig<T> {
  type Output = T;

  fn index(&self, variant: Level) -> &T {
    self.get(variant)
  }
}

impl<T> IndexMut<Level> for LevelsConfig<T> {
  fn index_mut(&mut self, variant: Level) -> &mut T {
    self.get_mut(variant)
  }
}
