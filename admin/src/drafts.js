export function slugify(value) {
  return value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

export function draftArea(name, location = defaultLocation()) {
  return {
    id: crypto.randomUUID(),
    parent_area_id: null,
    name,
    slug: slugify(name),
    description: "Draft area description.",
    access_notes: null,
    location,
    walls: [],
  };
}

export function draftWall(name, area, location = area.location) {
  return {
    id: crypto.randomUUID(),
    area_id: area.id,
    name,
    slug: slugify(name),
    description: "Draft wall description.",
    approach_notes: null,
    aspect: null,
    location,
    routes: [],
  };
}

export function draftRoute(name, wall, location = wall.location) {
  return {
    id: crypto.randomUUID(),
    wall_id: wall.id,
    name,
    slug: slugify(name),
    grade: "5.7",
    grade_system: "yosemite_decimal",
    route_types: ["sport"],
    length_feet: null,
    pitches: 1,
    stars_average: null,
    rating_votes: 0,
    first_ascent: null,
    description: "Draft route description.",
    location_notes: "Add start and wall location notes.",
    protection_notes: null,
    safety_notes: null,
    location,
    media: [],
    ar_overlays: [],
  };
}

export function draftOverlay(route) {
  return {
    id: crypto.randomUUID(),
    route_id: route.id,
    version: nextOverlayVersion(route),
    anchor_strategy: "manual_alignment",
    gps_hint: route.location,
    compass_bearing_degrees: null,
    wall_plane: null,
    route_trace: {
      coordinate_space: "normalized_wall_image",
      points: [
        { x: 0.5, y: 0.95, z: null },
        { x: 0.5, y: 0.2, z: null },
      ],
    },
    default_alignment: {
      horizontal_offset_meters: 0,
      vertical_offset_meters: 0,
      depth_offset_meters: 0,
      scale: 1,
    },
    confidence: "draft",
    reviewed_at: null,
  };
}

function nextOverlayVersion(route) {
  const versions = route.ar_overlays.map((overlay) => overlay.version);
  return versions.length > 0 ? Math.max(...versions) + 1 : 1;
}

function defaultLocation() {
  return {
    latitude: 34.0103,
    longitude: -116.1669,
    elevation_meters: null,
  };
}
