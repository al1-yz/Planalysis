# Architecture

Planalysis is a client-only React application. All persistent data is stored locally in
IndexedDB. There is no backend; the persistence boundary is kept clean so a sync layer could be
added later without rewriting domain logic or UI.

## Layers

```
features / app (React UI)
        ↓
application (use cases)
        ↓
domain (pure types, rules, repository interfaces)
        ↑
infrastructure (Dexie repositories, backup/export) — implements domain interfaces
```

| Folder                  | Contains                                                                     | May import                      |
| ----------------------- | ---------------------------------------------------------------------------- | ------------------------------- |
| `src/domain`            | Types, pure financial rules, `MoneyCents`/`LocalDate` utils, repo interfaces | other domain modules only       |
| `src/application`       | Use cases that coordinate rules and repositories                             | domain                          |
| `src/infrastructure`    | Dexie database, migrations, repository implementations, backup/export        | domain                          |
| `src/features/<name>`   | Pages, forms, tables, hooks for one feature                                  | application, domain, components |
| `src/components/ui`     | shadcn/Radix primitives, restyled with Planalysis tokens                     | lib                             |
| `src/components/common` | Composed, app-specific UI building blocks                                    | ui, lib                         |
| `src/app`               | Router, shell, navigation, composition root (wires repositories)             | everything                      |
| `src/lib`               | Small generic helpers                                                        | —                               |

The domain and application import boundaries are enforced by `no-restricted-imports` in
`eslint.config.js`.

Concrete repositories are created once in `src/app` and provided to features through React
context. No dependency-injection framework is used.

## State

- Persistent financial data: database, accessed through repositories. It is the source of truth.
- Derived values (totals, balances, net worth): computed by domain functions, not stored.
- Invoices: assigned by domain rules from purchases and card settings. Whether they are also
  persisted is provisional and will be decided with the credit card feature.
- Transfers: how a transfer is represented in storage is provisional and will be decided with the
  transactions feature.
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

## Testing

| Level              | Location                | Tooling                                 |
| ------------------ | ----------------------- | --------------------------------------- |
| Unit               | `src/**/*.test.ts`      | Vitest                                  |
| Component          | `src/**/*.test.tsx`     | Vitest + jsdom + React Testing Library  |
| Repository (later) | `src/infrastructure/**` | Vitest + fake-indexeddb                 |
| E2E                | `e2e/*.spec.ts`         | Playwright against the production build |

E2E projects run at 1440px (`desktop`) and 390px (`mobile`). Tests tagged `@desktop` / `@mobile`
run only in the matching project. The shared fixture in `e2e/fixtures.ts` fails any test that
produces a browser console error.

## Toolchain notes

- TypeScript is pinned to `~6.0` because `typescript-eslint` does not yet support TypeScript 7.
- jsdom is pinned to `^29` because jsdom 30 requires Node ≥ 24.15.
