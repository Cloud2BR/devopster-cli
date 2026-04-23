// Fetches GitHub Traffic API data, maintains metrics.json, and updates
// <!-- START BADGE --> / <!-- END BADGE --> blocks in all .md files and
// in docs/index.html.
//
// Required env vars:
//   TRAFFIC_TOKEN  – PAT with repo scope (read:traffic)
//   REPO           – owner/repo  e.g. "Cloud2BR/docs-foundry"

const fs   = require('fs');
const path = require('path');

const REPO         = process.env.REPO;
const GITHUB_TOKEN = process.env.TRAFFIC_TOKEN;
const METRICS_FILE = 'metrics.json';

if (!GITHUB_TOKEN || !REPO) {
  console.error('Error: TRAFFIC_TOKEN and REPO environment variables must be set.');
  process.exit(1);
}

if (typeof fetch !== 'function') {
  console.error('Error: global fetch is not available. Use Node.js 20 or later.');
  process.exit(1);
}

// ── Traffic API ──────────────────────────────────────────────────────────────

async function getLast14DaysTraffic() {
  const response = await fetch(`https://api.github.com/repos/${REPO}/traffic/views`, {
    headers: {
      Accept:        'application/vnd.github+json',
      Authorization: `Bearer ${GITHUB_TOKEN}`,
      'User-Agent':  'visitor-counter'
    }
  });

  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(
      `Failed to fetch traffic data: ${response.status} ${response.statusText}\n${errorText}`
    );
  }

  const data = await response.json();
  return data.views.map((item) => ({
    date:    item.timestamp.slice(0, 10),
    count:   item.count,
    uniques: item.uniques
  }));
}

// ── metrics.json ─────────────────────────────────────────────────────────────

function readMetrics() {
  if (!fs.existsSync(METRICS_FILE)) return [];
  try {
    const parsed = JSON.parse(fs.readFileSync(METRICS_FILE, 'utf-8'));
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    console.error('metrics.json is not valid JSON. Starting fresh.');
    return [];
  }
}

function writeMetrics(metrics) {
  fs.writeFileSync(METRICS_FILE, JSON.stringify(metrics, null, 2));
  console.log(`metrics.json updated with ${metrics.length} days`);
}

function mergeMetrics(existing, fetched) {
  const byDate = new Map();
  for (const entry of existing) byDate.set(entry.date, entry);
  // Fetched data wins (it has the authoritative numbers for its window)
  for (const entry of fetched)  byDate.set(entry.date, entry);
  return Array.from(byDate.values()).sort((a, b) => a.date.localeCompare(b.date));
}

function calculateTotalViews(metrics) {
  return metrics.reduce((sum, entry) => sum + entry.count, 0);
}

// ── Badge update — Markdown files ────────────────────────────────────────────

function findMarkdownFiles(dir) {
  let results = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      // Skip hidden dirs, node_modules, release, build outputs
      if (['node_modules', 'release', '.git'].includes(entry.name)) continue;
      results = results.concat(findMarkdownFiles(fullPath));
    } else if (entry.isFile() && entry.name.endsWith('.md')) {
      results.push(fullPath);
    }
  }
  return results;
}

function updateMarkdownBadges(totalViews) {
  const refreshDate = new Date().toISOString().split('T')[0];
  const badgeRegex  = /<!-- START BADGE -->[\s\S]*?<!-- END BADGE -->/g;
  const badgeBlock  = `<!-- START BADGE -->
<div align="center">
  <img src="https://img.shields.io/badge/Total%20views-${totalViews}-limegreen" alt="Total views">
  <p>Refresh Date: ${refreshDate}</p>
</div>
<!-- END BADGE -->`;

  for (const file of findMarkdownFiles('.')) {
    const content = fs.readFileSync(file, 'utf-8');
    if (!badgeRegex.test(content)) continue;
    fs.writeFileSync(file, content.replace(badgeRegex, badgeBlock));
    console.log(`Updated badge in ${file}`);
  }
}

// ── Badge update — GitHub Pages (docs/index.html) ────────────────────────────

function updatePagesBadge(totalViews) {
  const pagesFile = path.join('docs', 'index.html');
  if (!fs.existsSync(pagesFile)) return;

  const content = fs.readFileSync(pagesFile, 'utf-8');
  const START = '<!-- START BADGE -->';
  const END   = '<!-- END BADGE -->';
  if (!content.includes(START) || !content.includes(END)) return;

  const badge = `<span class="badge-link"><img src="https://img.shields.io/badge/Total%20views-${totalViews}-limegreen" alt="Total views" loading="lazy" /></span>`;
  const before = content.split(START)[0];
  const after  = content.split(END)[1];
  fs.writeFileSync(pagesFile, before + START + '\n          ' + badge + '\n          ' + END + after);
  console.log(`Updated badge in ${pagesFile}`);
}

// ── Main ─────────────────────────────────────────────────────────────────────

(async () => {
  try {
    const fetched    = await getLast14DaysTraffic();
    const existing   = readMetrics();
    const merged     = mergeMetrics(existing, fetched);
    writeMetrics(merged);

    const totalViews = calculateTotalViews(merged);
    updateMarkdownBadges(totalViews);
    updatePagesBadge(totalViews);
  } catch (error) {
    console.error(error);
    process.exit(1);
  }
})();