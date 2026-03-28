# ferrox-webapp-macros

Derive macros for Rust/Leptos webapp projects. Eliminates boilerplate for
reactive form models and common trait implementations.

Designed to be used alongside `webapp-lib` (or any crate that defines the
`HasId` / `HasName` traits), but the macros themselves have no dependency on
Leptos or any domain crate.

---

## Macros

| Macro | What it generates |
|---|---|
| `#[derive(HasId)]` | `impl HasId for T` using a designated `String` field |
| `#[derive(HasName)]` | `impl HasName for T` using a designated `String` field |
| `#[derive(FormModel)]` | A companion `{T}FormModel` struct with Leptos `RwSignal<_>` fields + `new()` / `from_t()` / `to_t()` |

---

## Important: trait definitions live elsewhere

A `proc-macro = true` crate cannot export regular items (traits, structs, etc.),
only macros. This means:

- `HasId` and `HasName` **traits must be defined in your library/domain crate**
  (e.g. `webapp-lib`).
- `RwSignal` (used by `FormModel`) must come from `leptos` in your crate.
- The derives just generate the `impl` blocks; the trait and type names are
  resolved at the call site.

---

## Setup

### Cargo.toml

```toml
[dependencies]
ferrox-webapp-macros = { path = "../ferrox-webapp-macros" }
# or once published:
# ferrox-webapp-macros = "0.1"
```

### Define the traits in your library crate

```rust
// In webapp-lib/src/lib.rs (or a traits module)

pub trait HasId {
    fn get_id(&self) -> String;
}

pub trait HasName {
    fn get_name(&self) -> String;
}
```

---

## `#[derive(HasId)]`

Generates `impl HasId for T` by cloning a `String` field named `id`.

### Default (field named `id`)

```rust
use webapp_lib::HasId;
use ferrox_webapp_macros::HasId;

#[derive(Clone, HasId)]
pub struct Location {
    pub id: String,
    pub name: String,
}
// Generated:
// impl HasId for Location {
//     fn get_id(&self) -> String { self.id.clone() }
// }
```

### Override with `#[has_id(field = "...")]`

Use this when the ID field has a different name (e.g. `email`, `code`).

```rust
use webapp_lib::HasId;
use ferrox_webapp_macros::HasId;

#[derive(Clone, HasId)]
#[has_id(field = "email")]
pub struct UserAccountRequest {
    pub email: String,
    pub display_name: String,
}
// Generated:
// impl HasId for UserAccountRequest {
//     fn get_id(&self) -> String { self.email.clone() }
// }
```

### Requirements

- The target field must be of type `String` (or any type that provides `.clone() -> String`).
- `HasId` trait must be in scope at the call site.

---

## `#[derive(HasName)]`

Generates `impl HasName for T` by cloning a `String` field named `name`.

### Default (field named `name`)

```rust
use webapp_lib::HasName;
use ferrox_webapp_macros::HasName;

#[derive(Clone, HasName)]
pub struct Location {
    pub id: String,
    pub name: String,
}
// Generated:
// impl HasName for Location {
//     fn get_name(&self) -> String { self.name.clone() }
// }
```

### Override with `#[has_name(field = "...")]`

```rust
use webapp_lib::HasName;
use ferrox_webapp_macros::HasName;

#[derive(Clone, HasName)]
#[has_name(field = "display_name")]
pub struct UserAccount {
    pub id: String,
    pub display_name: String,
}
// Generated:
// impl HasName for UserAccount {
//     fn get_name(&self) -> String { self.display_name.clone() }
// }
```

### Requirements

- The target field must be of type `String`.
- `HasName` trait must be in scope at the call site.

---

## `#[derive(FormModel)]`

Generates a reactive companion struct for Leptos form binding. Every field of
the original struct is wrapped in `RwSignal<T>`, and three methods are generated
to create, populate, and collect the form model.

### Example

```rust
use ferrox_webapp_macros::FormModel;

#[derive(Clone, FormModel)]
pub struct Deal {
    pub deal_number: String,
    pub volume: f64,
    pub deal_type: DealType,  // must implement Default + Clone
}
```

### Generated output

```rust
pub struct DealFormModel {
    pub deal_number: RwSignal<String>,
    pub volume:      RwSignal<f64>,
    pub deal_type:   RwSignal<DealType>,
}

impl DealFormModel {
    /// Create with default values — use for "new record" forms.
    pub fn new() -> Self { ... }

    /// Populate from an existing entity — use for "edit record" forms.
    pub fn from_deal(source: &Deal) -> Self { ... }

    /// Collect signal values back into the plain struct — use on form submit.
    pub fn to_deal(&self) -> Deal { ... }
}
```

The `from_*` / `to_*` method names are derived automatically from the struct
name using snake_case conversion (`DealLeg` → `from_deal_leg` / `to_deal_leg`).

### Usage in a Leptos component

```rust
#[component]
pub fn DealCaptureForm() -> impl IntoView {
    let model = DealFormModel::new();

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let deal = model.to_deal();
        // send deal to API...
    };

    view! {
        <form on:submit=on_submit>
            <TextInput value=model.deal_number label="Deal Number" />
            <NumberInput value=model.volume label="Volume" />
        </form>
    }
}
```

### Requirements

- `RwSignal` from `leptos` must be in scope.
- All field types must implement `Default` (used by `new()`).
- All field types must implement `Clone` (used by `from_*()`).
- The original struct must have named fields (tuple structs are not supported).

---

## Combining macros

You can stack `HasId`, `HasName`, and `FormModel` on the same struct:

```rust
use webapp_lib::{HasId, HasName};
use ferrox_webapp_macros::{HasId, HasName, FormModel};

#[derive(Clone, HasId, HasName, FormModel)]
#[has_name(field = "display_name")]
pub struct UserAccount {
    pub id: String,
    pub display_name: String,
    pub email: String,
    pub role: UserRole,
}
```

---

## Crate structure

```
ferrox-webapp-macros/
├── Cargo.toml   — proc-macro = true; deps: syn 2, quote, proc-macro2
└── src/
    └── lib.rs   — HasId, HasName, FormModel derives + helpers
```

---

## Adding to a workspace

If your project uses a Cargo workspace, add this crate to the workspace
members and reference it as a path dependency:

```toml
# workspace Cargo.toml
[workspace]
members = [
    "crates/your-app",
    "../ferrox-webapp-macros",   # path relative to workspace root
]
```

---

## Extending this crate

All macros live in `src/lib.rs`. To add a new derive:

1. Add a `#[proc_macro_derive(YourMacro, attributes(your_attr))]` function.
2. Parse the `DeriveInput` with `syn`.
3. Emit code with `quote!`.
4. Update this README.

Planned future macros (see design notes):

| Macro | Purpose |
|---|---|
| `Auditable` | `snapshot()` + `diff()` for audit trails |
| `ApiResponse` | Wraps a type in a standard `{ data, meta, errors }` envelope |
| `Filterable` | Generates an `Option<T>`-per-field filter struct + `to_query_params()` |
| `EnumSelect` | Generates `label()`, `all_variants()`, `from_str()` for dropdown enums |
| `Validatable` | Field-level validation via attributes, generates `validate() -> Result` |
