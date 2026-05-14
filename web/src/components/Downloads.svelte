<script lang="ts">
    type Asset = {
        name: string;
        url: string;
        sha: string;
    };

    let version = $state("");
    let badge = $state("");
    let linux: Asset | null = $state(null);
    let macos: Asset | null = $state(null);
    let windows: Asset | null = $state(null);
    let loading = $state(true);
    let feedback: Record<string, string> = $state({});

    const echopoint = "https://echopoint.ujjwalvivek.com";

    $effect(() => {
        function update() {
            var s = getComputedStyle(document.documentElement);
            var read = (n: string) =>
                s.getPropertyValue(n).trim().replace("#", "");
            badge = `bg=${read("--bg-card")}&badgeColor=${read("--border")}&textColor=${read("--text")}&border=${read("--border")}&borderWidth=2&rx=0&px=6&py=4`;
        }
        update();
        var observer = new MutationObserver(update);
        observer.observe(document.documentElement, {
            attributes: true,
            attributeFilter: ["data-theme", "data-mode"],
        });
        return () => observer.disconnect();
    });

    const SIZES: Record<string, number> = {
        linux: 8293934,
        macos: 6566924,
        windows: 5874729,
    };

    $effect(() => {
        fetch("https://api.github.com/repos/ujjwalvivek/devhub/releases/latest")
            .then((r) => r.json())
            .then((data) => {
                version = data.tag_name;
                for (const a of data.assets) {
                    const sha = (a.digest || "").replace("sha256:", "");
                    const asset: Asset = {
                        name: a.name,
                        url: a.browser_download_url,
                        sha,
                    };
                    if (a.name.includes("linux")) linux = asset;
                    else if (a.name.includes("apple-darwin")) macos = asset;
                    else if (a.name.includes("windows")) windows = asset;
                }
            })
            .finally(() => (loading = false));
    });

    function sizeMB(os: string) {
        return (SIZES[os] / 1024 / 1024).toFixed(1);
    }

    function assetFor(os: string): Asset | null {
        if (os === "linux") return linux;
        if (os === "macos") return macos;
        if (os === "windows") return windows;
        return null;
    }

    const oses = [
        { id: "linux", label: "linux" },
        { id: "macos", label: "macOS (Intel)" },
        { id: "windows", label: "windows" },
    ] as const;

    function handleClick(os: string) {
        const a = assetFor(os);
        if (!a) return;
        navigator.clipboard.writeText(a.sha);
        feedback[os] = "SHA copied!";
        setTimeout(() => (feedback[os] = ""), 3000);
        window.location.href = a.url;
    }
</script>

<section class="downloads" id="downloads">
    <div class="container">
        <h2 class="section-title">
            download
            {#if version}
                <img
                    src="{echopoint}/svg/badges/release?repo=devhub&logo=github&{badge}"
                    alt={version}
                    height="24"
                    class="version-badge"
                />
            {/if}
        </h2>
        {#if loading}
            <p class="loading">fetching latest release…</p>
        {:else}
            <div class="os-grid">
                {#each oses as os}
                    {@const a = assetFor(os.id)}
                    <button
                        class="dl-btn"
                        class:disabled={!a}
                        class:copied={!!feedback[os.id]}
                        onclick={() => handleClick(os.id)}
                        disabled={!a}
                    >
                        <svg
                            width="24"
                            height="24"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="1.5"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                        >
                            {#if os.id === "linux"}
                                <path
                                    fill="currentColor"
                                    d="M14.62 8.35c-.42.28-1.75 1.04-1.95 1.19c-.39.31-.75.29-1.14-.01c-.2-.16-1.53-.92-1.95-1.19c-.48-.31-.45-.7.08-.92c1.64-.69 3.28-.64 4.91.03c.49.21.51.6.05.9m7.22 7.28c-.93-2.09-2.2-3.99-3.84-5.66a4.3 4.3 0 0 1-1.06-1.88c-.1-.33-.17-.67-.24-1.01c-.2-.88-.29-1.78-.7-2.61c-.73-1.58-2-2.4-3.84-2.47c-1.81.05-3.16.81-3.95 2.4c-.21.43-.36.88-.46 1.34c-.17.76-.32 1.55-.5 2.32c-.15.65-.45 1.21-.96 1.71c-1.61 1.57-2.9 3.37-3.88 5.35c-.14.29-.28.58-.37.88c-.19.66.29 1.12.99.96c.44-.09.88-.18 1.3-.31c.41-.15.57-.05.67.35c.65 2.15 2.07 3.66 4.24 4.5c4.12 1.56 8.93-.66 9.97-4.58c.07-.27.17-.37.47-.27c.46.14.93.24 1.4.35c.49.09.85-.16.92-.64c.03-.26-.06-.49-.16-.73"
                                />
                            {:else if os.id === "macos"}
                                <path
                                    fill="currentColor"
                                    d="M17.05 20.28c-.98.95-2.05.8-3.08.35c-1.09-.46-2.09-.48-3.24 0c-1.44.62-2.2.44-3.06-.35C2.79 15.25 3.51 7.59 9.05 7.31c1.35.07 2.29.74 3.08.8c1.18-.24 2.31-.93 3.57-.84c1.51.12 2.65.72 3.4 1.8c-3.12 1.87-2.38 5.98.48 7.13c-.57 1.5-1.31 2.99-2.54 4.09zM12.03 7.25c-.15-2.23 1.66-4.07 3.74-4.25c.29 2.58-2.34 4.5-3.74 4.25"
                                />
                            {:else}
                                <path
                                    d="M3 3h8v8H3zM13 3h8v8h-8zM3 13h8v8H3zM13 13h8v8h-8z"
                                />
                            {/if}
                        </svg>
                        <span class="dl-label"
                            >{feedback[os.id] || os.label}</span
                        >
                        <span class="dl-size">{sizeMB(os.id)} MB</span>
                    </button>
                {/each}
            </div>
        {/if}
    </div>
</section>

<style>
    .downloads {
        padding: 20px 24px 40px;
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
    .version-badge {
        vertical-align: middle;
        margin-left: 8px;
    }
    .loading {
        font-size: 13px;
        color: var(--text-muted);
    }
    .os-grid {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        gap: 12px;
    }
    .dl-btn {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 10px;
        padding: 28px 16px;
        background: color-mix(in srgb, var(--accent) 7%, transparent);
        backdrop-filter: blur(24px);
        -webkit-backdrop-filter: blur(24px);
        border: 2px solid color-mix(in srgb, var(--accent) 50%, transparent);
        color: var(--text);
        font-family: var(--font-mono);
        font-size: 13px;
        cursor: pointer;
        transition:
            border-color 0.15s,
            background 0.15s,
            color 0.15s;
    }
    .dl-btn svg {
        color: var(--accent);
    }
    .dl-btn:hover {
        border-color: var(--accent);
        background: color-mix(in srgb, var(--bg-card) 25%, transparent);
    }
    .dl-btn:disabled {
        opacity: 0.4;
        pointer-events: none;
    }
    .dl-btn.copied {
        border-color: var(--accent);
        color: var(--accent);
    }
    .dl-label {
        line-height: 1;
    }
    .dl-size {
        font-size: 10px;
        color: var(--accent);
        line-height: 1;
        font-weight: 600;
    }
    @media (max-width: 768px) {
        .os-grid {
            grid-template-columns: repeat(1, 1fr);
        }
    }
</style>
