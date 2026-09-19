//! The validation failure modes, in the order the validator checks them.

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValidationError {
    Malformed,
    UnsupportedSchema,
    UnsupportedVersion(u32),
    Budget(&'static str),
    EmptyText(&'static str),
    TextTooLong(&'static str),
    DuplicateTable(String),
    DuplicateMemoryId(String),
    /// A turn index did not increase; traces are ordered.
    TurnOrder(u32),
    /// A decision claimed a kind outside [`crate::KINDS`]. `"generate"` lands here.
    UnknownKind(String),
    /// Labels and probabilities disagreed, or there were none.
    LabelCount(u32),
    /// A probability was negative, or the distribution did not sum to one.
    NotADistribution(u32),
    /// `selected` was out of range for the label set.
    SelectedOutOfRange(u32),
    /// A derived scalar disagreed with the distribution it is derived from.
    DerivedMismatch(&'static str),
    /// A kind-specific field was present on the wrong kind, or missing on its own.
    FieldForWrongKind(&'static str),
    /// The turn named a table the trace does not carry.
    UnknownTable(String),
    /// The turn's index was outside that table.
    IndexOutOfTable(u32),
    /// The turn supplied a different number of slots than the entry has
    /// placeholders.
    SlotCount(u32),
    /// A chosen output cited a decision index this turn does not have.
    UnknownDecision(u32),
    /// The output text was not one of the candidates the program offered: it was
    /// neither the selected label of the decision it cited, nor the table entry
    /// it cited with its recorded slots filled. This is the invariant that makes
    /// the bounded-output claim checkable from the artifact alone.
    TextNotOffered(u32),
}
