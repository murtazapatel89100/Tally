import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import sitemap from '@astrojs/sitemap';

const SITE = 'https://tally.rs';
const TITLE = 'Tally — Plain-text accounting for the terminal';
const DESCRIPTION =
  'Plain-text double-entry accounting for the terminal. ledger\'s data model, a genuinely pleasant TUI. Single binary, no dependencies, your data in a text file you own.';

const jsonLd = JSON.stringify({
  '@context': 'https://schema.org',
  '@type': 'SoftwareApplication',
  name: 'Tally',
  description: DESCRIPTION,
  url: SITE,
  downloadUrl: 'https://github.com/murtazapatel89100/Tally/releases',
  codeRepository: 'https://github.com/murtazapatel89100/Tally',
  applicationCategory: 'FinanceApplication',
  applicationSubCategory: 'Accounting',
  operatingSystem: 'Linux, macOS, Windows',
  programmingLanguage: 'Rust',
  license: 'https://opensource.org/licenses/MIT',
  isAccessibleForFree: true,
  offers: { '@type': 'Offer', price: '0', priceCurrency: 'USD' },
  keywords: [
    'accounting', 'plain-text accounting', 'double-entry', 'terminal', 'TUI',
    'ledger', 'hledger', 'finance', 'personal finance', 'Rust',
  ],
  screenshot: `${SITE}/screenshots/ss-dashboard.png`,
  featureList: [
    'Double-entry bookkeeping',
    'Interactive terminal UI',
    'In-app transaction entry with autocomplete',
    'Net-worth sparkline and expense bar charts',
    'Budget gauges',
    'ledger / hledger file format compatible',
    'Exact decimal arithmetic',
    'Single self-contained binary',
  ],
  softwareVersion: '0.1.0',
});

export default defineConfig({
  site: SITE,
  // Use Astro's no-op passthrough image service so the build needs no native
  // `sharp` binary (keeps CI installs script-free and reproducible). The docs
  // site's images are small PNGs that don't require optimization.
  image: {
    service: { entrypoint: 'astro/assets/services/noop' },
  },
  integrations: [
    sitemap(),
    starlight({
      title: 'Tally',
      description: DESCRIPTION,
      logo: {
        dark: './src/assets/logo-dark.png',
        light: './src/assets/logo-light.png',
        alt: 'Tally',
        replacesTitle: false,
      },
      favicon: '/favicon.ico',
      social: {
        github: 'https://github.com/murtazapatel89100/Tally',
      },
      head: [
        // ── Canonical / charset ───────────────────────────────────────────────
        { tag: 'meta', attrs: { charset: 'utf-8' } },
        { tag: 'meta', attrs: { name: 'viewport', content: 'width=device-width, initial-scale=1' } },
        // ── Favicon suite ─────────────────────────────────────────────────────
        { tag: 'link', attrs: { rel: 'icon', type: 'image/x-icon', href: '/favicon.ico' } },
        { tag: 'link', attrs: { rel: 'icon', type: 'image/png', sizes: '16x16', href: '/favicon-16x16.png' } },
        { tag: 'link', attrs: { rel: 'icon', type: 'image/png', sizes: '32x32', href: '/favicon-32x32.png' } },
        { tag: 'link', attrs: { rel: 'apple-touch-icon', sizes: '180x180', href: '/apple-touch-icon.png' } },
        { tag: 'link', attrs: { rel: 'manifest', href: '/site.webmanifest' } },
        { tag: 'meta', attrs: { name: 'theme-color', content: '#1a1b26' } },
        // ── Open Graph ────────────────────────────────────────────────────────
        { tag: 'meta', attrs: { property: 'og:type', content: 'website' } },
        { tag: 'meta', attrs: { property: 'og:site_name', content: 'Tally' } },
        { tag: 'meta', attrs: { property: 'og:title', content: TITLE } },
        { tag: 'meta', attrs: { property: 'og:description', content: DESCRIPTION } },
        { tag: 'meta', attrs: { property: 'og:image', content: `${SITE}/og-image.png` } },
        { tag: 'meta', attrs: { property: 'og:image:width', content: '1200' } },
        { tag: 'meta', attrs: { property: 'og:image:height', content: '630' } },
        { tag: 'meta', attrs: { property: 'og:image:alt', content: 'Tally — plain-text accounting TUI' } },
        { tag: 'meta', attrs: { property: 'og:url', content: SITE } },
        { tag: 'meta', attrs: { property: 'og:locale', content: 'en_US' } },
        // ── Twitter / X Card ──────────────────────────────────────────────────
        { tag: 'meta', attrs: { name: 'twitter:card', content: 'summary_large_image' } },
        { tag: 'meta', attrs: { name: 'twitter:title', content: TITLE } },
        { tag: 'meta', attrs: { name: 'twitter:description', content: DESCRIPTION } },
        { tag: 'meta', attrs: { name: 'twitter:image', content: `${SITE}/twitter-card.png` } },
        { tag: 'meta', attrs: { name: 'twitter:image:alt', content: 'Tally — plain-text accounting TUI' } },
        // ── General SEO ───────────────────────────────────────────────────────
        { tag: 'meta', attrs: { name: 'description', content: DESCRIPTION } },
        { tag: 'meta', attrs: { name: 'keywords', content: 'accounting,plain-text accounting,double-entry bookkeeping,terminal,TUI,ledger,hledger,personal finance,Rust,CLI' } },
        { tag: 'meta', attrs: { name: 'author', content: 'Murtaza Patel' } },
        { tag: 'meta', attrs: { name: 'robots', content: 'index, follow' } },
        { tag: 'meta', attrs: { name: 'googlebot', content: 'index, follow' } },
        // ── Structured data ───────────────────────────────────────────────────
        { tag: 'script', attrs: { type: 'application/ld+json' }, content: jsonLd },
      ],
      sidebar: [
        {
          label: 'Getting Started',
          items: [
            { label: 'Introduction', slug: 'index' },
            { label: 'Installation', slug: 'install' },
          ],
        },
        {
          label: 'Reference',
          items: [
            { label: 'Journal Format', slug: 'format' },
            { label: 'Commands', slug: 'commands' },
            { label: 'Keybindings', slug: 'keybindings' },
            { label: 'Configuration', slug: 'config' },
          ],
        },
      ],
      customCss: [],
    }),
  ],
});
