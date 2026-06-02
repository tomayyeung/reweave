# AGENTS.md

## Repo Shape
- Rust workspace members are `reweave` and `frontend`; `wordlist` is intentionally excluded from the workspace.
- `reweave/` is both the shared Rust game logic crate and the Vercel backend. Shared logic lives in `reweave/src/common/`; Vercel function binaries live in `reweave/api/`.
- `frontend/` has React/Vite source in `frontend/src/*.tsx` plus a Rust `cdylib` in `frontend/src/lib.rs` compiled to WASM.
- `frontend/pkg/` is wasm-pack output and ignored by `frontend/pkg/.gitignore`; regenerate it instead of editing it.
- Vite aliases are `@` -> `frontend/src`, `@components` -> `frontend/src/components`, and `@wasm` -> `frontend/pkg`.

## Commands
- Install frontend deps from the repo root with `pnpm --dir frontend install`; the root `package.json` has no scripts.
- Build WASM before Vite build: `wasm-pack build frontend --target bundler --out-dir pkg`.
- Build frontend like deploy: `wasm-pack build frontend --target bundler --out-dir pkg && pnpm --dir frontend run build`.
- Run frontend dev server: `pnpm --dir frontend run dev`.
- Lint frontend: `pnpm --dir frontend run lint`.
- Test shared/backend Rust crate: `cargo test -p reweave`.
- Run one Rust test with a filter, for example `cargo test -p reweave common::board::tests::find1`.
- Check the WASM crate against its real target with `cargo check -p frontend --target wasm32-unknown-unknown`.

## Runtime And Deploy
- Root `vercel.json` deploys the frontend, installs `wasm32-unknown-unknown` and `wasm-pack`, then builds `frontend/dist`.
- Backend is deployed separately from `reweave/`; `reweave/vercel.json` rewrites `/api/puzzle/:puzzle_id` to `/api/puzzle`.
- Vite proxies `/api` to `http://localhost:3000`; run the backend dev server there when testing frontend API calls locally.
- Frontend API base is `VITE_API_URL`; if unset it uses same-origin/proxied `/api`.
- Frontend routes are `/create` and `/play/:puzzleId`; there is no root route in `App.tsx`.

## Data And Env
- Backend DB code requires `DATABASE_URL` unless `USE_LOCAL_FILES` is set.
- With `USE_LOCAL_FILES`, puzzles are read/written under `../puzzles/` relative to the backend process.
- No SQL migrations are present in the repo; do not assume a migration workflow exists.

## Gotchas
- API shape is mixed: `POST /api/create` reads a JSON body, while `GET /api/puzzle` reads `puzzle_id` from the query string or final path segment.
- Board/trie logic expects lowercase ASCII letters; blanks/holes are represented by `_` and `!` in Rust board creation.
- The WASM crate embeds `wordlist/wordlist.txt` with `include_str!`; update that file before rebuilding WASM if changing the playable dictionary.
- The word list generator in `wordlist/` depends on local `CSW24.txt` and `blacklist.txt`; those inputs are gitignored.
