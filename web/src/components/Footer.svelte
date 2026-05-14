<footer>
    <div class="container">
        <span class="copy">devhub &mdash; mit</span>
        <a href="https://github.com/ujjwalvivek/devhub" class="link">github</a>
    </div>
</footer>

<style>
    footer {
        border-top: 1px solid var(--border);
        padding: 0 24px;
        background: color-mix(in srgb, var(--bg) 30%, transparent);
        backdrop-filter: blur(32px);
    }
    .container {
        max-width: 1200px;
        margin: 0 auto;
        height: 30px;
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0 24px;
    }
    .copy {
        font-family: var(--font-mono);
        font-size: 12px;
        color: var(--text-muted);
    }
    .link {
        font-family: var(--font-mono);
        font-size: 12px;
        color: var(--text-dim);
    }
    .link:hover {
        color: var(--text);
        opacity: 1;
    }
</style>
