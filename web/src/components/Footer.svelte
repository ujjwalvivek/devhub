<footer>
    <div class="footer-grid">
        <span class="footer-symbol" aria-hidden="true">
            {#each Array(12) as _, index}<i class:lit={index === 2 || index === 7 || index === 11}></i>{/each}
        </span>
        <svg class="wordmark" viewBox="0 0 900 230" role="img" aria-label="DevHub">
            <text class="wordmark-type" x="0" y="216" textLength="890" lengthAdjust="spacingAndGlyphs"><tspan class="wordmark-dev">DEV</tspan><tspan class="wordmark-hub">HUB</tspan></text>
        </svg>
        <div class="identity">
            <div>
                <strong>Build</strong>
                <p>Rust + GPUI</p>
                <p>GPL-3.0</p>
                <p>Local first</p>
            </div>
        </div>

        <nav class="product-links" aria-label="Product">
            <b>Product</b>
            <a href="/#workflow">Workflow</a>
            <a href="/#mcp">MCP</a>
            <a href="/#download">Download</a>
        </nav>

        <nav class="project-links" aria-label="Project">
            <b>Project</b>
            <a href="https://github.com/ujjwalvivek/devhub-gpui">Source</a>
            <a href="https://github.com/ujjwalvivek/devhub-gpui/releases/tag/v2.1.4">Latest Release</a>
        </nav>

        <a class="release-cell" href="https://github.com/ujjwalvivek/devhub-gpui/releases/tag/v2.1.4">
            <span>Current release</span>
            <strong>2.1.4</strong>
            <small>Notes and checksums</small>
        </a>

        <div class="signal-panel" aria-hidden="true"></div>

        <div class="build">
            <span class="mark" aria-hidden="true"></span>
        </div>
    </div>

    <div class="footer-rule">
        <span>DevHub</span>
        <span>Local-First</span>
        <span>2026</span>
    </div>

</footer>

<style>
    footer {
        position: relative;
        z-index: 1;
        overflow: hidden;
        background: var(--bg-soft);
        border-top: 1px solid var(--border);
    }

    .footer-grid,
    .footer-rule {
        width: min(calc(100% - 24px), var(--site-width));
        margin: 0 auto;
    }

    .footer-grid {
        position: relative;
        overflow: hidden;
        min-height: 212px;
        display: grid;
        grid-template-columns: repeat(12, 1fr);
        grid-template-rows: 1fr 1fr;
        border-inline: 1px solid var(--border);
    }

    .footer-grid::before {
        position: absolute;
        inset: 0;
        z-index: 0;
        content: "";
        background: var(--text-muted);
        mask: url("images/media-grid.svg") 0 0 / 24px 24px repeat;
        opacity: 0.035;
    }

    .identity,
    .product-links,
    .project-links,
    .release-cell,
    .build {
        position: relative;
        z-index: 1;
        padding: 18px;
    }

    .identity {
        grid-column: 1 / 5;
        grid-row: 1 / 3;
    }

    .product-links {
        grid-column: 5 / 7;
        grid-row: 1 / 3;
    }

    .project-links {
        grid-column: 7 / 9;
        grid-row: 1;
        border-bottom: 1px solid var(--border);
    }

    .release-cell {
        grid-column: 7 / 9;
        grid-row: 2;
    }

    .signal-panel {
        position: relative;
        z-index: 2;
        grid-column: 9 / 10;
        grid-row: 1 / 3;
        overflow: hidden;
        color: var(--accent-ink);
        background-color: var(--accent);
    }

    .signal-panel::before {
        position: absolute;
        inset: -72px 0;
        content: "";
        background-image: repeating-linear-gradient(
            135deg,
            var(--accent-ink) 0,
            var(--accent-ink) 16px,
            transparent 16px,
            transparent 32px
        );
        will-change: transform;
        animation: footer-signal 9s linear infinite;
    }

    @keyframes footer-signal {
        to { transform: translateY(45.255px); }
    }

    @media (prefers-reduced-motion: reduce) {
        .signal-panel::before {
            animation: none;
        }
    }

    .build {
        grid-column: 10 / 13;
        grid-row: 1 / 3;
    }

    .product-links,
    .project-links,
    .release-cell,
    .signal-panel,
    .build {
        border-left: 1px solid var(--border);
    }

    .identity {
        display: flex;
        align-items: flex-start;
        gap: 11px;
    }

    .identity strong,
    .footer-grid b,
    .build span,
    .footer-rule {
        font-family: var(--font-mono);
    }

    .identity strong {
        color: var(--text);
        font-size: 13px;
        cursor: default;
    }

    .identity p {
        margin-top: 4px;
        color: var(--text-muted);
        font-size: 11px;
        cursor: default;
    }

    .footer-grid nav,
    .build {
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        gap: 6px;
    }

    .footer-grid b {
        margin-bottom: 3px;
        color: var(--accent);
        font-size: 10px;
        font-weight: 650;
        text-transform: uppercase;
        cursor: default;
    }

    .footer-grid a,
    .build span {
        color: var(--text-dim);
        font-size: 11px;
    }

    .footer-grid a:hover {
        color: var(--text);
        text-decoration: underline;
        text-underline-offset: 3px;
    }

    .release-cell {
        display: flex;
        flex-direction: column;
        justify-content: center;
        color: var(--text-dim);
        font-family: var(--font-mono);
        transition: background 140ms ease, color 140ms ease;
    }

    .release-cell:hover {
        color: var(--accent-ink) !important;
        background: var(--accent);
        text-decoration: none;
    }

    .release-cell span,
    .release-cell small {
        font-size: 9.5px;
        text-transform: uppercase;
    }

    .release-cell strong {
        margin-block: 2px;
        color: var(--text);
        font-size: 18px;
        line-height: 1;
    }

    .release-cell:hover strong {
        color: inherit;
    }

    .build {
        align-items: center;
        justify-content: center;
    }

    .mark {
        width: 210px;
        height: 210px;
        flex: 0 0 210px;
        background: var(--accent);
        mask: url("images/devhub-mark.svg") center / contain no-repeat;
    }

    .build .mark {
        aspect-ratio: 1;
        flex: 0 0 auto;
    }

    .build span {
        font-size: 10px;
    }

    .footer-rule {
        height: 34px;
        display: grid;
        grid-template-columns: 1fr auto 1fr;
        align-items: center;
        padding: 0 10px;
        color: var(--text-muted);
        border: 1px solid var(--border);
        font-size: 9.5px;
        font-weight: 700;
    }

    .footer-rule span:last-child {
        justify-self: end;
    }

    .wordmark {
        position: absolute;
        left: 14px;
        bottom: -32px;
        z-index: 0;
        display: block;
        width: calc(66.666% - 28px);
        height: 194px;
        color: var(--accent);
        pointer-events: none;
    }

    .wordmark-type {
        fill: currentColor;
        font-family: var(--font-wordmark);
        font-size: 228px;
        font-stretch: condensed;
        font-weight: 900;
        letter-spacing: 0;
        stroke: #fff;
        stroke-width: 4px;
        stroke-opacity: 1;
        paint-order: stroke fill;
        vector-effect: non-scaling-stroke;
    }

    .wordmark-dev {
        opacity: 0.1;
        fill-opacity: 0.1;
    }

    .wordmark-hub {
        opacity: 0.1;
        /*fill-opacity: 0.2;*/
    }

    .footer-symbol {
        position: absolute;
        top: 10px;
        right: 12px;
        z-index: 2;
        width: 21px;
        display: grid;
        grid-template-columns: repeat(4, 3px);
        gap: 3px;
    }

    .footer-symbol i {
        width: 3px;
        height: 3px;
        background: var(--text-muted);
    }

    .footer-symbol i.lit {
        background: var(--accent);
    }

    @media (max-width: 700px) {
        .footer-grid {
            min-height: 0;
            grid-template-columns: 1fr 1fr 64px;
            grid-template-rows: auto auto auto;
        }

        .identity { grid-column: 1 / 3; grid-row: 1; min-height: 100px; }
        .product-links { grid-column: 1; grid-row: 2; border-top: 1px solid var(--border); border-left: 0; }
        .project-links { grid-column: 2; grid-row: 2; border-top: 1px solid var(--border); }
        .release-cell { grid-column: 1 / 3; grid-row: 3; min-height: 76px; border-top: 1px solid var(--border); border-left: 0; }
        .signal-panel { grid-column: 3; grid-row: 1 / 4; }
        .build { display: none; }

        .wordmark {
            bottom: -28px;
            left: 8px;
            width: calc(100% - 80px);
            height: 146px;
        }
    }

    @media (max-width: 450px) {
        .footer-grid,
        .footer-rule {
            width: 100%;
        }

        .footer-rule span:nth-child(2) {
            display: none;
        }

        .wordmark {
            bottom: -16px;
            width: calc(100% - 66px);
            height: 100px;
        }

        .footer-grid {
            grid-template-columns: 1fr 54px;
        }

        .identity { grid-column: 1; }
        .product-links { grid-column: 1; grid-row: 2; }
        .project-links { grid-column: 1; grid-row: 3; border-left: 0; }
        .release-cell { grid-column: 1; grid-row: 4; }
        .signal-panel { grid-column: 2; grid-row: 1 / 5; }
    }
</style>
