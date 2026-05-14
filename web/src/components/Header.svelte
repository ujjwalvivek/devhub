<script lang="ts">
    let { showWallpaperBtn = true }: { showWallpaperBtn?: boolean } = $props();

    const themes: { id: string; label: string; color: string }[] = [
        { id: "catppuccin", label: "Catppuccin", color: "#89b4fa" },
        { id: "rose-pine", label: "Rose Pine", color: "#ea9a97" },
        { id: "tokyo-night", label: "Tokyo Night", color: "#7aa2f7" },
        { id: "horizon", label: "Horizon", color: "#e95678" },
        { id: "monochrome", label: "Monochrome", color: "#ffffff" },
    ];

    const modes: { id: string; label: string }[] = [
        { id: "system", label: "system" },
        { id: "dark", label: "dark" },
        { id: "light", label: "light" },
    ];

    let open = $state(false);
    let currentTheme = $state("monochrome");
    let currentMode = $state("system");

    $effect(() => {
        currentTheme = localStorage.getItem("devhub-theme") || "monochrome";
        currentMode = localStorage.getItem("devhub-mode") || "system";
    });

    function pickTheme(id: string) {
        currentTheme = id;
        localStorage.setItem("devhub-theme", id);
        applyTheme(id, currentMode);
        open = false;
    }

    function pickMode(id: string) {
        currentMode = id;
        localStorage.setItem("devhub-mode", id);
        applyTheme(currentTheme, id);
        open = false;
    }

    function applyTheme(th: string, mo: string) {
        var effective = mo;
        if (mo === "system") {
            effective = window.matchMedia("(prefers-color-scheme: light)")
                .matches
                ? "light"
                : "dark";
        }
        document.documentElement.setAttribute("data-theme", th);
        document.documentElement.setAttribute("data-mode", effective);
    }

    function toggle(e: Event) {
        e.stopPropagation();
        open = !open;
    }

    function close(e: Event) {
        var target = e.target as HTMLElement;
        if (!document.querySelector(".theme-wrapper")?.contains(target)) {
            open = false;
        }
    }

    let isStatic = $state(false);

    $effect(() => {
        function onState(e: Event) {
            isStatic = (e as CustomEvent).detail.isStatic;
        }
        window.addEventListener("wallpaper-state", onState);
        return () => window.removeEventListener("wallpaper-state", onState);
    });

    function toggleAnim() {
        window.dispatchEvent(new CustomEvent("wallpaper-toggle"));
    }

    function hideWallpaper(e: Event) {
        e.preventDefault();
        window.dispatchEvent(new CustomEvent("wallpaper-hide"));
    }
</script>

<svelte:window onclick={close} />

<header>
    <nav>
        <a href="/" class="brand">D E V H U B</a>
        <div class="right">
            {#if showWallpaperBtn}
                <button
                    class="anim-btn"
                    onclick={toggleAnim}
                    oncontextmenu={hideWallpaper}
                    aria-label="toggle animation"
                >
                    <svg
                        width="16"
                        height="16"
                        viewBox="0 0 16 16"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="1.3"
                        stroke-linecap="square"
                    >
                        <rect
                            x="1.5"
                            y="1.5"
                            width="13"
                            height="13"
                            class:fill={!isStatic}
                        />
                        {#if !isStatic}
                            <rect
                                x="4"
                                y="4"
                                width="8"
                                height="8"
                                class="conc-inner"
                            />
                            <rect
                                x="6"
                                y="6"
                                width="4"
                                height="4"
                                class="conc-inner"
                            />
                        {/if}
                    </svg>
                </button>
            {/if}
            <div class="theme-wrapper">
                <button class="theme-btn" onclick={toggle}>
                    <span
                        class="dot"
                        style="background: {themes.find(
                            (t) => t.id === currentTheme,
                        )?.color}"
                    ></span>
                    <span class="theme-label"
                        >{themes.find((t) => t.id === currentTheme)?.label ??
                            currentTheme}</span
                    >
                </button>
                {#if open}
                    <div class="theme-dropdown">
                        {#each themes as t}
                            <button
                                class="theme-option"
                                class:active={currentTheme === t.id}
                                onclick={() => pickTheme(t.id)}
                            >
                                <span class="dot" style="background: {t.color}"
                                ></span>
                                <span class="option-label">{t.label}</span>
                                {#if currentTheme === t.id}<span class="check"
                                        >✦</span
                                    >{/if}
                            </button>
                        {/each}
                        <div class="divider"></div>
                        <div class="mode-row">
                            {#each modes as m}
                                <button
                                    class="mode-btn"
                                    class:active={currentMode === m.id}
                                    onclick={() => pickMode(m.id)}
                                >
                                    {m.label}
                                </button>
                            {/each}
                        </div>
                    </div>
                {/if}
            </div>
        </div>
    </nav>
</header>

<style>
    header {
        position: sticky;
        top: 0;
        z-index: 100;
        background: color-mix(in srgb, var(--bg) 30%, transparent);
        backdrop-filter: blur(32px);
        border-bottom: 1px solid var(--border);
        transition:
            background 0.2s,
            border-color 0.2s;
    }
    nav {
        max-width: 1200px;
        margin: 0 auto;
        padding: 0 24px;
        height: 40px;
        display: flex;
        align-items: center;
        justify-content: space-between;
    }
    .brand {
        font-family: var(--font-mono);
        font-size: 15px;
        font-weight: 500;
        letter-spacing: -0.3px;
    }
    .right {
        display: flex;
        align-items: center;
        gap: 8px;
    }
    .anim-btn {
        width: 25px;
        height: 25px;
        display: flex;
        align-items: center;
        justify-content: center;
        background: none;
        border: 1px solid var(--border);
        color: var(--text-muted);
        cursor: pointer;
        padding: 0;
        transition:
            color 0.15s,
            border-color 0.15s;
    }
    .anim-btn:hover {
        color: var(--text);
        border-color: var(--text-muted);
    }
    .anim-btn svg {
        height: 15px;
        width: 15px;
        display: block;
    }
    .anim-btn .fill {
        fill: var(--accent);
        stroke: var(--accent);
        transition:
            fill 0.2s,
            stroke 0.2s;
    }
    .conc-inner {
        animation: conc-pulse 2s ease-in-out infinite;
        transform-origin: center;
        stroke: var(--accent);
    }
    .conc-inner:nth-child(2) {
        animation-delay: 0.3s;
    }
    .conc-inner:nth-child(3) {
        animation-delay: 0.6s;
    }
    @keyframes conc-pulse {
        0%,
        100% {
            opacity: 0.2;
        }
        50% {
            opacity: 1;
        }
    }
    .theme-wrapper {
        position: relative;
    }
    .theme-btn {
        font-family: var(--font-mono);
        font-size: 12px;
        background: var(--bg-card);
        border: 1px solid var(--border);
        color: var(--text-dim);
        padding: 4px 10px;
        cursor: pointer;
        display: flex;
        align-items: center;
        gap: 6px;
        transition:
            color 0.15s,
            border-color 0.15s;
        border-radius: 0;
    }
    .theme-btn:hover {
        color: var(--text);
        border-color: var(--text-muted);
    }
    .dot {
        width: 6px;
        height: 6px;
        flex-shrink: 0;
    }
    .theme-label {
        text-transform: capitalize;
        transform: translateY(-1px);
    }
    .theme-dropdown {
        position: absolute;
        right: 0;
        top: calc(100% + 4px);
        background: var(--bg-card);
        border: 1px solid var(--border);
        min-width: 190px;
        z-index: 200;
    }
    .theme-option {
        display: flex;
        align-items: center;
        gap: 8px;
        width: 100%;
        padding: 7px 12px;
        font-family: var(--font-mono);
        font-size: 12px;
        background: none;
        border: none;
        color: var(--text-dim);
        cursor: pointer;
        text-align: left;
        transition:
            background 0.1s,
            color 0.1s;
        border-radius: 0;
    }
    .theme-option:hover {
        background: var(--bg-soft);
        color: var(--text);
    }
    .theme-option.active {
        color: var(--accent);
    }
    .option-label {
        flex: 1;
    }
    .check {
        color: var(--accent);
        font-size: 10px;
    }
    .divider {
        height: 1px;
        background: var(--border);
        margin: 4px 8px;
    }
    .mode-row {
        display: flex;
        gap: 0;
        padding: 6px 8px;
    }
    .mode-btn {
        flex: 1;
        font-family: var(--font-mono);
        font-size: 11px;
        padding: 5px 0;
        background: none;
        border: 1px solid var(--border);
        color: var(--text-muted);
        cursor: pointer;
        transition:
            background 0.1s,
            color 0.1s,
            border-color 0.1s;
        border-radius: 0;
    }
    .mode-btn + .mode-btn {
        border-left: 0;
    }
    .mode-btn.active {
        background: var(--bg-soft);
        color: var(--text);
        border-color: var(--text-muted);
    }
    .mode-btn:hover:not(.active) {
        color: var(--text-dim);
        border-color: var(--text-muted);
    }
</style>
