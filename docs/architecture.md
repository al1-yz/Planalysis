# Architecture

Planalysis is a desktop application for Windows and Linux: a Tauri 2 shell around a React
frontend, with all data in a local SQLite database owned by the Rust side. There is no backend,
no account and no network dependency; everything works offline and the data never leaves the
computer.

## Layers

```
features / app (React UI)
        ↓
application (use cases)
        ↓
domain (pure types, rules, repository interfaces)
        ↑
infrastructure/tauri (repositories that call the Rust commands)
        │  Tauri IPC (JSON)
Rust commands → store (SQLite operations) → SQLite file
```

| Folder                  | Contains                                                                      | May import                      |
| ----------------------- | ----------------------------------------------------------------------------- | ------------------------------- |
| `src/domain`            | Types, pure financial rules, entities, repository interfaces                  | other domain modules only       |
| `src/application`       | Use cases that coordinate rules and repositories                              | domain                          |
| `src/infrastructure`    | Repository implementations that call the Rust commands                        | domain                          |
| `src/features/<name>`   | Pages, forms, tables, hooks for one feature                                   | application, domain, components |
| `src/components/ui`     | shadcn/Radix primitives, restyled with Planalysis tokens                      | lib                             |
| `src/components/common` | Composed, app-specific UI building blocks                                     | ui, lib                         |
| `src/app`               | Router, shell, navigation, composition root (opens the database, wires repos) | everything                      |
| `src/lib`               | Small generic helpers                                                         | —                               |
| `src-tauri`             | The desktop shell, IPC commands, SQLite schema, migrations and persistence    | —                               |

The domain, application and infrastructure import boundaries are enforced by
`no-restricted-imports` in `eslint.config.js`; the domain and application layers cannot import
the Tauri runtime.

`DatabaseGate` (`src/app/database-gate.tsx`) is the composition root. It asks Rust whether the
database opened, then provides the repositories to features through React context. When the
database cannot be opened it shows the reason in Portuguese and stops. It never recreates,
overwrites or resets an existing database. Outside the desktop application (the browser, tests) it
renders the app untouched.

## Where rules live

| Kind of rule                                                                 | Lives in                             |
| ---------------------------------------------------------------------------- | ------------------------------------ |
| Value rules: cents, dates, installments, invoices, entity and transfer shape | TypeScript domain                    |
| Rules that need the clock or identity: no future dates, timestamps, ids      | TypeScript application (injected)    |
| Relational integrity: references, unique active names, deleting in use       | SQLite constraints                   |
| Timeline integrity: date not before the account's opening date               | SQLite triggers                      |
| Atomic multi-row writes: transfers                                           | Rust, one SQLite transaction         |
| Policy: no new use of archived accounts, cards or categories                 | Rust, inside the writing transaction |

Rust never decides what a financial rule means; it refuses writes that violate integrity and
reports a typed error code (`{ code, message }`), which the interface translates into text. The
domain code is the only place that computes totals, balances and classifications.

## Persistence

- **One connection.** The Rust side opens the database once and serializes every operation
  through it. Each write command runs in a single `IMMEDIATE` transaction. SQLite is
  synchronous, so commands run on a blocking thread, never on the UI thread.
- **Pragmas.** Foreign keys are on (verified at open), write-ahead logging, `synchronous = FULL`.
- **Location.** `planalysis.db` in the application's local data directory (the folder is named
  after the bundle identifier). Debug builds use `planalysis-dev.db`, so development can never
  touch real data.
- **Identity and versions.** The SQLite header carries `application_id` (`PLAN`) and
  `user_version` (the schema version). A file that is not Planalysis's, or that is newer than the
  running build, is refused without being written to.
- **Migrations.** Forward-only SQL files in `src-tauri/migrations`, each applied in its own
  transaction and followed by a foreign key check. A failing migration rolls back completely.
- **Money, dates, ids.** Money is integer cents (|value| ≤ 2^53 − 1, so it is exact in
  JavaScript). Dates are `YYYY-MM-DD` text, timestamps ISO-8601 UTC text, ids lowercase UUIDs.
  Constraints in the schema enforce the formats, so bad data is refused whichever code path
  writes it.
- **Names.** Account, category and card names are unique among active (non-archived) records,
  ignoring case and spacing. The domain computes a normalized `nameKey`; partial unique indexes on
  it enforce the rule in the database. Accents stay significant.
- **Deleting.** An account, card or category that anything refers to cannot be deleted; it is
  archived instead. Transactions can be edited and deleted freely.
- **Transfers.** A transfer is two linked transactions (equal and opposite amounts, same date,
  two different accounts, no category) written, replaced and removed together. Single-transaction
  operations refuse transfer legs.
- **Invoices** are assigned by domain rules from purchases and card settings. Whether they are
  also persisted is provisional, as is how transfers are represented in storage; both will be
  revisited with the credit card and transaction features.
- **Wire format.** Rust and TypeScript exchange the same JSON, pinned by the shared fixtures in
  `contract/`, which both test suites read.

## State

- Persistent financial data: the database, accessed through repositories. It is the source of
  truth.
- Derived values (totals, balances, net worth): computed by domain functions, not stored.
- UI state: local React state first. Zustand only for genuinely shared UI state (e.g. selected
  month), introduced when needed.

## Routing

React Router in client-side (data router) mode. Routes are defined in `src/app/routes.tsx`;
navigation entries in `src/app/navigation.ts`. Unimplemented sections render a placeholder page.
User-facing URL paths are Portuguese (`/transacoes`, `/contas`, …).

## Styling

Tailwind CSS v4 with CSS-first configuration. Design tokens are defined in
`src/styles/tokens.css` and mapped to utilities in `src/styles/index.css`. Tailwind's default
color palette is disabled so only semantic tokens can be used. System fonts only (no font
downloads).

### Theming

- **Dark is the default and primary theme.** It is defined on `:root`, including
  `color-scheme: dark`.
- **All theme-dependent values** (colors, overlay, shadows, color-scheme) live in `tokens.css`.
  `index.css` only maps them to utilities; components never contain raw colors.
- **Foreground-on-fill pairs** are named for their background (`on-brand`, `on-danger`), so they
  stay correct in any theme.
- **Adding a light theme later:** add a `:root[data-theme="light"]` block in `tokens.css` that
  redefines the same `--pl-*` properties, plus whatever sets the attribute. No component changes.
- **Contrast targets:** text ≥ 4.5:1 on canvas, surface and surface-muted; focus ring and
  indicators ≥ 3:1. Keep token colors inside the sRGB gamut so the computed contrast matches what
  browsers render.

## Security posture

- The webview loads only the bundled frontend, under a strict Content Security Policy whose only
  connection is to the IPC bridge; there is no remote content, no updater and no network access.
- The webview has no SQL access. It can only call the application's own commands.
- Only one instance runs at a time, so two processes never share a database file.
- The database file is not encrypted; protection comes from the operating system account and disk
  encryption. Backups are plain database copies.

## Development

Prerequisites: Node 22.12+, the Rust toolchain pinned in `rust-toolchain.toml`, and the platform
build dependencies from the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)
(on Windows, the Visual Studio C++ Build Tools; on Linux, WebKitGTK and a few libraries).

- `npm run tauri dev` — the desktop app with hot reload (uses the development database)
- `npm run tauri build` — release build and installers (NSIS on Windows; deb and AppImage on Linux)
- `npm run verify` — typecheck, lint, format check, unit tests, web build, then `rustfmt`,
  `clippy` and `cargo test`

## Testing

| Level               | Location                            | Tooling                                        |
| ------------------- | ----------------------------------- | ---------------------------------------------- |
| Unit                | `src/**/*.test.ts`                  | Vitest                                         |
| Component           | `src/**/*.test.tsx`                 | Vitest + jsdom + React Testing Library         |
| Repository adapters | `src/infrastructure/**`             | Vitest + Tauri `mockIPC` + shared fixtures     |
| Schema, persistence | `src-tauri/src/**` (`#[cfg(test)]`) | `cargo test` on in-memory and temporary SQLite |
| Commands (IPC)      | `src-tauri/tests`                   | `cargo test` on Tauri's mock runtime           |
| E2E (web shell)     | `e2e/*.spec.ts`                     | Playwright against the production web build    |

E2E projects run at 1440px (`desktop`) and 390px (`mobile`). Tests tagged `@desktop` / `@mobile`
run only in the matching project. The shared fixture in `e2e/fixtures.ts` fails any test that
produces a browser console error. Real-window desktop tests are not automated yet.

## Toolchain notes

- TypeScript is pinned to `~6.0` because `typescript-eslint` does not yet support TypeScript 7.
- jsdom is pinned to `^29` because jsdom 30 requires Node ≥ 24.15.
- Rust is pinned in `rust-toolchain.toml` (and in the CI workflows, which must be kept in sync).
  Tauri requires Rust 1.90 or newer.
- On Windows, Rust test executables that link Tauri need a Common Controls manifest; `build.rs`
  embeds `windows-test.manifest` into them.
