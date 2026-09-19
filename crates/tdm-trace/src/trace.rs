//! The trace document: its provenance, its bounded output space, and the entry
//! point that parses and validates one.

use std::collections::HashSet;

use serde::Deserialize;

use crate::error::ValidationError;
use crate::limits::{MAX_TABLE_ENTRIES, MAX_TABLES, MAX_TURNS, SCHEMA, VERSION};
use crate::turn::Turn;
use crate::validate::{
    bounded_text, validate_decision, validate_output, validate_provenance, validate_state,
};

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trace {
    pub(crate) schema: String,
    pub(crate) version: u32,
    pub(crate) provenance: Provenance,
    /// Which application produced the trace. Free text: a label for the reader
    /// and the instrument's title, never something this crate branches on.
    pub(crate) demo: String,
    /// The bounded output space. Every turn's text is quoted from here.
    pub(crate) tables: Vec<Table>,
    pub(crate) turns: Vec<Turn>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    pub(crate) producer: String,
    pub(crate) producer_revision: String,
    pub(crate) generated_at: String,
    pub(crate) source_description: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Table {
    pub(crate) name: String,
    pub(crate) entries: Vec<String>,
}

impl Trace {
    /// Parses and validates one complete trace.
    ///
    /// # Errors
    ///
    /// Returns the first error in the documented validation order.
    pub fn parse(source: &str) -> Result<Self, ValidationError> {
        let trace: Self = serde_json::from_str(source).map_err(|_| ValidationError::Malformed)?;
        trace.validate()?;
        Ok(trace)
    }

    #[must_use]
    pub const fn provenance(&self) -> &Provenance {
        &self.provenance
    }

    #[must_use]
    pub fn demo(&self) -> &str {
        &self.demo
    }

    #[must_use]
    pub fn tables(&self) -> &[Table] {
        &self.tables
    }

    #[must_use]
    pub fn turns(&self) -> &[Turn] {
        &self.turns
    }

    /// The table a turn cites, or `None` when the trace does not carry it.
    #[must_use]
    pub fn table(&self, name: &str) -> Option<&Table> {
        self.tables.iter().find(|t| t.name == name)
    }

    fn validate(&self) -> Result<(), ValidationError> {
        if self.schema != SCHEMA {
            return Err(ValidationError::UnsupportedSchema);
        }
        if self.version != VERSION {
            return Err(ValidationError::UnsupportedVersion(self.version));
        }
        validate_provenance(&self.provenance)?;
        bounded_text(&self.demo, "demo")?;
        self.validate_tables()?;
        self.validate_turns()
    }

    fn validate_tables(&self) -> Result<(), ValidationError> {
        if self.tables.len() > MAX_TABLES {
            return Err(ValidationError::Budget("tables"));
        }
        let mut seen = HashSet::new();
        for table in &self.tables {
            bounded_text(&table.name, "table name")?;
            if !seen.insert(table.name.as_str()) {
                return Err(ValidationError::DuplicateTable(table.name.clone()));
            }
            if table.entries.len() > MAX_TABLE_ENTRIES {
                return Err(ValidationError::Budget("table entries"));
            }
            for entry in &table.entries {
                bounded_text(entry, "table entry")?;
            }
        }
        Ok(())
    }

    fn validate_turns(&self) -> Result<(), ValidationError> {
        if self.turns.len() > MAX_TURNS {
            return Err(ValidationError::Budget("turns"));
        }
        let mut previous: Option<u32> = None;
        for turn in &self.turns {
            if let Some(prev) = previous
                && turn.index <= prev
            {
                return Err(ValidationError::TurnOrder(turn.index));
            }
            previous = Some(turn.index);
            validate_state(&turn.state)?;
            if turn.decisions.len() > crate::limits::MAX_DECISIONS_PER_TURN {
                return Err(ValidationError::Budget("decisions per turn"));
            }
            for decision in &turn.decisions {
                validate_decision(decision, turn.index)?;
            }
            bounded_text(&turn.policy.rule, "policy rule")?;
            bounded_text(&turn.policy.branch, "policy branch")?;
            let table = turn.output.table().and_then(|name| self.table(name));
            validate_output(&turn.output, &turn.decisions, table, turn.index)?;
        }
        Ok(())
    }
}

impl Provenance {
    #[must_use]
    pub fn producer(&self) -> &str {
        &self.producer
    }
    #[must_use]
    pub fn producer_revision(&self) -> &str {
        &self.producer_revision
    }
    #[must_use]
    pub fn generated_at(&self) -> &str {
        &self.generated_at
    }
    #[must_use]
    pub fn source_description(&self) -> &str {
        &self.source_description
    }
}

impl Table {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub fn entries(&self) -> &[String] {
        &self.entries
    }
}
