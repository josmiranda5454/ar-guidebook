export function tracePointsToText(points) {
  return points
    .map((point) => [point.x, point.y, point.z ?? ""].join(","))
    .join("\n");
}

export function parseTracePoints(value) {
  const points = value
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)
    .map(parseTracePoint);

  if (points.length < 2) {
    throw new Error("Route trace needs at least two points.");
  }

  return points;
}

export function validateNormalizedTrace(points) {
  for (const point of points) {
    if (point.x < 0 || point.x > 1 || point.y < 0 || point.y > 1) {
      throw new Error("Normalized wall-image trace points must be between 0 and 1.");
    }
  }
  return points;
}

function parseTracePoint(line) {
  const parts = line.split(",").map((part) => part.trim());

  if (parts.length < 2 || parts.length > 3) {
    throw new Error(`Trace point "${line}" must be x,y or x,y,z.`);
  }

  const [x, y, z] = parts.map((part) => (part === "" ? null : Number.parseFloat(part)));

  if (!Number.isFinite(x) || !Number.isFinite(y)) {
    throw new Error(`Trace point "${line}" has an invalid x or y value.`);
  }

  if (z !== null && z !== undefined && !Number.isFinite(z)) {
    throw new Error(`Trace point "${line}" has an invalid z value.`);
  }

  return {
    x,
    y,
    z: z ?? null,
  };
}
