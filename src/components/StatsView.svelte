<script lang="ts">
  import { onMount } from "svelte";
  import {
    project,
    writingStats,
    refreshWritingStats,
  } from "$lib/stores";
  import { formatNumber } from "$lib/util";
  import ProgressRing from "./ProgressRing.svelte";

  onMount(async () => {
    await refreshWritingStats();
  });

  // Build the last 56 days (8 weeks) of heatmap cells, week-by-week.
  function heatClass(words: number, goal: number, goalMet: boolean): string {
    if (words <= 0) return "heat-0";
    const r = goal > 0 ? words / goal : 0;
    if (goalMet || r >= 1) return "heat-4";
    if (r >= 0.75) return "heat-3";
    if (r >= 0.5) return "heat-2";
    if (r > 0) return "heat-1";
    return "heat-0";
  }

  let weeks = $derived.by(() => {
    const stats = $writingStats;
    const byDate = new Map<string, { words: number; goalMet: boolean }>();
    if (stats) {
      for (const d of stats.daily) {
        byDate.set(d.date, { words: d.wordsWritten, goalMet: d.goalMet });
      }
    }
    const goal = stats?.dailyGoal ?? 500;
    const today = new Date();
    today.setHours(0, 0, 0, 0);
    const cells: { date: string; words: number; goalMet: boolean }[] = [];
    for (let i = 55; i >= 0; i--) {
      const d = new Date(today);
      d.setDate(d.getDate() - i);
      const iso = d.toISOString().split("T")[0];
      const rec = byDate.get(iso);
      cells.push({
        date: iso,
        words: rec?.words ?? 0,
        goalMet: rec?.goalMet ?? false,
      });
    }
    const cols: typeof cells[] = [];
    for (let i = 0; i < cells.length; i += 7) {
      cols.push(cells.slice(i, i + 7));
    }
    return { cols, goal };
  });

  // 30-day bar chart data
  let last30 = $derived.by(() => {
    const stats = $writingStats;
    const goal = stats?.dailyGoal ?? 500;
    const byDate = new Map<string, number>();
    if (stats) {
      for (const d of stats.daily) {
        byDate.set(d.date, d.wordsWritten);
      }
    }
    const today = new Date();
    today.setHours(0, 0, 0, 0);
    const out: { date: string; words: number; goal: number; label: string }[] = [];
    for (let i = 29; i >= 0; i--) {
      const d = new Date(today);
      d.setDate(d.getDate() - i);
      const iso = d.toISOString().split("T")[0];
      out.push({
        date: iso,
        words: byDate.get(iso) ?? 0,
        goal,
        label: d.toLocaleDateString("en-US", { month: "numeric", day: "numeric" }),
      });
    }
    return out;
  });

  let maxBar = $derived(Math.max(...last30.map((d) => Math.max(d.words, d.goal)), 1));
</script>

<div class="stats-view fade-in">
  <header class="header chrome-fade">
    <div>
      <h1>Your progress</h1>
      <p>Small consistent words make a book. Here's the shape of yours.</p>
    </div>
  </header>

  <div class="content">
    {#if $writingStats}
      {@const stats = $writingStats}
      {@const p = $project}
      <div class="stat-cards">
        <div class="stat-card">
          <div class="stat-icon" style="color: var(--warn)">🔥</div>
          <div class="stat-body">
            <div class="stat-label">Current streak</div>
            <div class="stat-value">{stats.currentStreak} <span>days</span></div>
          </div>
        </div>
        <div class="stat-card">
          <div class="stat-icon" style="color: var(--accent)">🏆</div>
          <div class="stat-body">
            <div class="stat-label">Longest streak</div>
            <div class="stat-value">{stats.longestStreak} <span>days</span></div>
          </div>
        </div>
        <div class="stat-card">
          <div class="stat-icon" style="color: var(--fg-0)">✎</div>
          <div class="stat-body">
            <div class="stat-label">Total words</div>
            <div class="stat-value">{formatNumber(stats.totalWords)}</div>
          </div>
        </div>
        <div class="stat-card">
          <div class="stat-icon" style="color: var(--success)">✓</div>
          <div class="stat-body">
            <div class="stat-label">Goals met</div>
            <div class="stat-value">{stats.goalsMetCount} <span>days</span></div>
          </div>
        </div>
      </div>

      <div class="row">
        <!-- Today's ring -->
        <div class="card today-card">
          <ProgressRing value={stats.todayWords} max={stats.dailyGoal} size={120} stroke={6} />
          <div class="today-label">Today</div>
          <div class="today-sub">
            {formatNumber(stats.todayWords)} / {formatNumber(stats.dailyGoal)} words
          </div>
          {#if stats.todayGoalMet}
            <div class="goal-met">✓ Goal met — well done.</div>
          {/if}
        </div>

        <!-- Heatmap -->
        <div class="card heat-card">
          <div class="card-head">
            <div>
              <h3>Last 8 weeks</h3>
              <p class="card-sub">Each square is a day. Brighter = more words.</p>
            </div>
            <div class="legend">
              <span>Less</span>
              {#each [0, 1, 2, 3, 4] as i}
                <div class="heat-{i} heat-cell"></div>
              {/each}
              <span>More</span>
            </div>
          </div>
          <div class="heat-grid">
            {#each weeks.cols as week}
              <div class="heat-col">
                {#each week as day}
                  <div
                    class="heat-cell {heatClass(day.words, weeks.goal, day.goalMet)}"
                    title="{day.date}: {formatNumber(day.words)} words{day.goalMet ? ' — goal met!' : ''}"
                  ></div>
                {/each}
              </div>
            {/each}
          </div>
        </div>
      </div>

      <!-- 30-day bar chart -->
      <div class="card">
        <div class="card-head">
          <div>
            <h3>Words per day, last 30 days</h3>
            <p class="card-sub">Bars show your daily output. Line = your goal.</p>
          </div>
        </div>
        <div class="chart">
          {#each last30 as d}
            <div class="bar-col">
              <div class="bar-track">
                <div class="goal-line" style="bottom: {(d.goal / maxBar) * 100}%"></div>
                <div
                  class="bar"
                  class:met={d.words >= d.goal}
                  style="height: {Math.max(2, (d.words / maxBar) * 100)}%"
                  title="{d.date}: {formatNumber(d.words)} words"
                ></div>
              </div>
            </div>
          {/each}
        </div>
      </div>

      <!-- Chapter breakdown -->
      {#if p && p.chapters.length > 0}
        {@const max = Math.max(...p.chapters.map((c) => c.wordCount), 1)}
        <div class="card">
          <div class="card-head">
            <div>
              <h3>Chapters</h3>
              <p class="card-sub">{p.chapters.length} total · {formatNumber(stats.totalWords)} words</p>
            </div>
          </div>
          <div class="chapters">
            {#each p.chapters as ch, i}
              <div class="ch-row">
                <span class="ch-idx">{(i + 1).toString().padStart(2, "0")}</span>
                <div class="ch-body">
                  <div class="ch-head">
                    <span class="ch-title">{ch.title}</span>
                    <span class="ch-words">{formatNumber(ch.wordCount)} words</span>
                  </div>
                  <div class="ch-bar-bg">
                    <div class="ch-bar" style="width: {(ch.wordCount / max) * 100}%"></div>
                  </div>
                </div>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      <div class="quote">
        <em>"You can't edit a blank page."</em>
        <span>— Jodi Picoult</span>
      </div>
    {:else}
      <div class="loading">Loading…</div>
    {/if}
  </div>
</div>

<style>
  .stats-view {
    flex: 1;
    overflow-y: auto;
    background: var(--bg-2);
  }
  .header {
    position: sticky;
    top: 0;
    z-index: 5;
    padding: 18px 28px;
    background: color-mix(in srgb, var(--bg-2) 92%, transparent);
    backdrop-filter: blur(8px);
    border-bottom: 1px solid var(--border-soft);
  }
  .header h1 {
    margin: 0;
    font-family: var(--serif);
    font-size: 22px;
    font-weight: 500;
    color: var(--fg-0);
  }
  .header p {
    margin: 4px 0 0 0;
    font-size: 12px;
    color: var(--fg-2);
  }
  .content {
    max-width: 900px;
    margin: 0 auto;
    padding: 24px 28px 60px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .stat-cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 12px;
  }
  .stat-card {
    display: flex;
    align-items: center;
    gap: 12px;
    background: var(--bg-3);
    border: 1px solid var(--border-soft);
    border-radius: 10px;
    padding: 14px;
  }
  .stat-icon {
    font-size: 20px;
  }
  .stat-label {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--fg-2);
    font-weight: 600;
  }
  .stat-value {
    font-family: var(--serif);
    font-size: 22px;
    font-weight: 500;
    color: var(--fg-0);
    font-variant-numeric: tabular-nums;
  }
  .stat-value span {
    font-size: 11px;
    color: var(--fg-2);
    font-weight: 400;
  }
  .row {
    display: grid;
    grid-template-columns: 220px 1fr;
    gap: 16px;
  }
  @media (max-width: 720px) {
    .row {
      grid-template-columns: 1fr;
    }
  }
  .card {
    background: var(--bg-3);
    border: 1px solid var(--border-soft);
    border-radius: 12px;
    padding: 18px;
  }
  .today-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: 24px;
  }
  .today-label {
    margin-top: 12px;
    font-family: var(--serif);
    font-size: 16px;
    font-weight: 500;
    color: var(--fg-0);
  }
  .today-sub {
    margin-top: 4px;
    font-size: 12px;
    color: var(--fg-1);
    font-variant-numeric: tabular-nums;
  }
  .goal-met {
    margin-top: 8px;
    font-size: 11px;
    color: var(--success);
    font-weight: 500;
  }
  .card-head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    margin-bottom: 14px;
    gap: 12px;
  }
  .card-head h3 {
    margin: 0;
    font-family: var(--serif);
    font-size: 15px;
    font-weight: 500;
    color: var(--fg-0);
  }
  .card-sub {
    margin: 2px 0 0 0;
    font-size: 11px;
    color: var(--fg-2);
  }
  .legend {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 10px;
    color: var(--fg-2);
  }
  .heat-cell {
    width: 11px;
    height: 11px;
    border-radius: 2px;
  }
  .heat-grid {
    display: flex;
    gap: 3px;
  }
  .heat-col {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .chart {
    display: flex;
    align-items: flex-end;
    gap: 2px;
    height: 160px;
    padding: 0 4px;
  }
  .bar-col {
    flex: 1;
    height: 100%;
    display: flex;
    align-items: flex-end;
  }
  .bar-track {
    position: relative;
    width: 100%;
    height: 100%;
  }
  .bar {
    position: absolute;
    bottom: 0;
    left: 1px;
    right: 1px;
    background: var(--accent);
    border-radius: 2px 2px 0 0;
    transition: height 0.4s ease;
  }
  .bar.met {
    background: var(--success);
  }
  .goal-line {
    position: absolute;
    left: 0;
    right: 0;
    height: 1px;
    background: var(--fg-2);
    opacity: 0.6;
    border-top: 1px dashed var(--fg-2);
    background: transparent;
  }
  .chapters {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .ch-row {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .ch-idx {
    font-size: 10px;
    color: var(--fg-2);
    font-family: var(--mono);
    font-variant-numeric: tabular-nums;
    width: 24px;
    text-align: right;
  }
  .ch-body {
    flex: 1;
    min-width: 0;
  }
  .ch-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 8px;
    margin-bottom: 4px;
  }
  .ch-title {
    font-family: var(--serif);
    font-size: 13px;
    color: var(--fg-0);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .ch-words {
    font-size: 11px;
    color: var(--fg-2);
    font-variant-numeric: tabular-nums;
  }
  .ch-bar-bg {
    height: 4px;
    background: var(--bg-2);
    border-radius: 2px;
    overflow: hidden;
  }
  .ch-bar {
    height: 100%;
    background: var(--accent);
    border-radius: 2px;
  }
  .quote {
    text-align: center;
    padding: 24px 0;
  }
  .quote em {
    font-family: var(--serif);
    font-size: 16px;
    color: var(--fg-1);
    font-style: italic;
  }
  .quote span {
    display: block;
    margin-top: 4px;
    font-size: 11px;
    color: var(--fg-2);
  }
  .loading {
    text-align: center;
    padding: 40px;
    color: var(--fg-2);
  }
</style>
