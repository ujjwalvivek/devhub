<script lang="ts">
    import MediaFrame from "./MediaFrame.svelte";
    import BoundaryMarks from "./BoundaryMarks.svelte";

    const workflows = [
        {
            label: "Overview",
            verb: "Read",
            title: "Understand the selected project",
            copy: "README preview, raw source, stable project identity, and enough repository context to decide what comes next.",
            image: "/images/gpui/home.png",
            alt: "DevHub project overview and README",
            detail: "Local cache / README / offline media",
        },
        {
            label: "Files",
            verb: "Inspect",
            title: "Read source without opening an editor",
            copy: "Browse local or SSH trees and inspect syntax-aware source. The project terminal appears only when explicitly requested.",
            image: "/images/gpui/explorer.png",
            alt: "DevHub file tree, source viewer, and project terminal",
            detail: "Tree / source / explicit terminal",
        },
        {
            label: "Search",
            verb: "Locate",
            title: "Move from match to context",
            copy: "Search bounded project content, open the matching file, and keep per-project todos beside the work.",
            image: "/images/gpui/search.png",
            alt: "DevHub project search, source match, and todo panel",
            detail: "Bounded results / source / todos",
        },
        {
            label: "Git",
            verb: "Act",
            title: "Complete the everyday Git loop",
            copy: "Status, semantic diffs, stage and unstage, explicit discard, commit, branch switching, Fetch, and Push through system Git.",
            image: "/images/gpui/git.png",
            alt: "DevHub Git changes and semantic diff",
            detail: "System Git / local refresh / explicit remotes",
        },
        {
            label: "History",
            verb: "Trace",
            title: "Follow the repository backward",
            copy: "Paginated history, commit topology, refs, changed-file trees, hashes, links, and per-file commit diffs.",
            image: "/images/gpui/commits.png",
            alt: "DevHub commit graph and changed file detail",
            detail: "25 per page / topology / file diff",
        },
    ];

    const tools = [
        ["list_projects", "cached catalog + metadata"],
        ["project_overview", "README + layout + Git + todos"],
        ["list_tree", "gitignore-aware bounded tree"],
        ["read_file", "line-ranged text read"],
        ["search_content", "bounded full-text search"],
        ["git_status", "branch + upstream + changes"],
        ["git_diff", "unified repository diff"],
        ["git_log", "paged commit history"],
        ["list_todos", "project handoff context"],
    ];

    const handoffs = [
        {
            label: "Project switcher",
            title: "Switch without a permanent catalog",
            copy: "Filter cached local and SSH projects, then return the full width to the selected workspace.",
            image: "/images/gpui/project-switcher.png",
            alt: "DevHub transient project switcher",
        },
        {
            label: "Command palette",
            title: "One path for frequent actions",
            copy: "Keyboard and pointer routes share the same command palette, including git, and appearance controls.",
            image: "/images/gpui/commands-palette.png",
            alt: "DevHub command palette",
        },
        {
            label: "Editor handoff",
            title: "Hand off to the right editor",
            copy: "Zed stays primary. Compatible editors are detected from operating-system metadata and project evidence.",
            image: "/images/gpui/ide-switcher.png",
            alt: "DevHub detected editor launcher",
        },
    ];

    let active = $state(0);
    let paused = $state(false);
    let workflow = $derived(workflows[active]);

    $effect(() => {
        const timer = window.setInterval(() => {
            const lightboxOpen = document.querySelector("dialog.lightbox[open]");
            if (!paused && !lightboxOpen) {
                active = (active + 1) % workflows.length;
            }
        }, 6200);
        return () => window.clearInterval(timer);
    });

    function routeKey(event: KeyboardEvent) {
        if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return;
        event.preventDefault();
        const direction = event.key === "ArrowRight" ? 1 : -1;
        active = (active + direction + workflows.length) % workflows.length;
        document.getElementById(`route-${active}`)?.focus();
    }
</script>

<section class="site-section" id="workflow">
    <div class="section-shell">
        <BoundaryMarks variant="b" />
        <header class="section-header" data-reveal>
            <div>
                <span>Selected project</span>
                <h2>Five jobs. One route through the project.</h2>
            </div>
            <p>
                DevHub stays useful until the work becomes editing. Every view
                keeps the same selected project and safety boundary.
            </p>
        </header>

        <div
            class="route-tabs"
            role="tablist"
            tabindex="0"
            aria-label="Project workflow"
            onkeydown={routeKey}
            onmouseenter={() => (paused = true)}
            onmouseleave={() => (paused = false)}
            onfocusin={() => (paused = true)}
            onfocusout={() => (paused = false)}
            data-reveal
        >
            {#each workflows as item, index}
                <button
                    id={`route-${index}`}
                    type="button"
                    role="tab"
                    aria-selected={active === index}
                    aria-controls="route-panel"
                    tabindex={active === index ? 0 : -1}
                    class:active={active === index}
                    onclick={() => (active = index)}
                >
                    <span>{String(index + 1).padStart(2, "0")}</span>
                    <strong>{item.label}</strong>
                </button>
            {/each}
        </div>

        <div
            class="route-panel"
            id="route-panel"
            role="tabpanel"
            aria-labelledby={`route-${active}`}
            data-reveal
        >
            <div class="route-copy">
                <span>{workflow.verb}</span>
                <h3>{workflow.title}</h3>
                <p>{workflow.copy}</p>
                <b>{workflow.detail}</b>
            </div>
            <figure>
                {#key workflow.label}
                    <MediaFrame
                        src={workflow.image}
                        alt={workflow.alt}
                        label={`DevHub ${workflow.label}`}
                    />
                {/key}
            </figure>
        </div>

        <div class="capability-strip" data-reveal>
            <div><span>Terminal</span><b>One selected-project PTY</b></div>
            <div><span>Todos</span><b>Per-project handoff context</b></div>
            <div><span>Appearance</span><b>System / dark / light</b></div>
            <div><span>Persistence</span><b>Atomic and recoverable</b></div>
        </div>
    </div>
</section>

<section class="site-section" id="mcp">
    <div class="section-shell">
        <BoundaryMarks variant="c" />
        <header class="section-header" data-reveal>
            <div>
                <span>Read-only project intelligence</span>
                <h2>Your agent. DevHub's evidence.</h2>
            </div>
            <p>
                Bring your projects context along with you to ANY MCP-enabled agent. No chat panel or inference runtime.
            </p>
        </header>

        <div class="mcp-grid">
            <div class="tool-ledger" data-reveal>
                <div class="ledger-head">
                    <span>Tool</span><span>Result boundary</span><span>Access</span>
                </div>
                {#each tools as tool}
                    <div class="tool-row">
                        <code>{tool[0]}</code>
                        <span>{tool[1]}</span>
                        <b>Read</b>
                    </div>
                {/each}
            </div>

            <figure class="activity-frame" data-reveal style="--reveal-delay:80ms">
                <MediaFrame
                    src="/images/gpui/activity-log.png"
                    alt="DevHub MCP activity overlay with read-only tool calls"
                    label="DevHub MCP activity"
                    width={1000}
                    height={562}
                />
                <figcaption><strong>MCP activity</strong><span>No in-app AI</span></figcaption>
            </figure>
        </div>
    </div>
</section>

<section class="site-section" id="handoff">
    <div class="section-shell">
        <BoundaryMarks variant="d" />
        <header class="section-header" data-reveal>
            <div>
                <span>Explicit action</span>
                <h2>Small task here. Sustained work there.</h2>
            </div>
            <p>
                The project hub earns its place by staying compact and knowing
                when to get out of the way.
            </p>
        </header>

        <div class="handoff-grid">
            {#each handoffs as item, index}
                <article data-reveal style={`--reveal-delay:${index * 60}ms`}>
                    <figure>
                        <MediaFrame
                            src={item.image}
                            alt={item.alt}
                            label={item.label}
                            width={1000}
                            height={562}
                        />
                    </figure>
                    <div>
                        <div class="handoff-label">
                            <span>{String(index + 1).padStart(2, "0")}</span>
                            <b>{item.label}</b>
                        </div>
                        <h3>{item.title}</h3>
                        <p>{item.copy}</p>
                    </div>
                </article>
            {/each}
        </div>
    </div>
</section>

<style>
    .site-section {
        position: relative;
        z-index: 1;
        padding: 44px 12px 0;
    }

    .section-shell {
        position: relative;
        width: min(100%, var(--site-width));
        margin: 0 auto;
        background: color-mix(in srgb, var(--bg) 95%, transparent);
        border: 1px solid var(--border);
    }

    .section-header {
        min-height: 112px;
        display: grid;
        grid-template-columns: minmax(0, 1.45fr) minmax(280px, 0.55fr);
        align-items: center;
        gap: 38px;
        padding: 18px 24px;
        border-bottom: 1px solid var(--border);
    }

    .section-header span,
    .route-copy > span,
    .route-copy b {
        color: var(--accent);
        font-family: var(--font-mono);
        font-size: 10px;
        font-weight: 650;
        text-transform: uppercase;
    }

    .section-header h2 {
        margin-top: 5px;
        color: var(--text);
        font-family: var(--font-display);
        font-size: 32px;
        font-weight: 700;
        line-height: 1.08;
    }

    .section-header > p {
        color: var(--text-dim);
        font-size: 12px;
        line-height: 1.55;
    }

    .route-tabs {
        display: grid;
        grid-template-columns: repeat(5, 1fr);
        border-bottom: 1px solid var(--border);
    }

    .route-tabs button {
        min-width: 0;
        height: 48px;
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 0 14px;
        color: var(--text-dim);
        background: var(--bg-soft);
        border: 0;
        border-radius: 0;
        cursor: pointer;
        text-align: left;
    }

    .route-tabs button + button {
        border-left: 1px solid var(--border);
    }

    .route-tabs button span {
        color: var(--text-muted);
        font-family: var(--font-mono);
        font-size: 10px;
    }

    .route-tabs button strong {
        overflow: hidden;
        font-size: 12px;
        font-weight: 600;
        text-overflow: ellipsis;
    }

    .route-tabs button:hover {
        color: var(--text);
        background: var(--surface-hover);
    }

    .route-tabs button.active {
        color: var(--accent-ink);
        background: var(--accent);
    }

    .route-tabs button.active span {
        color: inherit;
        opacity: 0.66;
    }

    .route-panel {
        min-height: 430px;
        display: grid;
        grid-template-columns: minmax(230px, 0.34fr) minmax(0, 1fr);
        border-bottom: 1px solid var(--border);
    }

    .route-copy {
        display: flex;
        flex-direction: column;
        justify-content: center;
        padding: 24px;
        border-right: 1px solid var(--border);
    }

    .route-copy h3 {
        margin-top: 10px;
        color: var(--text);
        font-size: 21px;
        font-weight: 650;
        line-height: 1.2;
    }

    .route-copy p {
        margin-top: 9px;
        color: var(--text-dim);
        font-size: 12px;
        line-height: 1.55;
    }

    .route-copy b {
        margin-top: 18px;
        color: var(--text-muted);
        line-height: 1.45;
    }

    .route-panel figure {
        min-width: 0;
        overflow: hidden;
        background: var(--bg-card);
    }

    .route-panel :global(.media-trigger) {
        height: 100%;
        animation: screen-enter 280ms var(--ease-out) both;
    }

    .route-panel :global(.media-canvas) {
        border: 0;
    }

    .capability-strip {
        display: grid;
        grid-template-columns: repeat(4, 1fr);
    }

    .capability-strip > div {
        min-height: 54px;
        display: flex;
        flex-direction: column;
        justify-content: center;
        padding: 0 14px;
    }

    .capability-strip > div + div {
        border-left: 1px solid var(--border);
    }

    .capability-strip span,
    .capability-strip b {
        font-family: var(--font-mono);
        font-size: 10px;
    }

    .capability-strip span {
        color: var(--accent);
    }

    .capability-strip b {
        margin-top: 3px;
        color: var(--text-dim);
        font-weight: 500;
    }

    .mcp-grid {
        display: grid;
        grid-template-columns: minmax(410px, 0.9fr) minmax(0, 1.1fr);
        align-items: stretch;
    }

    .tool-ledger {
        min-width: 0;
        border-right: 1px solid var(--border);
    }

    .ledger-head,
    .tool-row {
        display: grid;
        grid-template-columns: minmax(130px, 0.8fr) minmax(180px, 1.4fr) 52px;
        align-items: center;
        gap: 12px;
        padding: 0 14px;
    }

    .ledger-head {
        height: 34px;
        color: var(--text-muted);
        background: var(--bg-soft);
        border-bottom: 1px solid var(--border);
        font-family: var(--font-mono);
        font-size: 9px;
        text-transform: uppercase;
    }

    .tool-row {
        min-height: 38px;
        border-bottom: 1px solid var(--border);
    }

    .tool-row code {
        overflow: hidden;
        color: var(--text);
        font-size: 11px;
        font-weight: 600;
        text-overflow: ellipsis;
    }

    .tool-row span {
        overflow: hidden;
        color: var(--text-dim);
        font-size: 11px;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .tool-row b {
        color: var(--success);
        font-family: var(--font-mono);
        font-size: 9px;
        font-weight: 650;
        text-transform: uppercase;
    }

    .activity-frame {
        width: 100%;
        min-width: 0;
        min-height: 0;
        align-self: stretch;
        contain: size;
        overflow: hidden;
        display: flex;
        flex-direction: column;
        background: #1b1b1b;
    }

    .activity-frame :global(.media-trigger) {
        min-height: 0;
        flex: 1;
    }

    .activity-frame :global(.media-canvas) {
        border: 0;
    }

    .activity-frame :global(.media-canvas img),
    .handoff-grid :global(.media-canvas img) {
        object-fit: cover;
    }

    .activity-frame figcaption {
        min-height: 38px;
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0 12px;
        color: #858585;
        border-top: 1px solid #2c2c2c;
        font-size: 10px;
    }

    .activity-frame figcaption strong {
        color: #d0d0d0;
        font-family: var(--font-mono);
        font-size: 10px;
    }

    .handoff-grid {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
    }

    .handoff-grid article {
        min-width: 0;
    }

    .handoff-grid article + article {
        border-left: 1px solid var(--border);
    }

    .handoff-grid figure {
        overflow: hidden;
        aspect-ratio: 16 / 9;
        background: var(--bg-card);
        border-bottom: 1px solid var(--border);
    }

    .handoff-grid :global(.media-canvas) {
        border: 0;
    }

    .handoff-grid article > div {
        min-height: 132px;
        padding: 15px;
    }

    .handoff-label {
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .handoff-label span {
        width: 28px;
        height: 24px;
        display: grid;
        flex: 0 0 28px;
        place-items: center;
        color: var(--accent-ink);
        background: var(--accent);
        font-family: var(--font-mono);
        font-size: 10px;
        font-weight: 650;
    }

    .handoff-label b {
        color: var(--accent);
        font-family: var(--font-mono);
        font-size: 10px;
        font-weight: 650;
        text-transform: uppercase;
    }

    .handoff-grid h3 {
        margin-top: 8px;
        color: var(--text);
        font-size: 15px;
        font-weight: 650;
    }

    .handoff-grid p {
        margin-top: 6px;
        color: var(--text-dim);
        font-size: 11px;
        line-height: 1.5;
    }

    @keyframes screen-enter {
        from { opacity: 0.45; transform: translateY(5px); }
        to { opacity: 1; transform: translateY(0); }
    }

    @media (max-width: 900px) {
        .section-header {
            grid-template-columns: 1fr;
            gap: 8px;
        }

        .route-panel,
        .mcp-grid {
            grid-template-columns: 1fr;
        }

        .activity-frame {
            height: auto;
            aspect-ratio: 16 / 9;
            contain: none;
        }

        .route-copy,
        .tool-ledger {
            border-right: 0;
            border-bottom: 1px solid var(--border);
        }

        .route-copy {
            min-height: 180px;
        }

        .route-panel figure {
            aspect-ratio: 16 / 9;
        }
    }

    @media (max-width: 680px) {
        .site-section {
            padding: 34px 0 0;
        }

        .section-shell {
            border-inline: 0;
        }

        .route-tabs {
            overflow-x: auto;
            grid-template-columns: repeat(5, minmax(110px, 1fr));
        }

        .capability-strip,
        .handoff-grid {
            grid-template-columns: 1fr 1fr;
        }

        .capability-strip > div:nth-child(3),
        .handoff-grid article:nth-child(3) {
            border-top: 1px solid var(--border);
            border-left: 0;
        }

        .handoff-grid article:nth-child(3) {
            grid-column: 1 / -1;
        }
    }

    @media (max-width: 480px) {
        .section-header {
            padding: 16px 14px;
        }

        .section-header h2 {
            font-size: 27px;
        }

        .route-copy {
            padding: 20px 14px;
        }

        .capability-strip,
        .handoff-grid {
            grid-template-columns: 1fr;
        }

        .capability-strip > div + div,
        .capability-strip > div:nth-child(3),
        .handoff-grid article + article,
        .handoff-grid article:nth-child(3) {
            border-top: 1px solid var(--border);
            border-left: 0;
        }

        .handoff-grid article:nth-child(3) {
            grid-column: auto;
        }

        .ledger-head,
        .tool-row {
            grid-template-columns: minmax(115px, 0.9fr) minmax(0, 1fr) 38px;
            gap: 7px;
            padding-inline: 10px;
        }

        .tool-row span {
            font-size: 10px;
        }
    }
</style>
