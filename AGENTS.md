# AGENTS.md

## Repo Shape
- Rust workspace members are `reweave` and `frontend`; `wordlist` is intentionally excluded from the workspace.
- `reweave/` is both the shared Rust game logic crate and the Vercel backend. Shared logic lives in `reweave/src/common/`; Vercel function binaries live in `reweave/api/`.
- `frontend/` has React/Vite source in `frontend/src/` plus a Rust `cdylib` in `frontend/src/lib.rs` compiled to WASM.
- Frontend pages are in `frontend/src/pages/create/` and `frontend/src/pages/play/`; shared UI components are in `frontend/src/components/`.
- `frontend/pkg/` is wasm-pack output and ignored by `frontend/pkg/.gitignore`; regenerate it instead of editing it.
- Vite aliases are `@` -> `frontend/src`, `@components` -> `frontend/src/components`, and `@wasm` -> `frontend/pkg`.
- The frontend uses React 19 with the React Compiler configured in `frontend/vite.config.ts` via `reactCompilerPreset()`.

## Commands
- The root `package.json` only declares `packageManager: pnpm@10.28.0`; it has no scripts.
- Install frontend deps from the repo root with `pnpm --dir frontend install`.
- Build WASM before Vite build: `wasm-pack build frontend --target bundler --out-dir pkg`.
- Build frontend like deploy: `wasm-pack build frontend --target bundler --out-dir pkg && pnpm --dir frontend run build`.
- Type-check/build the current checked-in frontend package with `pnpm --dir frontend run build`; this assumes `frontend/pkg/` already exists.
- Run frontend dev server: `pnpm --dir frontend run dev`.
- Lint frontend: `pnpm --dir frontend run lint`.
- Test shared/backend Rust crate: `cargo test -p reweave`.
- Run one Rust test with a filter, for example `cargo test -p reweave common::board::tests::find1`.
- Check the WASM crate against its real target with `cargo check -p frontend --target wasm32-unknown-unknown`.
- Check Rust formatting with `cargo fmt --check`; run `cargo fmt` if Rust edits need formatting.

## Runtime And Deploy
- Root `vercel.json` deploys the frontend, installs `wasm32-unknown-unknown`, installs `wasm-pack`, installs global `pnpm@9`, then builds `frontend/dist`.
- Backend is deployed separately from `reweave/`; `reweave/vercel.json` rewrites `/api/puzzle/:puzzle_id` to `/api/puzzle?puzzle_id=:puzzle_id` and includes CORS headers for `/api/(.*)`.
- Vite proxies `/api` to `http://localhost:3000`; run the backend dev server there when testing frontend API calls locally.
- Frontend API base is `VITE_API_URL`; if unset it uses same-origin/proxied `/api`.
- Frontend routes are `/create` and `/play/:puzzleId`; there is no root route in `App.tsx`.

## Data And Env
- Backend DB code requires `DATABASE_URL` unless `USE_LOCAL_FILES` is set.
- With `USE_LOCAL_FILES`, puzzles are read/written under `../puzzles/` relative to the backend process.
- The database code expects a `puzzles` table with `id`, `name`, `width`, `height`, `letters`, `words`, and `answer`; `id` is returned as a UUID string.
- No SQL migrations are present in the repo; do not assume a migration workflow exists.

## Game And API Shape
- `POST /api/create` accepts JSON with `name`, `width`, `height`, `letters`, `words`, and `answer`, then returns `{ "id": string }` or `{ "error": string }`.
- `GET /api/puzzle` reads `puzzle_id` from the query string; `GET /api/puzzle/:puzzle_id` is supported by Vercel rewrite and by a fallback path-segment parser.
- Rust `Puzzle` stores `letters` as the starting puzzle state, `answer` as the solved board, and `words` as a `HashSet<String>`.
- Frontend word-list data is a discriminated union: `CreateWords` has `kind: "create"` and `all`, while `PlayWords` has `kind: "play"`, `found`, `missing`, and `extra`.
- `WordList` props are also discriminated by `listType`; do not reintroduce optional word arrays plus non-null assertions.

## Gotchas
- Board/trie logic expects lowercase ASCII letters; blanks and holes are represented by `_` and `!` in Rust board creation.
- `Board::create` and `Puzzle::create` return `Result`; invalid board dimensions or invalid letters should be handled as errors, not panics.
- The WASM `check` return type is `any` in generated typings; cast it at frontend boundaries into `PlayWords` rather than spreading `any` through component props.
- The WASM crate embeds `wordlist/wordlist.txt` with `include_str!`; update that file before rebuilding WASM if changing the playable dictionary.
- The word list generator in `wordlist/` depends on local `CSW24.txt` and `blacklist.txt`; those inputs are gitignored.
- `frontend/pkg/` and `frontend/dist/` are generated. Do not manually edit generated WASM package files or built assets.
