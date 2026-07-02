<script lang="ts">
    let globalScore = $state<number | null>(null);
    let pendingClicks = $state(0);
    let scoreText = $state("...");

    $effect(() => {
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

<section class="clicker-section">
    <div class="clicker">
        <div class="clicker-score">{scoreText}</div>
        <button class="clicker-btn" onclick={handleClick}>CLICK ME!</button>
    </div>
</section>

<style>
    .clicker-section {
        padding: 24px;
        max-width: 1200px;
        margin: 0 auto;
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
    @media (max-width: 850px) {
        .clicker {
            width: 100%;
        }
        .clicker-score {
            text-shadow: 0 0 3px var(--accent);
        }
    }
</style>
