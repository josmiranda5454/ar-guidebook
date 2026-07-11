# ClimbAR Admin

Static admin UI for guidebook editing and uploaded AR calibration capture review.

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

Hosted static deployments can set the API URL in `config.js`:

```js
globalThis.CLIMBAR_API_BASE_URL = "https://api.example.com/api/v1";
```

## Current Capabilities

- Browse area, wall, and route hierarchy.
- Create draft areas, walls, routes, and first AR overlays.
- Edit core route fields for existing routes.
- Edit AR overlay placement metadata for existing overlays.
- Edit AR route trace coordinate space and trace points.
- List uploaded AR calibration captures.
- Filter captures by route ID or overlay ID.
- Review captures as pending, good candidate, rejected, or applied.
- Save reviewer notes.
- Apply a capture to its overlay as the overlay default alignment.

## Planned Responsibilities

- Delete flows for areas, walls, routes, and overlays.
- Rich create forms for areas, walls, routes, and overlays.
- Photo and topo upload.
- Visual route trace editor.
- Offline pack publishing.
