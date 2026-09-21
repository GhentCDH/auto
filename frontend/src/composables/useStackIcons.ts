import { ref } from 'vue';
import { addCollection } from '@iconify/vue';

// Module-level singleton: matches free-text stack names to the bundled Simple
// Icons set. The icon data is large, so it is imported lazily and only once,
// and only when the feature is enabled via `stack_icons_enabled`.
const iconsReady = ref(false);
let loadPromise: Promise<void> | null = null;
let index: Map<string, string> | null = null;
let colors: Map<string, string> | null = null;

/** Shape of the official simple-icons metadata used for brand colours. */
interface SimpleIconDatum {
  slug: string;
  hex: string;
}

/**
 * Normalize a name for matching: lowercase, spell out the symbol characters
 * that appear in language names (`C#` -> `csharp`), then drop everything that
 * is not alphanumeric (`Node.js` -> `nodejs`).
 */
function normalize(name: string): string {
  return name
    .trim()
    .toLowerCase()
    .replace(/#/g, 'sharp')
    .replace(/\+/g, 'plus')
    .replace(/[^a-z0-9]/g, '');
}

// Names whose Simple Icons slug differs from the obvious normalized form.
const SYNONYMS: Record<string, string> = {
  net: 'dotnet',
  netcore: 'dotnet',
  aspnet: 'dotnet',
  aspnetcore: 'dotnet',
  k8s: 'kubernetes',
  postgres: 'postgresql',
  aws: 'amazonwebservices',
  amazon: 'amazonwebservices',
  gcp: 'googlecloud',
  gcloud: 'googlecloud',
  mssql: 'microsoftsqlserver',
  sqlserver: 'microsoftsqlserver',
  elastic: 'elasticsearch',
  golang: 'go',
  java: 'openjdk',
  jdk: 'openjdk',
  kafka: 'apachekafka',
  tailwind: 'tailwindcss',
  socketio: 'socketdotio',
  nuxtjs: 'nuxt',
  d3js: 'd3',
};

/** Register one spelling of a name against an icon slug. */
function addName(target: Map<string, string>, name: string, key: string): void {
  const n = normalize(name);
  if (n && !target.has(n)) {
    target.set(n, key);
  }
}

/**
 * Register an icon slug plus the spellings people actually type: the slug with
 * `dot` dropped (`nodedotjs` -> `nodejs`) and without the `js` suffix
 * (`nodedotjs` -> `node`).
 */
function addKey(target: Map<string, string>, key: string): void {
  addName(target, key, key);
  const n = normalize(key);
  if (n.includes('dotjs')) {
    addName(target, n.replace('dotjs', 'js'), key);
    addName(target, n.replace('dotjs', ''), key);
  }
}

/** Candidate normalized spellings for an input name. */
function candidates(name: string): string[] {
  const n = normalize(name);
  const list = [n];
  if (n.endsWith('js') && n.length > 2) {
    list.push(n.slice(0, -2));
  }
  return list;
}

/** Resolve a stack name to a Simple Icons slug, or `undefined`. */
function resolveKey(name: string): string | undefined {
  if (!index) return undefined;
  for (const candidate of candidates(name)) {
    const key = SYNONYMS[candidate] ?? index.get(candidate);
    if (key && index.has(normalize(key))) {
      return key;
    }
  }
  return undefined;
}

/** Resolve a stack name to an Iconify icon reference, or `undefined`. */
export function iconFor(name: string): string | undefined {
  const key = resolveKey(name);
  return key ? `simple-icons:${key}` : undefined;
}

/** Brand colour for a stack name as `#RRGGBB`, or `undefined`. */
export function colorFor(name: string): string | undefined {
  const key = resolveKey(name);
  return key && colors ? colors.get(key) : undefined;
}

/** Load and index the icon set once; concurrent callers share the request. */
export function loadStackIcons(): Promise<void> {
  if (!loadPromise) {
    loadPromise = Promise.all([
      import('@iconify-json/simple-icons'),
      import('simple-icons/icons.json'),
    ]).then(([{ icons }, dataModule]) => {
      addCollection(icons);

      const target = new Map<string, string>();
      for (const key of Object.keys(icons.icons ?? {})) {
        addKey(target, key);
      }
      for (const [alias, data] of Object.entries(icons.aliases ?? {})) {
        const parent = data?.parent;
        if (parent && icons.icons?.[parent]) {
          addName(target, alias, parent);
        }
      }
      index = target;

      const data = (dataModule as unknown as { default: SimpleIconDatum[] })
        .default;
      const palette = new Map<string, string>();
      for (const entry of data ?? []) {
        if (entry.slug && entry.hex) {
          palette.set(entry.slug, `#${entry.hex}`);
        }
      }
      colors = palette;

      iconsReady.value = true;
    });
  }
  return loadPromise;
}

export function useStackIcons() {
  return { iconsReady, loadStackIcons, iconFor, colorFor };
}
