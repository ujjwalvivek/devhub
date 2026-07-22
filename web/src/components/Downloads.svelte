<script lang="ts">
    import BoundaryMarks from "./BoundaryMarks.svelte";

    type Asset = {
        name: string;
        url: string;
        digest: string;
        size: number;
    };

    type PlatformId = "windows" | "linux" | "macos";

    const releasesUrl = "https://github.com/ujjwalvivek/devhub-gpui/releases";
    const feedUrl =
        "https://echopoint.ujjwalvivek.com/v1/store/github:devhub-gpui:releases";
    const platforms: {
        id: PlatformId;
        label: string;
        target: string;
        placeholder: string;
    }[] = [
        {
            id: "windows",
            label: "Windows",
            target: "x64 installer",
            placeholder: "Latest Windows installer",
        },
        {
            id: "linux",
            label: "Linux",
            target: "x64 AppImage",
            placeholder: "Latest Linux AppImage",
        },
        {
            id: "macos",
            label: "macOS",
            target: "Apple silicon DMG",
            placeholder: "Latest macOS disk image",
        },
    ];

    let version = $state("latest");
    let releaseUrl = $state(releasesUrl);
    let loading = $state(true);
    let failed = $state(false);
    let assets = $state<Record<PlatformId, Asset | null>>({
        windows: null,
        linux: null,
        macos: null,
    });
    let copied = $state<Record<string, boolean>>({});

    $effect(() => {
        const controller = new AbortController();
        fetch(feedUrl, { signal: controller.signal })
            .then((response) => {
                if (!response.ok) throw new Error("release lookup failed");
                return response.json();
            })
            .then((data) => {
                const releases = Array.isArray(data) ? data : [data];
                const release = releases[0];
                if (!release) throw new Error("release feed is empty");

                version = String(release.tag_name || "latest");
                releaseUrl = String(release.html_url || releasesUrl);

                const next: Record<PlatformId, Asset | null> = {
                    windows: null,
                    linux: null,
                    macos: null,
                };
                for (const item of release.assets || []) {
                    const name = String(item.name || "");
                    const lower = name.toLowerCase();
                    const asset: Asset = {
                        name,
                        url: item.browser_download_url,
                        digest: String(item.digest || "").replace("sha256:", ""),
                        size: Number(item.size || 0),
                    };
                    if (lower.endsWith(".exe")) next.windows = asset;
                    else if (lower.endsWith(".appimage")) next.linux = asset;
                    else if (lower.endsWith(".dmg")) next.macos = asset;
                }
                assets = next;
            })
            .catch((error) => {
                if (error.name !== "AbortError") failed = true;
            })
            .finally(() => (loading = false));

        return () => controller.abort();
    });

    function size(asset: Asset | null) {
        if (!asset?.size) return loading ? "Fetching" : "Unavailable";
        return `${(asset.size / 1024 / 1024).toFixed(1)} MB`;
    }

    function copyDigest(id: PlatformId, digest: string) {
        if (!digest) return;
        navigator.clipboard.writeText(digest).then(() => {
            copied[id] = true;
            window.setTimeout(() => (copied[id] = false), 1800);
        });
    }
</script>

<section class="download-section" id="download">
    <div class="section-shell">
        <BoundaryMarks variant="e" />
        <header data-reveal>
            <div>
                <span>Native / {version}</span>
                <h2>Get on the Hub</h2>
            </div>
            <p>
                No account, hosted catalog, or telemetry. DevHub stays local and reaches the network only on command.
            </p>
        </header>

        <div class="platform-grid" aria-busy={loading}>
            {#each platforms as platform, index}
                {@const asset = assets[platform.id]}
                <article data-reveal style={`--reveal-delay:${index * 50}ms`}>
                    <div class="platform-head">
                        <span>{String(index + 1).padStart(2, "0")}</span>
                        <h3>{platform.label}</h3>
                        <b>{platform.target}</b>
                    </div>
                    <div class="platform-filename">
                        <svg viewBox="0 0 24 24" aria-hidden="true">
                            {#if platform.id === "windows"}
                                <path d="M3 3h8v8H3zM13 3h8v8h-8zM3 13h8v8H3zM13 13h8v8h-8z" />
                            {:else if platform.id === "linux"}
                                <path d="M14.62 8.35c-.42.28-1.75 1.04-1.95 1.19c-.39.31-.75.29-1.14-.01c-.2-.16-1.53-.92-1.95-1.19c-.48-.31-.45-.7.08-.92c1.64-.69 3.28-.64 4.91.03c.49.21.51.6.05.9m7.22 7.28c-.93-2.09-2.2-3.99-3.84-5.66a4.3 4.3 0 0 1-1.06-1.88c-.1-.33-.17-.67-.24-1.01c-.2-.88-.29-1.78-.7-2.61c-.73-1.58-2-2.4-3.84-2.47c-1.81.05-3.16.81-3.95 2.4c-.21.43-.36.88-.46 1.34c-.17.76-.32 1.55-.5 2.32c-.15.65-.45 1.21-.96 1.71c-1.61 1.57-2.9 3.37-3.88 5.35c-.14.29-.28.58-.37.88c-.19.66.29 1.12.99.96c.44-.09.88-.18 1.3-.31c.41-.15.57-.05.67.35c.65 2.15 2.07 3.66 4.24 4.5c4.12 1.56 8.93-.66 9.97-4.58c.07-.27.17-.37.47-.27c.46.14.93.24 1.4.35c.49.09.85-.16.92-.64c.03-.26-.06-.49-.16-.73" />
                            {:else}
                                <path d="M17.05 20.28c-.98.95-2.05.8-3.08.35c-1.09-.46-2.09-.48-3.24 0c-1.44.62-2.2.44-3.06-.35C2.79 15.25 3.51 7.59 9.05 7.31c1.35.07 2.29.74 3.08.8c1.18-.24 2.31-.93 3.57-.84c1.51.12 2.65.72 3.4 1.8c-3.12 1.87-2.38 5.98.48 7.13c-.57 1.5-1.31 2.99-2.54 4.09zM12.03 7.25c-.15-2.23 1.66-4.07 3.74-4.25c.29 2.58-2.34 4.5-3.74 4.25" />
                            {/if}
                        </svg>
                        <code>{asset?.name || platform.placeholder}</code>
                    </div>
                    <div class="platform-actions">
                        <a href={asset?.url || releaseUrl}>Download <span>{size(asset)}</span></a>
                        <button
                            type="button"
                            disabled={!asset?.digest}
                            onclick={() => copyDigest(platform.id, asset?.digest || "")}
                        >
                            {copied[platform.id] ? "Copied" : "SHA-256"}
                        </button>
                    </div>
                </article>
            {/each}
        </div>

        <div class="release-strip" data-reveal>
            <span>Read-only editor</span>
            <span>MCP Server for your projects</span>
            <a href={releaseUrl}>Checksums and release notes</a>
            <span class:failed>{failed ? "Release feed unavailable" : loading ? "Fetching latest release" : `${version} verified`}</span>
        </div>
    </div>
</section>

<style>
    .download-section {
        position: relative;
        z-index: 1;
        padding: 44px 12px 52px;
    }

    .section-shell {
        position: relative;
        width: min(100%, var(--site-width));
        margin: 0 auto;
        background: color-mix(in srgb, var(--bg) 95%, transparent);
        border: 1px solid var(--border);
    }

    header {
        min-height: 112px;
        display: grid;
        grid-template-columns: minmax(0, 1.45fr) minmax(280px, 0.55fr);
        align-items: center;
        gap: 38px;
        padding: 18px 24px;
        border-bottom: 1px solid var(--border);
    }

    header span {
        color: var(--accent);
        font-family: var(--font-mono);
        font-size: 10px;
        font-weight: 650;
        text-transform: uppercase;
    }

    header h2 {
        margin-top: 5px;
        color: var(--text);
        font-family: var(--font-display);
        font-size: 32px;
        font-weight: 700;
        line-height: 1.08;
    }

    header > p {
        color: var(--text-dim);
        font-size: 12px;
        line-height: 1.55;
    }

    .platform-grid {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
    }

    .platform-grid article {
        min-width: 0;
        padding: 16px;
    }

    .platform-grid article + article {
        border-left: 1px solid var(--border);
    }

    .platform-head {
        display: grid;
        grid-template-columns: 28px 1fr auto;
        align-items: center;
        gap: 9px;
        padding-bottom: 11px;
        border-bottom: 1px solid var(--border);
    }

    .platform-head span {
        width: 28px;
        height: 24px;
        display: grid;
        place-items: center;
        color: var(--accent-ink);
        background: var(--accent);
        font-family: var(--font-mono);
        font-size: 10px;
        font-weight: 650;
    }

    .platform-head b {
        color: var(--text-muted);
        font-family: var(--font-mono);
        font-size: 9.5px;
        font-weight: 700;
        text-transform: uppercase;
    }

    .platform-head h3 {
        color: var(--text);
        font-size: 14px;
        font-weight: 650;
    }

    .platform-filename {
        min-width: 0;
        margin-top: 14px;
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .platform-filename svg {
        width: 16px;
        height: 16px;
        flex: 0 0 16px;
        fill: currentColor;
        color: var(--text-dim);
    }

    .platform-grid code {
        min-width: 0;
        display: block;
        overflow: hidden;
        color: var(--text);
        font-size: 11px;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .platform-actions {
        margin-top: 12px;
        display: grid;
        grid-template-columns: 1fr 72px;
        gap: 5px;
    }

    .platform-actions a,
    .platform-actions button {
        height: 44px;
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0 10px;
        border: 1px solid var(--border-strong);
        border-radius: 0;
        font-family: var(--font-mono);
        font-size: 11px;
    }

    .platform-actions a {
        color: var(--accent-ink);
        background: var(--accent);
        border-color: var(--accent);
        font-weight: 700;
    }

    .platform-actions a span {
        margin-left: 10px;
        opacity: 0.66;
    }

    .platform-actions button {
        color: var(--text-dim);
        background: var(--bg-card);
        cursor: pointer;
    }

    .platform-actions button:hover:not(:disabled) {
        color: var(--text);
        border-color: var(--accent);
    }

    .platform-actions button:disabled {
        cursor: default;
        opacity: 0.42;
    }

    .release-strip {
        min-height: 44px;
        display: grid;
        grid-template-columns: 1fr 1fr 1.3fr 1fr;
        align-items: center;
        color: var(--text-muted);
        background: var(--bg-soft);
        border-top: 1px solid var(--border);
        font-family: var(--font-mono);
        font-size: 9.5px;
        font-weight: 700;
    }

    .release-strip > * {
        height: 100%;
        display: flex;
        align-items: center;
        padding: 0 12px;
    }

    .release-strip > * + * {
        border-left: 1px solid var(--border);
    }

    .release-strip a {
        color: var(--text-dim);
    }

    .release-strip a:hover {
        color: var(--text);
    }

    .release-strip span:last-child {
        color: var(--success);
    }

    .release-strip span.failed {
        color: var(--warning);
    }

    @media (max-width: 920px) {
        header {
            grid-template-columns: 1fr;
            gap: 8px;
        }

        .platform-grid {
            grid-template-columns: 1fr;
        }

        .platform-grid article + article {
            border-top: 1px solid var(--border);
            border-left: 0;
        }

        .release-strip {
            grid-template-columns: 1fr 1fr;
        }

        .release-strip > *:nth-child(3) {
            border-top: 1px solid var(--border);
            border-left: 0;
        }

        .release-strip > *:nth-child(4) {
            border-top: 1px solid var(--border);
        }
    }

    @media (max-width: 680px) {
        .download-section {
            padding: 34px 0 42px;
        }

        .section-shell {
            border-inline: 0;
        }
    }

    @media (max-width: 480px) {
        header {
            padding: 16px 14px;
        }

        header h2 {
            font-size: 27px;
        }

        .platform-grid article {
            padding: 14px;
        }

        .platform-head {
            grid-template-columns: 28px 1fr;
        }

        .platform-head b {
            grid-column: 2;
        }

        .platform-actions,
        .release-strip {
            grid-template-columns: 1fr;
        }

        .release-strip > * + *,
        .release-strip > *:nth-child(3),
        .release-strip > *:nth-child(4) {
            border-top: 1px solid var(--border);
            border-left: 0;
        }

        .release-strip > * {
            min-height: 38px;
        }
    }
</style>
