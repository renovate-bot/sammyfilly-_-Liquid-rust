use std::fmt;
use std::io::Write;

use itertools;

use super::Filter;
use liquid_error::{Result, ResultLiquidExt, ResultLiquidReplaceExt};
use liquid_interpreter::Context;
use liquid_interpreter::Expression;
use liquid_interpreter::Renderable;
use liquid_value::{ValueCow, ValueView};

/// A `Value` expression.
#[derive(Debug)]
pub struct FilterChain {
    entry: Expression,
    filters: Vec<Box<dyn Filter>>,
}

impl FilterChain {
    /// Create a new expression.
    pub fn new(entry: Expression, filters: Vec<Box<dyn Filter>>) -> Self {
        Self { entry, filters }
    }

    /// Process `Value` expression within `context`'s stack.
    pub fn evaluate<'s>(&'s self, context: &'s Context) -> Result<ValueCow<'s>> {
        // take either the provided value or the value from the provided variable
        let mut entry = ValueCow::Borrowed(self.entry.evaluate(context)?);

        // apply all specified filters
        for filter in &self.filters {
            entry = ValueCow::Owned(
                filter
                    .evaluate(entry.as_view(), context)
                    .trace("Filter error")
                    .context_key("filter")
                    .value_with(|| format!("{}", filter).into())
                    .context_key("input")
                    .value_with(|| format!("{}", entry.source()).into())?,
            );
        }

        Ok(entry)
    }
}

impl fmt::Display for FilterChain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} | {}",
            self.entry,
            itertools::join(&self.filters, " | ")
        )
    }
}

impl Renderable for FilterChain {
    fn render_to(&self, writer: &mut dyn Write, context: &mut Context) -> Result<()> {
        let entry = self.evaluate(context)?;
        write!(writer, "{}", entry.render()).replace("Failed to render")?;
        Ok(())
    }
}