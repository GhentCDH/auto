// DaisyUI badge colors for consistent stack coloring
const badgeColors = [
  'badge-primary',
  'badge-secondary',
  'badge-accent',
  'badge-info',
  'badge-success',
  'badge-warning',
  'badge-error',
];

/**
 * Get a deterministic badge color class based on a name.
 * The same name will always produce the same color.
 */
export function getStackColor(name: string): string {
  // Simple hash function for consistent color
  let hash = 0;
  for (let i = 0; i < name.length; i++) {
    hash = name.charCodeAt(i) + ((hash << 5) - hash);
  }
  return badgeColors[Math.abs(hash) % badgeColors.length];
}
