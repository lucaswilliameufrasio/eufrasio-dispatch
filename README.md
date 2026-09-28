# Eufrasio Dispatch

A working public incident desk split into an overview, incident log, incident creation form, and individual incident detail pages. Incidents and operator updates are written to `dispatch.sqlite` with Toasty; public URLs use UUIDv7 identifiers, while the app retains its internal row keys. Connected viewers on an incident detail page receive timeline changes over Topcoat's live WebSocket rendering.

## Run

Install the Topcoat CLI once, then start the app:

```sh
cargo install topcoat-cli
topcoat dev
```

Open <http://127.0.0.1:3000>. Browse `/incidents`, create one at `/incidents/new`, then open its detail page to post updates. A first run inserts one clearly labeled example incident into SQLite so the detail and timeline screens are immediately explorable; new records are created by actual forms and persist locally.
