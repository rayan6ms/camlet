# Localization Guide

Camlet keeps all UI translations local to the repository. There is no remote loading, no machine translation service, and no runtime language pack download path.

## Source Of Truth

- English (`en`) is the source-of-truth locale.
- The canonical locale shape lives in [`src/renderer/locales/schema.ts`](./src/renderer/locales/schema.ts).
- The English strings live in [`src/renderer/locales/en.ts`](./src/renderer/locales/en.ts).
- All other locales must match the English structure exactly.

## Supported Locales

The app currently ships these UI locales:

- `en`
- `pt-BR`
- `es`
- `fr`
- `de`
- `it`
- `ja`

The shared language list and system-locale resolution rules live in [`src/shared/language.ts`](./src/shared/language.ts).

## Locale File Layout

- [`src/renderer/locales/index.ts`](./src/renderer/locales/index.ts) registers all renderer locales.
- [`src/renderer/locales/schema.ts`](./src/renderer/locales/schema.ts) defines the required translation object shape.
- One file per locale keeps review diff noise low and makes missing coverage easier to spot.

The `language.options` section should use native labels for selectable languages so the language picker remains usable even when the current UI language is unfamiliar.

## Adding A New Language

1. Add the new locale code to `supportedLanguages` in [`src/shared/language.ts`](./src/shared/language.ts).
2. Update `resolveSupportedLanguage()` if the new language needs system-locale prefix mapping.
3. Create a new locale file under [`src/renderer/locales/`](./src/renderer/locales/).
4. Register the locale in [`src/renderer/locales/index.ts`](./src/renderer/locales/index.ts).
5. Add the new language label to `language.options` in every locale file.
6. Run the verification commands listed below.

## Translation Safety Checks

Camlet uses two safeguards:

- TypeScript: each locale file is typed against `RendererLocale`, which catches missing or invalid keys during typecheck.
- Vitest: [`tests/unit/locales.test.ts`](./tests/unit/locales.test.ts) compares every locale against English and fails on missing keys, extra keys, or shape mismatches.

Run these commands after changing translations:

```bash
pnpm typecheck
pnpm test -- --run tests/unit/locales.test.ts
pnpm check
```

`pnpm verify` runs the full project validation used in CI.

## Translation Maintenance Rules

- Add new UI strings to English first.
- Keep key names stable and grouped by feature area.
- Prefer short, direct product language over literal word-for-word translations.
- Avoid mixing translated labels with untranslated option values, except where native language names are intentional in the language picker.
- If a string is ambiguous, add a short code comment near the usage site rather than encoding translator notes into the key name.
