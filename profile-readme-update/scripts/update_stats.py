#!/usr/bin/env python3
"""Update README.md with DocArmor install counts.

Headline numbers are real installs: requests made by package managers such as
pip. PyPI tags every download with the installer that requested it, and
services that clone the whole index (bandersnatch and similar) are counted
separately here as automated requests -- they pull each new release once
whether or not anyone wants it. Repeat installs on one machine, and CI runs,
report as pip and so do count as installs.

The project shipped as `docgaurd` (2026-06-02 .. 2026-08-14, now yanked)
before being renamed to `docarmor` (2026-08-02 onwards). PyPI cannot merge
the two names, so lifetime figures are summed from both series here.

Every fetched day is merged into data/download-history.json, which is
committed. That file is the source of truth for two reasons: pypistats
/overall retains only 180 days, and a rate-limited run then still renders
correct numbers instead of blanking them to N/A.
"""

import json
import re
import time
import urllib.error
import urllib.request
from datetime import date, datetime, timedelta, timezone
from pathlib import Path

PACKAGES = ("docgaurd", "docarmor")
CURRENT_PACKAGE = "docarmor"
LEGACY_PACKAGE = "docgaurd"
RENAME_DATE = "2026-08-02"
HISTORY_PATH = Path("data/download-history.json")
USER_AGENT = "docarmor-readme-bot/1.0 (+https://github.com/JIVTESH28/JIVTESH28)"

# mirrors=false counts installs by package managers; mirrors=true additionally
# counts full-index mirroring clients. The difference is automated traffic.
VARIANTS = {"installs": "false", "all_requests": "true"}


def fetch_series(package, mirrors, attempts=4):
    """Daily download counts for one package, retrying on rate limits.

    pypistats throttles by IP and GitHub Actions runners share addresses, so
    a 429 on the first attempt is routine and usually clears within a minute.
    """
    url = f"https://pypistats.org/api/packages/{package}/overall?mirrors={mirrors}"
    req = urllib.request.Request(
        url, headers={"User-Agent": USER_AGENT, "Accept": "application/json"}
    )
    backoff = [5, 15, 30]
    for attempt in range(attempts):
        try:
            with urllib.request.urlopen(req, timeout=30) as resp:
                rows = json.loads(resp.read().decode())["data"]
            series = {}
            for row in rows:
                if row["downloads"]:
                    series[row["date"]] = series.get(row["date"], 0) + row["downloads"]
            return series
        except urllib.error.HTTPError as e:
            print(f"  [warn] {package} mirrors={mirrors}: HTTP {e.code} "
                  f"(attempt {attempt + 1}/{attempts})")
            if e.code not in (429, 500, 502, 503, 504) or attempt == attempts - 1:
                break
        except Exception as e:
            print(f"  [warn] {package} mirrors={mirrors}: {e} "
                  f"(attempt {attempt + 1}/{attempts})")
            if attempt == attempts - 1:
                break
        time.sleep(backoff[min(attempt, len(backoff) - 1)])
    return None


def load_history():
    if HISTORY_PATH.exists():
        return json.loads(HISTORY_PATH.read_text(encoding="utf-8"))
    return {"series": {}}


def save_history(history):
    history["note"] = (
        "installs = pypistats mirrors=false (package-manager downloads). "
        "all_requests = pypistats mirrors=true, which also includes full-index "
        "mirroring clients; the difference is automated traffic. Kept here "
        "because pypistats /overall retains only 180 days. docgaurd is the "
        "pre-rename package name and its series is frozen."
    )
    for pkg, variants in history["series"].items():
        history["series"][pkg] = {k: dict(sorted(v.items())) for k, v in variants.items()}
    HISTORY_PATH.parent.mkdir(parents=True, exist_ok=True)
    HISTORY_PATH.write_text(json.dumps(history, indent=2) + "\n", encoding="utf-8")


def total(history, package, variant):
    return sum(history["series"].get(package, {}).get(variant, {}).values())


def window_sum(series, as_of, days):
    """Sum over the `days`-long window ending at as_of, inclusive."""
    start = as_of - timedelta(days=days - 1)
    return sum(
        n for day, n in series.items() if start <= date.fromisoformat(day) <= as_of
    )


def pretty(d):
    d = date.fromisoformat(d) if isinstance(d, str) else d
    return f"{d.strftime('%b')} {d.day}, {d.year}"


def compute_stats(history, stale):
    current = history["series"].get(CURRENT_PACKAGE, {}).get("installs", {})
    legacy = history["series"].get(LEGACY_PACKAGE, {}).get("installs", {})

    all_days = sorted(
        day
        for variants in history["series"].values()
        for series in variants.values()
        for day in series
    )
    as_of = date.fromisoformat(max(all_days))
    span = (as_of - date.fromisoformat(min(current))).days + 1 if current else 0

    automated = sum(
        total(history, pkg, "all_requests") - total(history, pkg, "installs")
        for pkg in history["series"]
    )
    return {
        "lifetime": sum(current.values()) + sum(legacy.values()),
        "current": sum(current.values()),
        "legacy": sum(legacy.values()),
        "automated": max(automated, 0),
        "last_day": window_sum(current, as_of, 1),
        "last_week": window_sum(current, as_of, 7),
        "last_month": window_sum(current, as_of, 30),
        # While docarmor is younger than 30 days a "Last 30 Days" row merely
        # restates its lifetime figure, so hold it back until it means something.
        "show_month": span > 30,
        "as_of": pretty(as_of),
        "rename": pretty(RENAME_DATE),
        "legacy_first": pretty(min(legacy)) if legacy else "launch",
        "legacy_last": pretty(max(legacy)) if legacy else "the rename",
        "stale": stale,
    }


def build_stats_section(s):
    ist = timezone(timedelta(hours=5, minutes=30))
    now = datetime.now(ist).strftime("%B %d, %Y at %I:%M %p IST")
    num = lambda n: f"{n:,}"
    badge = lambda n: num(n).replace(",", "%2C")

    rows = [
        ("\U0001f3c6", "**Total Installs** — all releases", f"**{num(s['lifetime'])}**"),
        ("\U0001f4e6", f"docarmor — since {s['rename']}", num(s["current"])),
        ("\U0001f5c3️", "docgaurd — before the rename", num(s["legacy"])),
    ]
    if s["show_month"]:
        rows.append(("\U0001f4c5", "Last 30 days", num(s["last_month"])))
    rows += [
        ("\U0001f4c6", "Last 7 days", num(s["last_week"])),
        ("\U0001f550", "Last 24 hours", num(s["last_day"])),
    ]
    table = "\n".join(f"| {i} {label} | {v} |" for i, label, v in rows)
    stale = " · carried over from stored history (pypistats unreachable)" if s["stale"] else ""

    return f"""## 📦 DocArmor — PyPI Installs

<div align="center">

<a href="https://pypi.org/project/docarmor/">
  <img src="https://img.shields.io/pypi/v/docarmor?style=for-the-badge&logo=pypi&logoColor=white&label=PyPI&color=3775A9" alt="PyPI Version"/>
</a>
<a href="https://pypistats.org/packages/docarmor">
  <img src="https://img.shields.io/badge/Total%20Installs-{badge(s['lifetime'])}-00C853?style=for-the-badge&logo=python&logoColor=white" alt="Total Installs"/>
</a>
<a href="https://pypistats.org/packages/docarmor">
  <img src="https://img.shields.io/badge/Last%207%20Days-{badge(s['last_week'])}-AA00FF?style=for-the-badge&logo=download&logoColor=white" alt="Last 7 Days"/>
</a>
<a href="https://pypistats.org/packages/docarmor">
  <img src="https://img.shields.io/badge/Last%2024h-{badge(s['last_day'])}-2979FF?style=for-the-badge&logo=download&logoColor=white" alt="Last 24 Hours"/>
</a>

</div>

<div align="center">

| 📊 Metric | 📈 Installs |
|:---|---:|
{table}

<sub><b>What counts as an install:</b> a download requested by a package manager such as <code>pip</code> — repeat installs and CI runs included. A further <b>{num(s['automated'])}</b> requests came from automated services that clone the entire PyPI index; those are excluded above, since they fetch every release whether or not anyone wants it.</sub>

<sub><b>Two package names, one project:</b> first published as <code>docgaurd</code> on {s['legacy_first']}, then renamed to <code>docarmor</code> on {s['rename']} and republished under a new PyPI account. The retired name kept serving installs until {s['legacy_last']}. PyPI records download history per name and cannot merge the two, so the total above sums both.</sub>

<sub>🤖 Auto-updated {now} via GitHub Actions · data through {s['as_of']}{stale}</sub>

</div>

<div align="center">
  <a href="https://github.com/JIVTESH28/docarmor"><img src="https://img.shields.io/badge/GitHub-Source_Code-181717?style=for-the-badge&logo=github" alt="GitHub"/></a>
  <a href="https://pypi.org/project/docarmor/"><img src="https://img.shields.io/badge/Install-pip_install_docarmor-3775A9?style=for-the-badge&logo=pypi&logoColor=white" alt="Install"/></a>
  <a href="https://pypistats.org/packages/docarmor"><img src="https://img.shields.io/badge/Analytics-pypistats-4B8BBE?style=for-the-badge&logo=python&logoColor=white" alt="Analytics"/></a>
</div>"""


def main():
    print("Fetching DocArmor install stats (automated mirror traffic separated)...")

    history = load_history()
    history.setdefault("series", {})
    stale = True
    for package in PACKAGES:
        stored = history["series"].setdefault(package, {})
        for variant, mirrors in VARIANTS.items():
            fetched = fetch_series(package, mirrors)
            if fetched is None:
                kept = len(stored.get(variant, {}))
                print(f"  [carry] {package}/{variant}: using {kept} stored days")
                continue
            stale = False
            merged = stored.get(variant, {})
            merged.update(fetched)
            stored[variant] = merged
        print(f"  [ok] {package}: installs={total(history, package, 'installs'):,} "
              f"automated={total(history, package, 'all_requests') - total(history, package, 'installs'):,}")

    if not any(v.get("installs") for v in history["series"].values()):
        raise SystemExit("No stats and no stored history — refusing to write README.")

    save_history(history)
    s = compute_stats(history, stale)
    print(f"  lifetime={s['lifetime']:,} (docarmor {s['current']:,} + docgaurd {s['legacy']:,}), "
          f"7d={s['last_week']:,}, 24h={s['last_day']:,}, automated={s['automated']:,}")

    readme = Path("README.md")
    content = readme.read_text(encoding="utf-8")
    start, end = "<!-- DOCARMOR-STATS:START -->", "<!-- DOCARMOR-STATS:END -->"
    block = f"{start}\n{build_stats_section(s)}\n{end}"
    if start in content and end in content:
        content = re.sub(
            re.escape(start) + r".*?" + re.escape(end), lambda _: block, content, flags=re.DOTALL
        )
        print("  [ok] stats section updated")
    else:
        content = content.rstrip() + "\n\n" + block + "\n"
        print("  [ok] stats section appended (markers were missing)")
    readme.write_text(content, encoding="utf-8")
    print("README.md updated.")


if __name__ == "__main__":
    main()
