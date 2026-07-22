<script lang="ts">
    let {
        src,
        alt,
        label = "Product view",
        width = 1920,
        height = 1080,
    }: {
        src: string;
        alt: string;
        label?: string;
        width?: number;
        height?: number;
    } = $props();

    let dialog: HTMLDialogElement;
    let naturalWidth = width;
    let naturalHeight = height;
    let panelWidth = $state(width + 34);

    function updatePanelWidth() {
        if (typeof window === "undefined") return;

        const viewportGap = window.innerWidth <= 640 ? 0 : 48;
        const availableImageHeight = Math.max(
            160,
            window.innerHeight - viewportGap - 44 - 34,
        );
        const heightLimitedWidth =
            availableImageHeight * (naturalWidth / naturalHeight);

        panelWidth = Math.min(
            naturalWidth + 34,
            window.innerWidth - viewportGap,
            heightLimitedWidth + 34,
        );
    }

    function rememberImageSize(event: Event) {
        const image = event.currentTarget as HTMLImageElement;
        naturalWidth = image.naturalWidth || width;
        naturalHeight = image.naturalHeight || height;
        updatePanelWidth();
    }

    function open() {
        updatePanelWidth();
        dialog?.showModal();
    }

    function close() {
        dialog?.close();
    }
</script>

<svelte:window onresize={updatePanelWidth} />

<button
    class="media-trigger"
    type="button"
    aria-label={`Expand ${label}`}
    title={`Expand ${label}`}
    onclick={open}
>
    <span class="media-canvas">
        <span class="dot-matrix top" aria-hidden="true">
            {#each Array(9) as _, index}<i class:lit={index === 1 || index === 5}></i>{/each}
        </span>
        <img {src} {alt} {width} {height} onload={rememberImageSize} />
        <span class="expand-icon" aria-hidden="true">&#x26F6;</span>
        <span class="dot-matrix bottom" aria-hidden="true">
            {#each Array(9) as _, index}<i class:lit={index === 3 || index === 8}></i>{/each}
        </span>
    </span>
</button>

<dialog
    bind:this={dialog}
    class="lightbox"
    aria-label={label}
    onclick={(event) => {
        if (event.target === event.currentTarget) close();
    }}
>
        <section style:--lightbox-width={`${panelWidth}px`}>
            <header>
                <strong>{label}</strong>
                <button type="button" aria-label="Close expanded image" title="Close" onclick={close}>&times;</button>
            </header>
            <div><img {src} {alt} {width} {height} /></div>
        </section>
</dialog>

<style>
    .media-trigger {
        width: 100%;
        height: 100%;
        display: block;
        padding: 0;
        color: inherit;
        background: transparent;
        border: 0;
        border-radius: 0;
        cursor: zoom-in;
        text-align: left;
    }

    .media-canvas {
        position: relative;
        isolation: isolate;
        width: 100%;
        height: 100%;
        min-height: 0;
        display: grid;
        place-items: center;
        overflow: hidden;
        padding: 24px;
        background: var(--bg-card);
        border: 1px solid var(--border);
    }

    .media-canvas::before {
        position: absolute;
        inset: 0;
        z-index: -1;
        content: "";
        background: var(--text-muted);
        -webkit-mask: url("/images/media-grid.svg") 0 0 / 24px 24px repeat;
        mask: url("/images/media-grid.svg") 0 0 / 24px 24px repeat;
        opacity: 0.16;
    }

    .media-canvas img {
        width: 100%;
        height: 100%;
        min-height: 0;
        object-fit: contain;
        background: #1b1b1b;
        border: 1px solid color-mix(in srgb, var(--text) 18%, transparent);
        box-shadow: 0 12px 30px var(--screen-shadow);
        transition: transform 240ms var(--ease-out), border-color 160ms ease;
    }

    .media-trigger:hover .media-canvas img {
        transform: translateY(-2px);
        border-color: var(--accent);
    }

    .expand-icon {
        position: absolute;
        right: 30px;
        bottom: 30px;
        width: 28px;
        height: 28px;
        display: grid;
        place-items: center;
        color: var(--accent-ink);
        background: var(--accent);
        border: 1px solid var(--accent);
        font-size: 16px;
        line-height: 1;
        opacity: 0;
        transform: translateY(3px);
        transition: opacity 140ms ease, transform 140ms ease;
    }

    .media-trigger:hover .expand-icon,
    .media-trigger:focus-visible .expand-icon {
        opacity: 1;
        transform: translateY(0);
    }

    .dot-matrix {
        position: absolute;
        z-index: 2;
        width: 15px;
        display: grid;
        grid-template-columns: repeat(3, 3px);
        gap: 3px;
    }

    .dot-matrix.top {
        top: 5px;
        left: 6px;
    }

    .dot-matrix.bottom {
        right: 6px;
        bottom: 5px;
    }

    .dot-matrix i {
        width: 3px;
        height: 3px;
        background: var(--text-muted);
    }

    .dot-matrix i.lit {
        background: var(--accent);
    }

    .lightbox[open] {
        position: fixed;
        inset: 0;
        z-index: 300;
        width: 100vw;
        max-width: none;
        height: 100vh;
        max-height: none;
        margin: 0;
        display: grid;
        place-items: center;
        padding: 24px;
        color: var(--text);
        background: transparent;
        border: 0;
        cursor: zoom-out;
        animation: lightbox-in 160ms var(--ease-out) both;
    }

    .lightbox::backdrop {
        background: rgb(0 0 0 / 72%);
        -webkit-backdrop-filter: blur(16px);
        backdrop-filter: blur(16px);
    }

    .lightbox section {
        width: min(var(--lightbox-width), 100%, var(--site-width));
        max-height: calc(100vh - 48px);
        background: var(--bg-card);
        border: 1px solid var(--border-strong);
        box-shadow: 0 24px 64px var(--screen-shadow);
        cursor: default;
    }

    .lightbox header {
        height: 44px;
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding-left: 14px;
        border-bottom: 1px solid var(--border);
    }

    .lightbox header strong {
        color: var(--text);
        font-family: var(--font-mono);
        font-size: 11px;
        font-weight: 600;
    }

    .lightbox header button {
        width: 44px;
        height: 43px;
        display: grid;
        place-items: center;
        color: var(--text);
        background: transparent;
        border: 0;
        border-left: 1px solid var(--border);
        border-radius: 0;
        cursor: pointer;
        font-size: 24px;
    }

    .lightbox header button:hover {
        color: var(--accent-ink);
        background: var(--accent);
    }

    .lightbox section > div {
        display: grid;
        place-items: center;
        max-height: calc(100vh - 94px);
        overflow: auto;
        padding: 16px;
        background: var(--bg-soft);
    }

    .lightbox section > div img {
        width: 100%;
        height: auto;
        max-width: 100%;
        max-height: calc(100vh - 126px);
        object-fit: contain;
        border: 1px solid var(--border);
    }

    @keyframes lightbox-in {
        from { opacity: 0; }
    }

    @media (max-width: 640px) {
        .media-canvas {
            padding: 12px;
        }

        .expand-icon {
            right: 18px;
            bottom: 18px;
            opacity: 1;
        }

        .lightbox {
            padding: 0;
        }

        .lightbox section {
            width: 100%;
            max-height: 100vh;
            border-inline: 0;
        }
    }
</style>
