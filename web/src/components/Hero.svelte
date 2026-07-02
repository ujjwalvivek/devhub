<script lang="ts">
    const echopoint = "https://echopoint.ujjwalvivek.com";

    let badge = $state("");
    let cardStr = $state("");
    let langStr = $state("");

    $effect(() => {
        function update() {
            var s = getComputedStyle(document.documentElement);
            var read = (n: string) =>
                s.getPropertyValue(n).trim().replace("#", "");
            badge = `bg=${read("--bg-card")}&badgeColor=${read("--border")}&textColor=${read("--text")}&border=${read("--border")}&borderWidth=2&rx=0&px=6&py=4`;
            cardStr = `bg=${read("--bg-card")}&border=${read("--border")}&borderWidth=4&rx=0&px=12&py=10&textColor=${read("--text")}&accentColor=${read("--accent")}&lineColor=${read("--border")}&positiveColor=${read("--accent")}&negativeColor=${read("--text-dim")}`;
            langStr = `color1=${read("--accent")}`;
        }
        update();
        var observer = new MutationObserver(update);
        observer.observe(document.documentElement, {
            attributes: true,
            attributeFilter: ["data-theme", "data-mode"],
        });
        return () => observer.disconnect();
    });

    const infoBadges = [{ kind: "health", alt: "health" }];

    let dlUrl = $state("#");
    let dlLabel = $state("download");

    $effect(() => {
        var ua = navigator.userAgent;
        var os = "";
        if (ua.includes("Win")) os = "windows";
        else if (ua.includes("Mac")) os = "macos";
        else if (ua.includes("Linux")) os = "linux";

        dlLabel = os ? "download for " + os : "download";

        fetch(
            "https://echopoint.ujjwalvivek.com/v1/store/github:devhub-gpui:releases",
        )
            .then((r) => r.json())
            .then((data) => {
                const release = Array.isArray(data) ? data[0] : data;
                for (const a of release.assets) {
                    if (os === "linux" && a.name.includes("linux")) {
                        dlUrl = a.browser_download_url;
                        break;
                    } else if (
                        os === "macos" &&
                        a.name.includes("apple-darwin")
                    ) {
                        dlUrl = a.browser_download_url;
                        break;
                    } else if (os === "windows" && a.name.includes("windows")) {
                        dlUrl = a.browser_download_url;
                        break;
                    }
                }
            });
    });

    const slides = [
        { src: "/images/gpui/shot-1.webp", alt: "devhub gpui screenshot 1" },
        { src: "/images/gpui/shot-2.webp", alt: "devhub gpui screenshot 2" },
        { src: "/images/gpui/shot-3.webp", alt: "devhub gpui screenshot 3" },
        { src: "/images/gpui/shot-4.webp", alt: "devhub gpui screenshot 4" },
        { src: "/images/gpui/shot-5.webp", alt: "devhub gpui screenshot 5" },
        {
            src: "/images/legacy/shot-1.webp",
            alt: "devhub legacy screenshot 1",
        },
        {
            src: "/images/legacy/shot-2.webp",
            alt: "devhub legacy screenshot 2",
        },
        {
            src: "/images/legacy/shot-3.webp",
            alt: "devhub legacy screenshot 3",
        },
        {
            src: "/images/legacy/shot-4.webp",
            alt: "devhub legacy screenshot 4",
        },
        {
            src: "/images/legacy/shot-5.webp",
            alt: "devhub legacy screenshot 5",
        },
    ];

    let current = $state(0);
    let paused = false;

    function go(i: number) {
        current = i;
    }
    function next() {
        current = (current + 1) % slides.length;
    }
    function prev() {
        current = (current - 1 + slides.length) % slides.length;
    }
    function pause() {
        paused = true;
    }
    function resume() {
        paused = false;
    }

    $effect(() => {
        const id = setInterval(() => {
            if (!paused) next();
        }, 4000);
        return () => clearInterval(id);
    });
</script>

<section class="hero">
    <div class="hero-box">
        <div class="box-corner tl"></div>
        <div class="box-corner tr"></div>
        <div class="box-corner bl"></div>
        <div class="box-corner br"></div>

        <div class="hero-left">
            <div class="badges">
                {#each infoBadges as b}
                    <img
                        src="{echopoint}/svg/badges/{b.kind}?repo=devhub-gpui&logo=github&{badge}"
                        alt={b.alt}
                        height="24"
                    />
                {/each}
            </div>
            <h1 class="title">Your Project Hub</h1>
            <p class="subtitle">
                Native GPUI desktop app, a Zed-first project hub for scanning,
                indexing, and browsing local and remote SSH software projects.
            </p>
            <div class="actions">
                <a href="/docs" class="btn">LLM Rationale</a>
                <a href="#downloads" class="btn">download</a>
                <a
                    href="https://github.com/ujjwalvivek/devhub-gpui"
                    class="btn secondary">github</a
                >
            </div>
        </div>

        <div class="hero-right">
            <div class="portrait-card">
                <div class="card-header">
                    <a
                        href="https://github.com/ujjwalvivek/devhub-gpui"
                        class="card-repo">devhub-gpui</a
                    >
                    <span class="card-author">ujjwalvivek/devhub-gpui</span>
                </div>

                <div class="card-badges">
                    <img
                        src="{echopoint}/svg/badges/stars?repo=devhub-gpui&logo=github&{badge}"
                        alt="stars"
                        height="20"
                    />
                    <img
                        src="{echopoint}/svg/badges/updated?repo=devhub-gpui&logo=github&{badge}"
                        alt="updated"
                        height="20"
                    />
                    <img
                        src="{echopoint}/svg/badges/release?repo=devhub-gpui&logo=github&{badge}"
                        alt="release"
                        height="20"
                    />
                    <img
                        src="{echopoint}/svg/badges/docs?repo=devhub-gpui&logo=docs&{badge}"
                        alt="docs"
                        height="20"
                    />
                </div>

                <div class="card-charts">
                    <img
                        src="{echopoint}/svg/commits?repo=devhub-gpui&limit=3&width=280&{cardStr}&egui_width=280"
                        alt="commits"
                        class="card-img"
                    />
                    <img
                        src="{echopoint}/svg/langs?repo=devhub-gpui&limit=6&width=280&height=8&{cardStr}&pctColor=a6a6a6&{langStr}&color2=c0c0c0&color3=969696&color4=6d6d6d&color5=464646&egui_width=280"
                        alt="languages"
                        class="card-img"
                    />
                </div>
            </div>
        </div>
    </div>

    <div class="slider-box">
        <div class="box-corner tl"></div>
        <div class="box-corner tr"></div>
        <div class="box-corner bl"></div>
        <div class="box-corner br"></div>

        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="slider" onmouseenter={pause} onmouseleave={resume}>
            <div class="viewport">
                <span class="legacy-preview"
                    >{current < 5 ? "gpui" : "legacy"}</span
                >
                <div
                    class="track"
                    style="transform: translateX(-{current * 100}%)"
                >
                    {#each slides as s}
                        <img src={s.src} alt={s.alt} class="slide-img" />
                    {/each}
                </div>
                <button class="arrow left" onclick={prev} aria-label="previous"
                    >‹</button
                >
                <button class="arrow right" onclick={next} aria-label="next"
                    >›</button
                >
            </div>
            <div class="controls">
                <div class="dots">
                    {#each slides as _, i}
                        <button
                            class="dot"
                            class:active={current === i}
                            onclick={() => go(i)}
                            aria-label="go to slide {i + 1}"
                        ></button>
                    {/each}
                </div>
                <span class="counter">{current + 1} / {slides.length}</span>
            </div>
        </div>
    </div>
</section>

<style>
    .hero {
        padding: 24px;
        max-width: 1200px;
        margin: 0 auto;
    }
    .hero-box {
        position: relative;
        display: flex;
        gap: 0;
        background: color-mix(in srgb, var(--bg-card) 10%, transparent);
        backdrop-filter: blur(32px);
        -webkit-backdrop-filter: blur(32px);
        border: 2px dashed color-mix(in srgb, var(--border) 50%, transparent);
        padding: 12px;
        margin-bottom: 12px;
    }
    .slider-box {
        position: relative;
        width: 100%;
        max-width: 1200px;
        background: color-mix(in srgb, var(--bg-card) 10%, transparent);
        backdrop-filter: blur(32px);
        -webkit-backdrop-filter: blur(32px);
        border: 2px dashed color-mix(in srgb, var(--border) 50%, transparent);
        padding: 12px;
        margin-bottom: 12px;
    }
    .box-corner {
        position: absolute;
        width: 20px;
        height: 20px;
        border-color: var(--accent);
        border-style: solid;
        border-width: 0;
        opacity: 0.6;
        pointer-events: none;
    }
    .box-corner.tl {
        top: -1px;
        left: -1px;
        border-top-width: 2px;
        border-left-width: 2px;
    }
    .box-corner.tr {
        top: -1px;
        right: -1px;
        border-top-width: 2px;
        border-right-width: 2px;
    }
    .box-corner.bl {
        bottom: -1px;
        left: -1px;
        border-bottom-width: 2px;
        border-left-width: 2px;
    }
    .box-corner.br {
        bottom: -1px;
        right: -1px;
        border-bottom-width: 2px;
        border-right-width: 2px;
    }
    .hero-left {
        flex: 7;
        padding-right: 32px;
        display: flex;
        flex-direction: column;
        justify-content: center;
    }
    .hero-right {
        flex: 3;
        display: flex;
        align-items: center;
        justify-content: center;
    }
    .portrait-card {
        position: relative;
        width: 100%;
        max-width: 320px;
        background: color-mix(in srgb, var(--bg-card) 50%, transparent);
        backdrop-filter: blur(24px);
        -webkit-backdrop-filter: blur(24px);
        border: 1px solid color-mix(in srgb, var(--border) 50%, transparent);
        padding: 16px;
        display: flex;
        flex-direction: column;
        gap: 10px;
    }
    .card-header {
        display: flex;
        flex-direction: column;
        gap: 2px;
    }
    .card-repo {
        font-family: var(--font-mono);
        font-size: 16px;
        font-weight: 500;
        color: var(--text);
        text-decoration: none;
    }
    .card-repo:hover {
        opacity: 0.7;
    }
    .card-author {
        font-size: 11px;
        color: var(--text-muted);
        letter-spacing: 0.3px;
    }
    .card-badges {
        display: flex;
        flex-wrap: wrap;
        gap: 6px;
    }
    .card-charts {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }
    .card-img {
        width: 100%;
        display: block;
    }

    .badges {
        display: flex;
        gap: 8px;
        margin-bottom: 20px;
        flex-wrap: wrap;
    }
    .title {
        font-family: var(--font-mono);
        font-size: 44px;
        font-weight: 800;
        letter-spacing: -1px;
        margin-bottom: 10px;
        line-height: 1.15;
        text-transform: uppercase;
        text-shadow: 0 0 6px var(--accent);
    }
    .subtitle {
        font-size: 14px;
        color: var(--text-dim);
        line-height: 1.75;
        margin-bottom: 20px;
        max-width: 700px;
    }
    .actions {
        display: grid;
        grid-template-columns: repeat(3, minmax(0, 1fr));
        gap: 10px;
        width: min(100%, 520px);
        margin-bottom: 20px;
    }
    .btn {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        min-width: 0;
        min-height: 40px;
        font-family: var(--font-mono);
        font-size: 12px;
        line-height: 1.2;
        padding: 9px 16px;
        border: 1px solid var(--border);
        background: var(--bg-card);
        color: var(--text);
        text-decoration: none;
        text-align: center;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        transition:
            background 0.15s,
            border-color 0.15s,
            color 0.15s;
    }
    .btn:hover {
        background: var(--bg-soft);
        border-color: var(--text-muted);
        opacity: 1;
    }
    .secondary {
        background: transparent;
        color: var(--text-dim);
    }
    .secondary:hover {
        color: var(--text);
    }

    .slider {
        max-width: 1200px;
        margin: 0 auto;
        user-select: none;
    }
    .viewport {
        position: relative;
        overflow: hidden;
        border: 2px solid var(--border);
        background: var(--bg);
        transition: border-color 0.2s;
    }
    .legacy-preview {
        position: absolute;
        top: 8px;
        left: 8px;
        z-index: 5;
        font-family: var(--font-mono);
        font-size: 10px;
        color: var(--text-dim);
        background: color-mix(in srgb, var(--bg) 80%, transparent);
        padding: 3px 8px;
        border: 1px solid var(--border);
        letter-spacing: 0.5px;
        text-transform: uppercase;
    }
    .track {
        display: flex;
        transition: transform 0.5s cubic-bezier(0.4, 0, 0.2, 1);
    }
    .slide-img {
        width: 100%;
        flex-shrink: 0;
        display: block;
    }
    .arrow {
        position: absolute;
        top: 50%;
        transform: translateY(-50%);
        width: 36px;
        height: 36px;
        border: 1px solid var(--border);
        background: color-mix(in srgb, var(--bg-card) 80%, transparent);
        color: var(--text-dim);
        font-size: 22px;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        opacity: 0;
        transition:
            opacity 0.2s,
            background 0.15s,
            color 0.15s;
        z-index: 10;
    }
    .viewport:hover .arrow {
        opacity: 1;
    }
    .arrow:hover {
        background: var(--bg-soft);
        color: var(--text);
        border-color: var(--text-muted);
    }
    .arrow.left {
        left: 8px;
    }
    .arrow.right {
        right: 8px;
    }
    .controls {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 16px;
        margin-top: 16px;
    }
    .dots {
        display: flex;
        gap: 6px;
    }
    .dot {
        width: 8px;
        height: 8px;
        border: 1px solid var(--text-muted);
        background: transparent;
        cursor: pointer;
        padding: 0;
        transition:
            background 0.2s,
            border-color 0.2s;
    }
    .dot.active {
        background: var(--accent);
        border-color: var(--accent);
    }
    .dot:hover {
        border-color: var(--text);
    }
    .counter {
        font-family: var(--font-mono);
        font-size: 11px;
        color: var(--text-muted);
        letter-spacing: 0.5px;
    }
    @media (max-width: 1000px) {
        .hero-left {
            flex: 6;
        }
        .hero-right {
            flex: 4;
            padding-right: 0;
        }
    }
    @media (max-width: 850px) {
        .controls {
            flex-direction: column;
            gap: 8px;
        }
        .hero-box {
            flex-direction: column;
            margin-bottom: 32px;
        }
        .hero-left {
            padding-right: 0;
        }
        .hero-right {
            width: 100%;
            justify-content: stretch;
        }
        .actions {
            grid-template-columns: repeat(3, minmax(0, 1fr));
            width: 100%;
            gap: 8px;
        }
        .btn {
            padding: 10px 8px;
        }
        .title {
            text-shadow: 0 0 2px var(--accent);
        }
        .hero-right {
            width: 100%;
        }
    }
    @media (max-width: 600px) {
        .actions {
            grid-template-columns: repeat(2, minmax(0, 1fr));
            gap: 8px;
        }
    }
    @media (max-width: 430px) {
        .title {
            font-size: 36px;
        }
        .actions {
            width: 100%;
            grid-template-columns: 1fr;
        }
        .btn {
            width: 100%;
        }
    }
</style>
