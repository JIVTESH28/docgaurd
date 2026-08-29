# 🚀 DocArmor Stats Auto-Updater Setup

This folder contains everything needed to auto-update your GitHub profile README with live DocArmor PyPI download stats.

## Quick Setup

### 1. Copy files to your profile repo

```bash
# Clone your profile repo
git clone https://github.com/JIVTESH28/JIVTESH28.git
cd JIVTESH28

# Copy the workflow
mkdir -p .github/workflows
cp /path/to/profile-readme-update/.github/workflows/update-readme.yml .github/workflows/

# Copy the script
mkdir -p scripts
cp /path/to/profile-readme-update/scripts/update_stats.py scripts/
```

### 2. Add markers to your README.md

Add these two comment lines in your README.md where you want the stats to appear:

```markdown
<!-- DOCARMOR-STATS:START -->
<!-- DOCARMOR-STATS:END -->
```

### 3. Push and test

```bash
git add .
git commit -m "Add DocArmor stats auto-updater"
git push
```

Then go to **Actions** tab → **Update README with DocArmor Stats** → **Run workflow** to test it manually.

## How It Works

- **GitHub Actions** runs daily at 6:00 AM IST (00:30 UTC)
- `scripts/update_stats.py` pulls daily counts from pypistats for both package
  names — `docgaurd` (pre-rename, yanked) and `docarmor` — and writes the
  section between the `DOCARMOR-STATS` markers in README.md
- Changes are auto-committed by the GitHub Actions bot

## What Gets Counted

Headline figures are **real installs**: `pypistats mirrors=false`, meaning
downloads requested by a package manager such as pip. Repeat installs on one
machine and CI runs are included. Services that clone the whole PyPI index
(bandersnatch and similar) are counted separately and reported as automated
requests, since they fetch every release whether or not anyone wants it.

Lifetime totals sum both package names. PyPI records download history per
name and cannot merge them, so the June–August `docgaurd` installs would
otherwise be invisible.

## data/download-history.json

Committed on purpose, and the source of truth for the numbers:

- pypistats `/overall` retains only **180 days**. The `docgaurd` series starts
  2026-06-02 and would begin falling out of that window around 2026-11-29.
- pypistats throttles by IP and Actions runners share addresses, so a **429**
  is routine. Each run retries with backoff, then falls back to the stored
  series — the table never blanks to N/A, it just notes the data is carried
  over.

Delete this file and the pre-rename history is gone for good once the
retention window passes.

## No API Keys Needed

`pypistats.org/api` is free, public and unauthenticated. Shields.io badge
values are written in by the script rather than fetched live, which avoids
shields.io rate-limit errors on the profile page.
