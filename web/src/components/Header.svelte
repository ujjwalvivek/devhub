<script lang="ts">
    type Command = {
        label: string;
        hint: string;
        group: string;
        href?: string;
        mode?: "system" | "dark" | "light";
        action?: "appearance";
    };

    const navigation: Command[] = [
        {
            label: "Download DevHub 2.1.4",
            hint: "Windows, Linux, and macOS",
            group: "Go",
            href: "/#download",
        },
        {
            label: "Product workflow",
            hint: "Overview, files, search, Git, and history",
            group: "Go",
            href: "/#workflow",
        },
        {
            label: "Project intelligence",
            hint: "Read-only MCP tools",
            group: "Go",
            href: "/#mcp",
        },
        {
            label: "Source repository",
            hint: "ujjwalvivek/devhub-gpui",
            group: "Open",
            href: "https://github.com/ujjwalvivek/devhub-gpui",
        },
        {
            label: "Release notes",
            hint: "DevHub 2.1.4",
            group: "Read",
            href: "https://github.com/ujjwalvivek/devhub-gpui/releases/tag/v2.1.4",
        },
    ];

    const appearances: Command[] = [
        {
            label: "Follow System",
            hint: "Match the operating system",
            group: "Appearance",
            mode: "system",
        },
        {
            label: "Dark",
            hint: "Use the monochrome dark appearance",
            group: "Appearance",
            mode: "dark" as const,
        },
        {
            label: "Light",
            hint: "Use the monochrome light appearance",
            group: "Appearance",
            mode: "light" as const,
        },
    ];

    const appearanceCommand: Command = {
        label: "Appearance",
        hint: "Monochrome: system, dark, or light",
        group: "Appearance",
        action: "appearance",
    };

    let open = $state(false);
    let paletteView = $state<"commands" | "appearance">("commands");
    let query = $state("");
    let selected = $state(0);
    let currentMode = $state<"system" | "dark" | "light">("system");
    let progress = $state(0);
    let input = $state<HTMLInputElement>();

    let commands = $derived.by(() => {
        const all = paletteView === "appearance"
            ? appearances
            : [...navigation, appearanceCommand];
        const value = query.trim().toLowerCase();
        if (!value) return all;
        return all.filter((command) =>
            `${command.label} ${command.hint} ${command.group}`
                .toLowerCase()
                .includes(value),
        );
    });

    $effect(() => {
        const storedMode = localStorage.getItem("devhub-mode");
        currentMode = storedMode === "dark" || storedMode === "light"
            ? storedMode
            : "system";

        function updateProgress() {
            const total = document.documentElement.scrollHeight - window.innerHeight;
            progress = total > 0 ? window.scrollY / total : 0;
        }
        updateProgress();
        window.addEventListener("scroll", updateProgress, { passive: true });
        return () => window.removeEventListener("scroll", updateProgress);
    });

    $effect(() => {
        query;
        selected = 0;
    });

    function effectiveMode(mode: "system" | "dark" | "light") {
        if (mode !== "system") return mode;
        return window.matchMedia("(prefers-color-scheme: light)").matches
            ? "light"
            : "dark";
    }

    function applyAppearance(command: Command) {
        const mode = command.mode || "system";
        currentMode = mode;
        localStorage.setItem("devhub-mode", mode);
        document.documentElement.setAttribute("data-mode", effectiveMode(mode));
    }

    function openPalette() {
        open = true;
        paletteView = "commands";
        query = "";
        selected = 0;
        window.setTimeout(() => input?.focus(), 0);
    }

    function closePalette() {
        open = false;
        paletteView = "commands";
        query = "";
    }

    function openAppearance() {
        paletteView = "appearance";
        query = "";
        selected = 0;
        window.setTimeout(() => input?.focus(), 0);
    }

    function backToCommands() {
        paletteView = "commands";
        query = "";
        selected = 0;
        window.setTimeout(() => input?.focus(), 0);
    }

    function run(command: Command) {
        if (command.action === "appearance") {
            openAppearance();
        } else if (command.mode) {
            applyAppearance(command);
            closePalette();
        } else if (command.href) {
            window.location.href = command.href;
        }
    }

    function globalKey(event: KeyboardEvent) {
        if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
            event.preventDefault();
            open ? closePalette() : openPalette();
        } else if (event.key === "Escape") {
            if (open && paletteView === "appearance") backToCommands();
            else closePalette();
        } else if (
            !open &&
            !event.defaultPrevented &&
            !event.repeat &&
            !event.ctrlKey &&
            !event.metaKey &&
            !event.altKey &&
            !event.shiftKey &&
            !document.querySelector("dialog[open]") &&
            !isTypingTarget(event.target)
        ) {
            const destination = {
                p: "/#workflow",
                m: "/#mcp",
                c: "https://github.com/ujjwalvivek/devhub-gpui",
                d: "/#download",
            }[event.key.toLowerCase()];

            if (destination) {
                event.preventDefault();
                navigateWithShortcut(destination);
            }
        }
    }

    function isTypingTarget(target: EventTarget | null) {
        if (!(target instanceof HTMLElement)) return false;
        return (
            target.isContentEditable ||
            target.matches("input, textarea, select, [role='textbox']")
        );
    }

    function navigateWithShortcut(destination: string) {
        if (destination.startsWith("/#") && window.location.pathname === "/") {
            const id = destination.slice(2);
            document.getElementById(id)?.scrollIntoView({
                behavior: window.matchMedia("(prefers-reduced-motion: reduce)").matches
                    ? "auto"
                    : "smooth",
                block: "start",
            });
            window.history.replaceState(null, "", `#${id}`);
            return;
        }
        window.location.href = destination;
    }

    function paletteKey(event: KeyboardEvent) {
        if (
            paletteView === "appearance" &&
            (event.key === "ArrowLeft" || (event.key === "Backspace" && !query))
        ) {
            event.preventDefault();
            backToCommands();
            return;
        }
        if (!commands.length) return;
        if (event.key === "ArrowDown") {
            event.preventDefault();
            selected = (selected + 1) % commands.length;
        } else if (event.key === "ArrowUp") {
            event.preventDefault();
            selected = (selected - 1 + commands.length) % commands.length;
        } else if (event.key === "Enter") {
            event.preventDefault();
            run(commands[selected]);
        }
    }
</script>

<svelte:window onkeydown={globalKey} />

<header>
    <nav aria-label="Primary">
        <a class="brand" href="/" aria-label="DevHub home">
            <span class="brand-mark" aria-hidden="true"></span>
            <span>DevHub</span>
        </a>

        <div class="links">
            <a href="/#workflow" aria-keyshortcuts="p"><span>Product</span><kbd>P</kbd></a>
            <a href="/#mcp" aria-keyshortcuts="m"><span>MCP</span><kbd>M</kbd></a>
            <a href="https://github.com/ujjwalvivek/devhub-gpui" aria-keyshortcuts="c"><span>Github</span><kbd>C</kbd></a>
        </div>

        <div class="actions">
            <button type="button" onclick={openPalette}>
                <span>Search</span><kbd>Ctrl K</kbd>
            </button>
            <a href="/#download" aria-keyshortcuts="d"><span>Download</span><kbd>D</kbd></a>
        </div>
    </nav>
    <div class="progress" style={`transform:scaleX(${progress})`}></div>
</header>

{#if open}
    <div
        class="palette-layer"
        role="presentation"
        onclick={(event) => {
            if (event.target === event.currentTarget) closePalette();
        }}
    >
        <div
            class="palette"
            role="dialog"
            aria-modal="true"
            tabindex="-1"
            aria-label={paletteView === "appearance" ? "Choose an appearance" : "Site commands"}
            onkeydown={paletteKey}
        >
            <div class:has-back={paletteView === "appearance"} class="palette-input">
                {#if paletteView === "appearance"}
                    <button
                        class="palette-back"
                        type="button"
                        aria-label="Back to site commands"
                        title="Back"
                        onclick={backToCommands}
                    >&larr;</button>
                {/if}
                <input
                    bind:this={input}
                    bind:value={query}
                    type="search"
                    autocomplete="off"
                    placeholder={paletteView === "appearance" ? "System, dark, or light" : "Search DevHub"}
                    aria-label={paletteView === "appearance" ? "Search appearances" : "Search commands"}
                />
                <kbd>Esc</kbd>
            </div>
            <div class="palette-list" role="listbox">
                {#each commands as command, index}
                    <button
                        type="button"
                        role="option"
                        aria-selected={selected === index}
                        class:active={selected === index}
                        onmouseenter={() => (selected = index)}
                        onclick={() => run(command)}
                    >
                        <span>{command.label}</span>
                        <small>{command.hint}</small>
                        <b>{command.mode === currentMode ? "Active" : command.group}</b>
                    </button>
                {:else}
                    <p>No matching command</p>
                {/each}
            </div>
            <footer>
                <span>{paletteView === "appearance" ? "Esc to go back" : "Arrow keys to move"}</span>
                <span>Enter to open</span>
            </footer>
        </div>
    </div>
{/if}

<style>
    header {
        position: fixed;
        inset: 0 0 auto;
        z-index: 100;
        height: var(--header-height);
        background: color-mix(in srgb, var(--bg) 95%, transparent);
        border-bottom: 1px solid var(--border);
        backdrop-filter: blur(10px);
    }

    nav {
        width: min(100%, var(--site-width));
        height: 100%;
        margin: 0 auto;
        padding: 0 12px;
        display: grid;
        grid-template-columns: 1fr auto 1fr;
        align-items: center;
        gap: 18px;
    }

    .brand {
        justify-self: start;
        display: inline-flex;
        align-items: center;
        gap: 7px;
        color: var(--text);
        font-family: var(--font-mono);
        font-size: 16px;
        font-weight: 700;
        line-height: 1;
    }

    .brand-mark {
        width: 24px;
        height: 24px;
        background: currentColor;
        -webkit-mask: url("/images/devhub-mark.svg") center / contain no-repeat;
        mask: url("/images/devhub-mark.svg") center / contain no-repeat;
    }

    .links {
        height: 100%;
        display: flex;
    }

    .links a {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 7px;
        padding: 0 13px;
        color: var(--text-dim);
        font-size: 13px;
        font-weight: 500;
    }

    .links a:hover {
        color: var(--text);
        background: var(--bg-soft);
        box-shadow: inset 0 2px 0 var(--accent);
    }

    .actions {
        justify-self: end;
        display: flex;
        gap: 6px;
    }

    .actions button,
    .actions a {
        height: 28px;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        gap: 10px;
        padding: 0 8px;
        border: 1px solid var(--border);
        border-radius: 0;
        font-size: 12px;
    }

    .actions button {
        color: var(--text-dim);
        background: var(--bg-card);
        cursor: pointer;
    }

    .actions a {
        color: var(--accent-ink);
        background: var(--accent);
        border-color: var(--accent);
        font-weight: 700;
    }

    .links kbd,
    .actions kbd {
        min-width: 18px;
        height: 18px;
        display: grid;
        place-items: center;
        padding-inline: 4px;
        color: inherit;
        border: 1px solid color-mix(in srgb, currentColor 32%, transparent);
        font-family: var(--font-mono);
        font-size: 9px;
        line-height: 1;
        opacity: 0.78;
    }

    .links kbd,
    .actions a kbd {
        width: 18px;
        padding-inline: 0;
    }

    .palette-input kbd {
        color: var(--text-muted);
        font-family: var(--font-mono);
        font-size: 11px;
    }

    .progress {
        position: absolute;
        inset: auto 0 -1px;
        height: 1px;
        background: var(--accent);
        transform-origin: left;
    }

    .palette-layer {
        position: fixed;
        inset: 0;
        z-index: 200;
        padding-top: 70px;
        background: color-mix(in srgb, var(--bg) 74%, transparent);
    }

    .palette {
        width: min(calc(100% - 24px), 580px);
        margin: 0 auto;
        background: var(--bg-card);
        border: 1px solid var(--border-strong);
        box-shadow: 0 16px 36px var(--screen-shadow);
        animation: palette-in 150ms var(--ease-out) both;
    }

    .palette-input {
        height: 46px;
        display: grid;
        grid-template-columns: 1fr auto;
        align-items: center;
        gap: 8px;
        padding: 0 12px;
        border-bottom: 1px solid var(--border);
    }

    .palette-input.has-back {
        grid-template-columns: 32px 1fr auto;
        padding-left: 7px;
    }

    .palette-back {
        width: 30px;
        height: 30px;
        display: grid;
        place-items: center;
        color: var(--text-dim);
        background: transparent;
        border: 1px solid var(--border);
        border-radius: 0;
        cursor: pointer;
        font-size: 16px;
    }

    .palette-back:hover,
    .palette-back:focus-visible {
        color: var(--accent-ink);
        background: var(--accent);
        border-color: var(--accent);
    }

    .palette-input input {
        width: 100%;
        color: var(--text);
        background: transparent;
        border: 0;
        outline: 0;
        font-size: 14px;
    }

    .palette-input input::placeholder {
        color: var(--text-muted);
    }

    .palette-list {
        max-height: min(540px, 70vh);
        overflow-y: auto;
        padding: 5px;
    }

    .palette-list button {
        width: 100%;
        min-height: 42px;
        display: grid;
        grid-template-columns: minmax(0, 1fr) minmax(0, 0.85fr) auto;
        align-items: center;
        gap: 12px;
        padding: 0 9px;
        color: var(--text-dim);
        background: transparent;
        border: 0;
        border-radius: 0;
        text-align: left;
        font-size: 13px;
        cursor: pointer;
    }

    .palette-list button.active {
        color: var(--text);
        background: var(--surface-selected);
        box-shadow: inset 2px 0 0 var(--accent);
    }

    .palette-list button span,
    .palette-list button small {
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .palette-list button small {
        color: var(--text-muted);
        font-size: 12px;
    }

    .palette-list button b {
        color: var(--accent);
        font-family: var(--font-mono);
        font-size: 11px;
        font-weight: 500;
    }

    .palette-list > p {
        padding: 24px 10px;
        color: var(--text-muted);
        text-align: center;
        font-size: 13px;
    }

    .palette footer {
        height: 28px;
        display: flex;
        align-items: center;
        gap: 18px;
        padding: 0 10px;
        color: var(--text-muted);
        background: var(--bg-soft);
        border-top: 1px solid var(--border);
        font-size: 12px;
    }

    @keyframes palette-in {
        from {
            opacity: 0;
            transform: translateY(-5px);
        }
    }

    @media (max-width: 820px) {
        nav {
            grid-template-columns: auto 1fr;
        }

        .links {
            display: none;
        }

        .actions {
            justify-self: end;
        }
    }

    @media (max-width: 460px) {
        nav {
            padding: 0 8px;
        }

        .brand > span:last-child,
        .actions button > span {
            display: none;
        }

        .actions a {
            padding-inline: 10px;
        }

        .actions button {
            min-width: 48px;
            justify-content: center;
            padding-inline: 7px;
        }

        .palette-list button {
            grid-template-columns: minmax(0, 1fr) auto;
        }

        .palette-list button small {
            display: none;
        }
    }
</style>
