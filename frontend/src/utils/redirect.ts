/**
 * Extract a safe same-origin path from a `redirect` query value.
 *
 * Only absolute paths on this origin are allowed; protocol-relative and
 * backslash variants (`//evil.com`, `/\evil.com`) are rejected so the value
 * cannot bounce the browser to another site.
 */
export function safeRedirect(value: unknown): string | null {
  if (typeof value !== 'string') return null;
  if (!value.startsWith('/')) return null;
  if (value.startsWith('//') || value.startsWith('/\\')) return null;
  return value;
}
