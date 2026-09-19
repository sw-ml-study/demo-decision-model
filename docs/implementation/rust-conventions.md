# Rust conventions for the microscope crates

The plan says `sw-checklist` applies to Rust code in this repository. It is run,
it is read, and where it disagrees with what we shipped the disagreement is
written down here rather than silently ignored.

## What the crates do honour

- Edition 2024, `unsafe_code = "deny"`, and `clippy::all` plus
  `clippy::pedantic` denied at the workspace level. `cargo clippy --all-targets`
  is clean with no `allow` escapes; the one narrowing is a scoped `#[expect]`
  with a stated reason.
- `cargo fmt --check` is clean and is part of `just check`.
- No source file over 500 lines, and no function over 25 lines.
- The crate boundary rule from [`../plan.md`](../plan.md): the crates know about
  *decisions*. Label names, policy text and output strings arrive as data. A
  script (`scripts/check-abstraction`) enforces that no demo vocabulary appears
  under `crates/`.

## Accepted deviation: module function count

`sw-checklist` fails a module with more than 7 functions, and warns a crate with
more than 4 modules (failing above 7). `tdm-trace` is at 7 modules and still
reports 14 to 18 functions in the four modules that carry types.

Those two rules cannot both be satisfied here, and the reason is structural
rather than stylistic. `tdm-trace` is a data-interchange crate: its public
surface is read-only accessors over parsed types. `Turn`, `State`, `Memory`,
`Policy`, and `Output` contribute eighteen accessors between them, and splitting
each type into its own module would put the crate well past the module cap
without making anything easier to read.

Options considered:

1. **Public fields instead of accessors.** Rejected. `Trace::parse` is the only
   way to obtain a validated trace; public fields would let a consumer construct
   an invalid `Decision` by hand and defeat the point of validating at all.
2. **One module per type.** Rejected. It trades a failing rule for a failing
   rule and costs readability: `Policy` would be a six-line file.
3. **Ship it and record the deviation.** Chosen.

The checks whose intent is "keep units small enough to hold in your head" —
function length, file length, crate module count — are met. The function-count
rule is measuring accessor ceremony here, not complexity.

For reference, the sibling `../demo-extensions` runs a large number of
`sw-checklist` failures against the same standard, so treating it as advisory
rather than as a blocking gate is the established practice in these
repositories, not a local exemption invented here.

## What would change this

If `tdm-trace` grows behaviour beyond parse-validate-read — a builder, a differ,
a renderer — that behaviour gets its own crate rather than being added to these
modules, and this note gets revisited.
