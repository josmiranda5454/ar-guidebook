export function formatReviewStatus(status) {
  return status
    .split("_")
    .map((part) => part[0].toUpperCase() + part.slice(1))
    .join(" ");
}

export function formatDateTime(value) {
  if (!value) {
    return "Not reviewed";
  }

  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(value));
}

export function formatAlignment(alignment) {
  if (!alignment) {
    return "No alignment";
  }

  return [
    `x ${formatMeters(alignment.horizontal_offset_meters)}`,
    `y ${formatMeters(alignment.vertical_offset_meters)}`,
    `z ${formatMeters(alignment.depth_offset_meters)}`,
    `scale ${alignment.scale.toFixed(2)}x`,
  ].join("  ");
}

export function formatMeters(value) {
  const sign = value >= 0 ? "+" : "";
  return `${sign}${value.toFixed(1)}m`;
}
