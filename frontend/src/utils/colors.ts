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

/** Relative luminance (0..1) of a `#RRGGBB` colour, per WCAG. */
function luminance(hex: string): number {
  const value = hex.replace('#', '');
  if (value.length !== 6) return 0;
  const channel = (offset: number) => {
    const c = parseInt(value.slice(offset, offset + 2), 16) / 255;
    return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
  };
  return 0.2126 * channel(0) + 0.7152 * channel(2) + 0.0722 * channel(4);
}

/**
 * Pick black or white text for the best contrast against a `#RRGGBB`
 * background. The threshold is where both choices contrast equally.
 */
export function readableTextColor(hex: string): string {
  return luminance(hex) > 0.179 ? '#18181b' : '#ffffff';
}
