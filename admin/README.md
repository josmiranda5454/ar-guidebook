# ClimbAR Admin

Static admin UI for reviewing uploaded AR calibration captures.

## Run Locally

Start the backend first:

```sh
cd ../backend
cargo run
```

Then run the admin UI:

```sh
npm run dev
```

Open `http://127.0.0.1:5173`.

The UI defaults to `http://127.0.0.1:8080/api/v1`, but you can change the API
base URL in the page.

## Current Capabilities

- List uploaded AR calibration captures.
- Filter captures by route ID or overlay ID.
- Review captures as pending, good candidate, rejected, or applied.
- Save reviewer notes.
- Apply a capture to its overlay as the overlay default alignment.

## Planned Responsibilities

- Area, wall, and route CRUD.
- Photo and topo upload.
- Route trace editor.
- Offline pack publishing.
