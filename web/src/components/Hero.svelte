<script lang="ts">
    import { onMount } from "svelte";
    import MediaFrame from "./MediaFrame.svelte";
    import BoundaryMarks from "./BoundaryMarks.svelte";
    import PatternDoodler from "./PatternDoodler.svelte";

    const routeSteps = [
        "Find",
        "Understand",
        "Act",
        "Hand off",
        "Portable KB"
    ];

    const releasesUrl = "https://github.com/ujjwalvivek/devhub-gpui/releases";
    const feedUrl =
        "https://echopoint.ujjwalvivek.com/v1/store/github:devhub-gpui:releases";

    let releaseTag = $state("latest");
    let releaseUrl = $state(releasesUrl);
    let downloadPlatform = $state<string | null>(null);
    let version = $derived(releaseTag.replace(/^v/i, ""));
    let downloadLabel = $derived(
        downloadPlatform ? `Download for ${downloadPlatform}` : "Download latest",
    );
    let buildVersion = $derived(
        releaseTag === "latest"
            ? "latest"
            : releaseTag.startsWith("v")
              ? releaseTag
              : `v${releaseTag}`,
    );

    $effect(() => {
        const controller = new AbortController();

        fetch(feedUrl, { signal: controller.signal })
            .then((response) => {
                if (!response.ok) throw new Error("release lookup failed");
                return response.json();
            })
            .then((data) => {
                const release = Array.isArray(data) ? data[0] : data;
                if (!release?.tag_name) return;
                releaseTag = String(release.tag_name);
                releaseUrl = String(release.html_url || releasesUrl);
            })
            .catch((error) => {
                if (error.name !== "AbortError") releaseUrl = releasesUrl;
            });

        return () => controller.abort();
    });

    onMount(() => {
        const browserNavigator = navigator as Navigator & {
            userAgentData?: { platform?: string };
        };
        const platform = [
            browserNavigator.userAgentData?.platform,
            browserNavigator.platform,
            browserNavigator.userAgent,
        ]
            .filter(Boolean)
            .join(" ");

        if (/windows|win32|win64/i.test(platform)) {
            downloadPlatform = "Windows";
        } else if (/macos|macintosh|macintel/i.test(platform)) {
            downloadPlatform = "macOS";
        } else if (/linux|x11/i.test(platform) && !/android/i.test(platform)) {
            downloadPlatform = "Linux";
        }
    });
</script>

<div class="release-strip">
    <a href={releaseUrl}>
        <span class="status-dot" aria-hidden="true"></span>
        <strong>DevHub {version}</strong>
        <span>Intelligent MCP now available to download</span>
        <b>Release notes</b>
    </a>
</div>

<section class="hero" id="product">
    <BoundaryMarks variant="a" />
    <div class="hero-grid">
        <div class="hero-copy" data-reveal>
            <PatternDoodler />
            <div class="hero-content">
                <p class="kicker">Local-first | Zed-first | No-AI</p>
                <h1 data-title="DEVHUB">the only Hub you need</h1>
                <p class="lede">
                    Find the right repository. Read what matters. Finish the small
                    Git or terminal task. Hand sustained work to Zed.
                </p>
                <div class="hero-actions">
                    <a class="primary" href="#download" aria-keyshortcuts="d">
                        <span>{downloadLabel}</span><kbd>D</kbd>
                    </a>
                    <a href="https://github.com/ujjwalvivek/devhub-gpui" aria-keyshortcuts="c">
                        <span>Clone source</span><kbd>C</kbd>
                    </a>
                </div>
                <p class="platforms">Available on Windows, Linux, and macOS</p>
            </div>
        </div>

        <dl class="build-board" data-reveal style="--reveal-delay: 80ms">
            <div>
                <dt>Build</dt>
                <dd>{buildVersion}</dd>
            </div>
            <div>
                <dt>Projects</dt>
                <dd>Local + SSH</dd>
            </div>
            <div>
                <dt>Network</dt>
                <dd>On command</dd>
            </div>
            <div>
                <dt>MCP</dt>
                <dd>HTTP + STDIO</dd>
            </div>
        </dl>
    </div>

    <div class="principles" data-reveal style="--reveal-delay: 100ms">
        <article>
            <span>01</span>
            <div><h2>The right project</h2><p>One cached catalog across local roots and SSH hosts.</p></div>
        </article>
        <article>
            <span>02</span>
            <div><h2>Enough context</h2><p>README, files, search, Git, history, todos, and bounded MCP.</p></div>
        </article>
        <article>
            <span>03</span>
            <div><h2>A deliberate handoff</h2><p>Zed first, with detected editors as a second path.</p></div>
        </article>
    </div>

    <figure class="product-frame" data-reveal style="--reveal-delay: 120ms">
        <MediaFrame
            src="/images/gpui/home.png"
            alt={`DevHub ${version} displaying a selected project's README and context`}
            label="DevHub project overview"
        />
        <figcaption>
            <strong>Take a peek inside!</strong>
            <span>README, repository context, and selected-project identity</span>
        </figcaption>
    </figure>

    <ol class="route-rail" aria-label="DevHub project route">
        {#each routeSteps as step, index}
            <li><span>{String(index + 1).padStart(2, "0")}</span>{step}</li>
        {/each}
    </ol>
</section>

<style>
    .release-strip {
        position: relative;
        z-index: 2;
        height: var(--release-height);
        margin-top: var(--header-height);
        background: var(--accent);
        color: var(--accent-ink);
        border-bottom: 1px solid var(--border);
    }

    .release-strip a {
        width: min(100%, var(--site-width));
        height: 100%;
        margin: 0 auto;
        padding: 0 16px;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 10px;
        font-size: 11px;
    }

    .release-strip strong,
    .release-strip b {
        font-family: var(--font-mono);
        font-size: 10px;
        font-weight: 650;
    }

    .release-strip b {
        margin-left: 4px;
        text-decoration: underline;
        text-underline-offset: 3px;
    }

    .status-dot {
        width: 6px;
        height: 6px;
        background: var(--accent-ink);
        animation: status-pulse 2.4s steps(2, end) infinite;
    }

    .hero {
        position: relative;
        z-index: 1;
        width: min(calc(100% - 24px), var(--site-width));
        margin: 0 auto;
        padding-bottom: 40px;
        background: color-mix(in srgb, var(--bg) 95%, transparent);
        border-inline: 1px solid var(--border);
    }

    .hero-grid {
        min-height: 330px;
        display: grid;
        grid-template-columns: minmax(0, 1fr) 212px;
        border-bottom: 1px solid var(--border);
    }

    .hero-copy {
        position: relative;
        isolation: isolate;
        overflow: hidden;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 38px 28px 34px;
        text-align: center;
    }

    .hero-content {
        position: relative;
        z-index: 1;
        display: flex;
        flex-direction: column;
        align-items: center;
    }

    .kicker,
    .platforms {
        color: var(--text-muted);
        font-family: var(--font-mono);
        font-size: 10px;
        font-weight: 700;
    }

    h1 {
        margin-top: 12px;
        color: var(--text);
        font-family: var(--font-wordmark);
        font-size: clamp(35px, 8vw, 60px);
        font-weight: 800;
        line-height: 0.9;
        letter-spacing: 0;
        text-shadow:
            0 0 14px var(--hero-glow),
            0 0 36px var(--hero-glow);
    }

    .claim {
        margin-top: 12px;
        color: var(--text);
        font-family: var(--font-display);
        font-size: 29px;
        line-height: 1.1;
        font-weight: 700;
    }

    .lede {
        max-width: 500px;
        margin-top: 10px;
        color: var(--text-dim);
        font-size: 13px;
        line-height: 1.55;
    }

    .hero-actions {
        display: flex;
        gap: 6px;
        margin-top: 18px;
    }

    .hero-actions a {
        height: 42px;
        min-width: 158px;
        display: inline-flex;
        align-items: center;
        justify-content: space-between;
        gap: 14px;
        padding: 0 14px;
        color: var(--text);
        background: var(--bg-card);
        border: 1px solid var(--border-strong);
        border-radius: 0;
        font-size: 13px;
        transition: background 120ms ease, border-color 120ms ease;
    }

    .hero-actions a:hover {
        background: var(--surface-hover);
        border-color: var(--accent);
    }

    .hero-actions .primary {
        color: var(--accent-ink);
        background: var(--accent);
        border-color: var(--accent);
        font-weight: 650;
    }

    .hero-actions kbd {
        width: 20px;
        height: 20px;
        display: grid;
        flex: 0 0 auto;
        place-items: center;
        color: inherit;
        border: 1px solid color-mix(in srgb, currentColor 32%, transparent);
        font-family: var(--font-mono);
        font-size: 10px;
        font-weight: 650;
        line-height: 1;
        opacity: 0.82;
    }

    .platforms {
        margin-top: 9px;
    }

    .build-board {
        display: grid;
        grid-template-rows: repeat(4, 1fr);
        color: var(--accent-ink);
        background: var(--accent);
        border-left: 1px solid var(--border);
    }

    .build-board > div {
        display: flex;
        flex-direction: column;
        justify-content: center;
        padding: 0 18px;
    }

    .build-board > div + div {
        border-top: 1px solid color-mix(in srgb, var(--accent-ink) 28%, transparent);
    }

    .build-board dt,
    .build-board dd {
        font-family: var(--font-mono);
    }

    .build-board dt {
        font-size: 10px;
        opacity: 0.68;
    }

    .build-board dd {
        margin-top: 3px;
        font-size: 12px;
        font-weight: 650;
    }

    .principles {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        border-bottom: 1px solid var(--border);
    }

    .principles article {
        min-height: 94px;
        display: grid;
        grid-template-columns: auto 1fr;
        gap: 12px;
        padding: 16px 18px;
    }

    .principles article + article {
        border-left: 1px solid var(--border);
    }

    .principles span {
        width: 24px;
        height: 24px;
        display: grid;
        place-items: center;
        color: var(--accent-ink);
        background: var(--accent);
        font-family: var(--font-mono);
        font-size: 10px;
        font-weight: 650;
    }

    .principles h2 {
        color: var(--text);
        font-size: 14px;
        font-weight: 650;
    }

    .principles p {
        margin-top: 4px;
        color: var(--text-dim);
        font-size: 11px;
        line-height: 1.45;
    }

    .product-frame {
        margin: 32px 28px 0;
        overflow: hidden;
        background: var(--bg-card);
        border-radius: 2px;
    }

    .product-frame :global(.media-trigger) {
        height: auto;
    }

    .product-frame :global(.media-canvas) {
        border-color: var(--border-strong);
    }

    .product-frame figcaption {
        min-height: 38px;
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 16px;
        padding: 0 12px;
        color: #8b8b8b;
        background: #1b1b1b;
        border-top: 1px solid #2c2c2c;
        font-size: 10px;
    }

    .product-frame figcaption strong {
        color: #d0d0d0;
        font-family: var(--font-mono);
        font-size: 10px;
    }

    .route-rail {
        margin: 24px 0 -40px;
        display: grid;
        grid-template-columns: repeat(5, 1fr);
        list-style: none;
        background: var(--bg-soft);
        border-block: 1px solid var(--border);
    }

    .route-rail li {
        min-height: 42px;
        display: flex;
        align-items: center;
        gap: 9px;
        padding: 0 12px;
        color: var(--text-dim);
        font-family: var(--font-mono);
        font-size: 10px;
    }

    .route-rail li + li {
        border-left: 1px solid var(--border);
    }

    .route-rail span {
        color: var(--accent);
        font-weight: 650;
    }

    @keyframes status-pulse {
        50% { opacity: 0.35; }
    }

    @media (max-width: 760px) {
        .hero-grid {
            grid-template-columns: 1fr;
        }

        .build-board {
            grid-template-columns: repeat(4, 1fr);
            grid-template-rows: auto;
            border-top: 1px solid var(--border);
            border-left: 0;
        }

        .build-board > div {
            min-height: 58px;
            padding: 0 10px;
        }

        .build-board > div + div {
            border-top: 0;
            border-left: 1px solid color-mix(in srgb, var(--accent-ink) 28%, transparent);
        }

        .principles {
            grid-template-columns: 1fr;
        }

        .principles article + article {
            border-top: 1px solid var(--border);
            border-left: 0;
        }

        .route-rail {
            overflow-x: auto;
            grid-template-columns: repeat(5, minmax(130px, 1fr));
        }
    }

    @media (max-width: 520px) {
        .release-strip a {
            justify-content: flex-start;
            overflow: hidden;
        }

        .release-strip a > span:nth-of-type(2),
        .release-strip b {
            display: none;
        }

        .hero {
            width: 100%;
            border-inline: 0;
        }

        .hero-copy {
            padding: 30px 14px 28px;
        }

        h1 {
            font-size: 52px;
        }

        .claim {
            font-size: 24px;
        }

        .hero-actions {
            width: 100%;
            display: grid;
            grid-template-columns: 1fr 1fr;
        }

        .hero-actions a {
            min-width: 0;
        }

        .build-board {
            grid-template-columns: 1fr 1fr;
        }

        .build-board > div {
            border-top: 1px solid color-mix(in srgb, var(--accent-ink) 28%, transparent);
        }

        .build-board > div:nth-child(odd) {
            border-left: 0;
        }

        .product-frame {
            margin-inline: 0;
            border-inline: 0;
        }

        .product-frame figcaption span {
            display: none;
        }
    }
</style>
