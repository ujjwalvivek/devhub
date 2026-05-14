<script lang="ts">
    const features = [
        {
            title: "project discovery",
            desc: "Scans local filesystems and remote SSH hosts for build markers i.e., Cargo.toml, package.json, go.mod, and more. Respects .gitignore.",
            icon: "search",
        },
        {
            title: "remote via SSH",
            desc: "Piped find/git/grep over the system ssh binary with BatchMode. Works with any key-based SSH host.",
            icon: "terminal",
        },
        {
            title: "file tree & search",
            desc: "Collapsible directory tree, file preview (512KB local / 200KB remote), full-text search. 500-entry cap.",
            icon: "folder",
        },
        {
            title: "editor handoff",
            desc: "Template-based launcher for VS Code (local + Remote SSH URI), Zed, or any CLI editor. Configured in TOML.",
            icon: "code",
        },
        {
            title: "telemetry badges",
            desc: "SVG badges and cards from echopoint.ujjwalvivek.com for stars, commits, languages, releases, health. Rendered via resvg.",
            icon: "chart",
        },
        {
            title: "five themes",
            desc: "Catppuccin Mocha, Rose Pine Moon, Tokyo Night Storm, Horizon Bold, Monochrome Zero. Auto light variants, follows OS theme.",
            icon: "palette",
        },
    ];

    const icons: Record<string, string> = {
        search: "M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z",
        terminal:
            "M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z",
        folder: "M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z",
        code: "M10 20l5-16m-7 4l-4 4 4 4m8-8l4 4-4 4",
        chart: "M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z",
        palette:
            "M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zm0 0h12a2 2 0 002-2v-4a2 2 0 00-2-2h-2.343M11 7.343l1.657-1.657a2 2 0 012.828 0l2.829 2.829a2 2 0 010 2.828l-8.486 8.485M7 17h.01",
    };
</script>

<section class="features" id="features">
    <div class="container">
        <h2 class="section-title">features</h2>
        <div class="grid">
            {#each features as f}
                <div class="card">
                    <div class="card-icon">
                        <svg
                            width="20"
                            height="20"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="1.5"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                        >
                            <path d={icons[f.icon]} />
                        </svg>
                    </div>
                    <h3 class="card-title">{f.title}</h3>
                    <p class="card-desc">{f.desc}</p>
                </div>
            {/each}
        </div>
    </div>
</section>

<style>
    .features {
        padding: 60px 24px 60px;
        max-width: 1200px;
        margin: 0 auto;
    }
    .section-title {
        font-family: var(--font-mono);
        font-size: 14px;
        font-weight: 500;
        color: var(--text-muted);
        text-transform: uppercase;
        letter-spacing: 1px;
        margin-bottom: 32px;
    }
    .grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
        gap: 16px;
    }
    .card {
        background: color-mix(in srgb, var(--bg-card) 15%, transparent);
        backdrop-filter: blur(24px);
        -webkit-backdrop-filter: blur(24px);
        border: 2px dashed color-mix(in srgb, var(--accent) 50%, transparent);
        padding: 24px;
        display: flex;
        flex-direction: column;
        gap: 10px;
    }
    .card-icon {
        width: 36px;
        height: 36px;
        display: flex;
        align-items: center;
        justify-content: center;
        border: 1.5px solid color-mix(in srgb, var(--border) 50%, transparent);
        color: var(--accent);
    }
    .card-title {
        font-family: var(--font-mono);
        font-size: 14px;
        font-weight: 500;
        color: var(--text);
    }
    .card-desc {
        font-size: 13px;
        color: var(--text-dim);
        line-height: 1.6;
    }
    @media (max-width: 768px) {
        .features {
            padding: 32px 24px;
        }
    }
</style>
