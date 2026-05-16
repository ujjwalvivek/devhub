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

        fetch("https://api.github.com/repos/ujjwalvivek/devhub/releases/latest")
            .then((r) => r.json())
            .then((data) => {
                for (const a of data.assets) {
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
        { src: "/images/shot-1.webp", alt: "devhub screenshot 1" },
        { src: "/images/shot-2.webp", alt: "devhub screenshot 2" },
        { src: "/images/shot-3.webp", alt: "devhub screenshot 3" },
        { src: "/images/shot-4.webp", alt: "devhub screenshot 4" },
        { src: "/images/shot-5.webp", alt: "devhub screenshot 5" },
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

    let globalScore = $state<number | null>(null);
    let pendingClicks = $state(0);
    let scoreText = $state("...");

    $effect(() => {
        const API_BASE = "https://echopoint.ujjwalvivek.com";
        const wsUrl = "wss://echopoint.ujjwalvivek.com/v1/click";
        if (!window.epClickerSocket) {
            try {
                window.epClickerSocket = new WebSocket(wsUrl);
                window.epClickerSocket.onmessage = (e: MessageEvent) => {
                    const data = JSON.parse(e.data);
                    if (data.global !== undefined) {
                        window.epGlobalScore = data.global;
                        globalScore = data.global;
                    }
                };
            } catch {
                console.error("WS fail");
            }
        } else {
            if (window.epGlobalScore !== undefined) {
                globalScore = window.epGlobalScore;
            }
        }
    });

    $effect(() => {
        if (globalScore !== null) {
            scoreText = globalScore.toLocaleString();
        }
    });

    function handleClick() {
        pendingClicks++;
        const cur = globalScore ?? 0;
        globalScore = cur + 1;
        scoreText = (cur + 1).toLocaleString();
        clearTimeout((handleClick as any).flushTimer);
        (handleClick as any).flushTimer = setTimeout(() => {
            const API_BASE = "https://echopoint.ujjwalvivek.com";
            if (
                window.epClickerSocket &&
                window.epClickerSocket.readyState === WebSocket.OPEN
            ) {
                window.epClickerSocket.send(
                    JSON.stringify({ type: "click", count: pendingClicks }),
                );
            } else {
                fetch(`${API_BASE}/v1/click`, {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ count: pendingClicks }),
                });
            }
            pendingClicks = 0;
        }, 300);
    }
</script>

<section class="hero">
    <div class="hero-top">
        <div class="clicker">
            <div class="clicker-score">{scoreText}</div>
            <button class="clicker-btn" onclick={handleClick}>CLICK ME!</button>
        </div>
    </div>
    <div class="hero-box">
        <div class="box-corner tl"></div>
        <div class="box-corner tr"></div>
        <div class="box-corner bl"></div>
        <div class="box-corner br"></div>

        <div class="hero-left">
            <div class="badges">
                {#each infoBadges as b}
                    <img
                        src="{echopoint}/svg/badges/{b.kind}?repo=devhub&logo=github&{badge}"
                        alt={b.alt}
                        height="24"
                    />
                {/each}
            </div>
            <h1 class="title">Your Project Hub</h1>
            <p class="subtitle">
                Native egui/eframe desktop app, a lightweight dev hub and
                launcher, for scanning, indexing, and browsing local and remote
                SSH software projects, with telemetry tracking powered by
                Echopoint.
            </p>
            <div class="actions">
                <a href="/docs" class="btn">docs</a>
                <a href={dlUrl} class="btn">{dlLabel}</a>
                <a
                    href="https://github.com/ujjwalvivek/devhub"
                    class="btn secondary">github</a
                >
            </div>
        </div>

        <div class="hero-right">
            <div class="portrait-card">
                <div class="card-header">
                    <a
                        href="https://github.com/ujjwalvivek/devhub"
                        class="card-repo">devhub</a
                    >
                    <span class="card-author">ujjwalvivek/devhub</span>
                </div>

                <div class="card-badges">
                    <img
                        src="{echopoint}/svg/badges/stars?repo=devhub&logo=github&{badge}"
                        alt="stars"
                        height="20"
                    />
                    <img
                        src="{echopoint}/svg/badges/updated?repo=devhub&logo=github&{badge}"
                        alt="updated"
                        height="20"
                    />
                    <img
                        src="{echopoint}/svg/badges/release?repo=devhub&logo=github&{badge}"
                        alt="release"
                        height="20"
                    />
                    <img
                        src="{echopoint}/svg/badges/docs?repo=devhub&logo=docs&{badge}"
                        alt="docs"
                        height="20"
                    />
                </div>

                <div class="card-charts">
                    <img
                        src="{echopoint}/svg/commits?repo=devhub&limit=3&width=280&{cardStr}&egui_width=280"
                        alt="commits"
                        class="card-img"
                    />
                    <img
                        src="{echopoint}/svg/releases?repo=devhub&limit=3&width=280&{cardStr}&egui_width=280"
                        alt="releases"
                        class="card-img"
                    />
                    <img
                        src="{echopoint}/svg/langs?repo=devhub&limit=6&width=280&height=8&{cardStr}&pctColor=a6a6a6&{langStr}&color2=c0c0c0&color3=969696&color4=6d6d6d&color5=464646&egui_width=280"
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
        padding: 60px 24px 24px;
        max-width: 1200px;
        margin: 0 auto;
    }
    .hero-top {
        position: relative;
        display: flex;
        gap: 0;
        background: color-mix(in srgb, var(--bg-card) 10%, transparent);
        backdrop-filter: blur(32px);
        -webkit-backdrop-filter: blur(32px);
        border: 2px dashed color-mix(in srgb, var(--border) 50%, transparent);
        padding: 16px;
        margin-bottom: 16px;
    }
    .hero-box {
        position: relative;
        display: flex;
        gap: 0;
        background: color-mix(in srgb, var(--bg-card) 10%, transparent);
        backdrop-filter: blur(32px);
        -webkit-backdrop-filter: blur(32px);
        border: 2px dashed color-mix(in srgb, var(--border) 50%, transparent);
        padding: 16px;
        margin-bottom: 64px;
    }
    .slider-box {
        position: relative;
        width: 100%;
        max-width: 1200px;
        background: color-mix(in srgb, var(--bg-card) 10%, transparent);
        backdrop-filter: blur(32px);
        -webkit-backdrop-filter: blur(32px);
        border: 2px dashed color-mix(in srgb, var(--border) 50%, transparent);
        padding: 16px;
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
        padding-right: 48px;
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
        padding: 20px;
        display: flex;
        flex-direction: column;
        gap: 12px;
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
        margin-bottom: 28px;
        flex-wrap: wrap;
    }
    .title {
        font-family: var(--font-mono);
        font-size: 44px;
        font-weight: 800;
        letter-spacing: -1px;
        margin-bottom: 14px;
        line-height: 1.15;
        text-transform: uppercase;
        text-shadow: 0 0 6px var(--accent);
    }
    .subtitle {
        font-size: 14px;
        color: var(--text-dim);
        line-height: 1.75;
        margin-bottom: 32px;
        max-width: 700px;
    }
    .actions {
        display: grid;
        grid-template-columns: repeat(3, minmax(0, 1fr));
        gap: 10px;
        width: min(100%, 520px);
        margin-bottom: 32px;
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

    .clicker {
        display: flex;
        align-items: stretch;
        width: 100%;
        gap: 0;
    }
    .clicker-score {
        display: flex;
        flex: 1;
        align-items: center;
        justify-content: center;
        font-size: 1.5rem;
        font-weight: bold;
        color: var(--accent);
        font-family: var(--font-mono);
        letter-spacing: 0.05em;
        text-shadow: 0 0 10px var(--accent);
        border: 1px solid var(--border);
        background: var(--bg-card);
        padding: 9px 20px;
        min-width: 100px;
        border-right: none;
    }
    .clicker-btn {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        font-family: var(--font-mono);
        font-size: 12px;
        padding: 9px 24px;
        border: 1px solid var(--border);
        background: var(--bg-card);
        color: var(--text);
        text-decoration: none;
        cursor: pointer;
        transition:
            background 0.15s,
            border-color 0.15s,
            transform 0.1s ease;
    }
    .clicker-btn:hover {
        background: var(--bg-soft);
        border-color: var(--text-muted);
    }
    .clicker-btn:active {
        transform: scale(0.97);
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
        .clicker {
            width: 100%;
        }
        .title {
            text-shadow: 0 0 2px var(--accent);
        }
        .clicker-score {
            text-shadow: 0 0 3px var(--accent);
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
