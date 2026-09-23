# Large property sets, defaults and overrides

[Research index](README.md) · [Earlier ECS examples](attributes-and-energy.md) · [Attributes context](../tutorial/context/attributes-and-defaults.md)

**Checked:** September 21, 2026. Nick broadened the question beyond games to
systems with many attributes and defaults. Two bounded helpers researched
configuration and scene-description systems; the lead researched UI properties
and CSS. These are documented designs, not benchmarks run for Moss.

The broader lesson is to separate **what can be configured**, **what someone
actually supplied**, and **what a consumer reads**. A large property model need
not have the same representation at all three stages.

## On this page

- [Practical precedents](#practical-precedents)
- [The recurring design](#the-recurring-design)
- [Choosing storage](#choosing-storage)
- [What this changes for Moss](#what-this-changes-for-moss)

## Practical precedents

### WPF: many possible properties, sparse instance storage

Microsoft's WPF architecture describes sparse property storage specifically for
objects with dozens or hundreds of properties. Each instance need not carry
every value when defaults, inheritance or styles provide it. The framework also
tracks dependencies and revalidates values when those dependencies change.
This is a documented architectural rationale, not a memory benchmark.
[WPF architecture](https://learn.microsoft.com/en-us/dotnet/desktop/wpf/advanced/wpf-architecture).

Its precedence rules distinguish metadata defaults, inherited values, styles,
local values, animation and coercion. Clearing a local value exposes the next
applicable source; it does not necessarily restore the metadata default.
Consequently, “set this to the default's current number” and “remove my override”
are different operations.
[Property precedence](https://learn.microsoft.com/en-us/dotnet/desktop/wpf/properties/dependency-property-value-precedence).

### CSS: authored declarations and computed values are different stages

The CSS Cascade Level 5 specification separates declarations, cascade winners,
defaulting, computed values and later layout-dependent values. Inheritance is a
property-specific rule, not something every field does automatically. Its
`initial`, `inherit`, `unset` and `revert` keywords have distinct meanings.
This is a useful precedent for explicit resolution stages; the standard does
not prescribe a particular browser's memory layout or caching implementation.
[CSS value processing](https://www.w3.org/TR/css-cascade-5/#value-stages).

### OpenUSD: schemas, authored layers and resolved attributes

OpenUSD's scene-description model supports schema fallback values without
requiring every object to author them. Layers contribute edits, and composition
determines which contributions take precedence. Strength follows composition
rules rather than the timestamp of the last write. Some data composes by key
or through list-edit operations, so one generic replacement rule is insufficient.
The reviewed release documentation identifies itself as **26.08**.
[USD terms and concepts](https://openusd.org/release/glossary.html).

`UsdResolveInfo` exposes information about where a resolved value comes from.
`UsdAttributeQuery` speeds repeated reads by caching source information. That
query does not listen for change notifications: consumers must discard it when
the associated attribute is resynchronized. This is a concrete example of
selective caching with an explicit invalidation obligation.
[Resolve information](https://openusd.org/release/api/class_usd_resolve_info.html),
[attribute queries](https://openusd.org/release/api/class_usd_attribute_query.html).

### Figment: compose flexible data, extract typed Rust values

Figment **0.10.19** combines configuration providers and extracts results into
types implementing `Deserialize`. It tracks source metadata through composition
and errors. Its profiles have their own precedence: default, selected, then
global. Sources are read when added; it does not provide automatic live reloading.
This gives a practical Rust example of flexible input paired with concrete
consumer structs. Retain the Figment if later inspection of source metadata is
needed; ordinary struct fields do not become provenance records automatically.
[Figment documentation](https://docs.rs/figment/0.10.19/figment/).

Its operations also distinguish replacement from concatenation: `merge`
recursively combines dictionaries but replaces arrays; `admerge` concatenates
arrays. These are explicit policies, not an intrinsic meaning of “merge.”
[Composition API](https://docs.rs/figment/0.10.19/figment/struct.Figment.html).

### Typesafe Config: missing and null have different meanings

Typesafe Config's immutable configurations compose with `withFallback`.
The primary configuration wins, objects merge recursively, and explicit null
can suppress a fallback even though ordinary getters reject null. Dedicated
APIs distinguish absent and null. Substitutions should be resolved against the
complete configuration after composition. The official repository lists
**1.4.4**; the linked API uses a moving `latest` URL.
[Config API](https://lightbend.github.io/config/latest/api/com/typesafe/config/Config.html),
[project](https://github.com/lightbend/config).

## The recurring design

The following is our synthesis of these systems. They do not all implement the
same pipeline or promise the same update behavior.

| Responsibility | What it answers |
| --- | --- |
| Definition or schema | Which properties exist, their types, units, defaults and validation rules. |
| Authored values | Which fields this source or object explicitly supplies. |
| Resolution | Which sources win, which values combine, and how missing values behave. |
| Consumer representation | The values a particular reader needs, possibly as a typed snapshot or selective query. |
| Provenance | Why a value has its result and which source or sources contributed. |

For a generic configuration example, suppose a default `retry_limit` is 3,
a profile supplies 5, and one object explicitly supplies 0. If the policy is
default < profile < object, the result is 0. Removing the object override reveals
5. Setting the object to 3 pins it to 3; it does not reconnect it to the default.
This example selects its own simple policy and does not reproduce every cited
framework's precedence rules.

The distinction remains important when values happen to be equal. An omitted
property and an explicit value matching today's default can react differently
to tomorrow's default change. Comparing numbers cannot recover that intent.

Collections need their own contract. An override might replace a list, append
to it, merge entries by identity, or deliberately clear it. Arithmetic modifiers
also need an equation and order; they are not ordinary scalar replacement.
Use only the operations the domain needs, but name them precisely.

Validate after composition as well as during parsing. A value can have the right
type yet violate a range or a relationship with another field. An invalid
explicit value should not silently masquerade as an absent value and fall back.

## Choosing storage

“Many attributes” alone does not choose a representation. The relevant questions
include how many properties are actually present, how often they are read or
changed, whether new property kinds must be introduced at runtime, and whether
default changes should propagate.

| Approach | Useful fit and cost to consider |
| --- | --- |
| Ordinary typed structs or components | Known fields and frequent reads; direct access and compiler help, with storage for the fields present in each value. |
| Schema-backed sparse property storage | Many optional or extensible fields; avoids storing every value but adds lookup, validation and resolution work. |
| Layered input with typed consumer views | Flexible authoring plus focused reads; requires an explicit rule for when views are rebuilt and what provenance is retained. |

These can coexist. Large shared definitions need not be expanded into every
consumer view, and seldom-read properties need not be eagerly resolved. Conversely,
a dynamic authoring format need not force string-key lookup on every hot read.
These are design options, not measured winners.

A typed snapshot is independent once created. A live resolved view must react
to changes. A cache intended to reflect changing inputs needs invalidation or
versioning; a cache tied to an immutable snapshot can remain valid indefinitely.
Make that contract explicit before choosing a storage optimization.

## What this changes for Moss

The broader research strengthens the separation between authored configuration
and runtime data. It also makes sparse overrides and explainable resolution
first-class alternatives to copying an entire attribute catalog per entity.
The earlier component recommendation applies to small, frequently used individual
values; it does not settle how every future attribute should be represented.

Keep authored scenario overrides separately from initialized runtime values so
reset and future inspection can distinguish intent from the resulting number.
We can express today's defaults and overrides with concrete Rust types. A
general registry, new configuration dependency or live dependency graph would
need an actual use case before implementation.

The active paired edit remains unchanged. No code, dependency or tutorial code
example changed during this research. No native tests, browser runs or performance
measurements were performed; these sources establish contracts and precedents.

[Research index](README.md) · [Attributes context](../tutorial/context/attributes-and-defaults.md)
