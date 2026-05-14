<script lang="ts">
    const SUBSTRATE =
        "https://cdn.ujjwalvivek.com/scripts/substrate/latest/main.js";

    let canvas: HTMLCanvasElement;
    let isStatic = $state(false);
    let hidden = $state(false);
    let mod: any = null;
    let scene: any = null;
    let mo: MutationObserver | null = null;

    function palette() {
        const s = getComputedStyle(document.documentElement);
        return {
            primary: s.getPropertyValue("--accent").trim() || "#ffffff",
            secondary: s.getPropertyValue("--bg").trim() || "#999999",
            accent: s.getPropertyValue("--text").trim() || "#e8e8e8",
            background: s.getPropertyValue("--bg").trim() || "#000000",
        };
    }

    function sceneForTheme() {
        var theme = document.documentElement.getAttribute("data-theme");
        var vOpts =
            theme === "horizon"
                ? { opacity: 0.5, innerRadius: 0.9, outerRadius: 0.3 }
                : { opacity: 0.5 };

        return mod.compose([
            {
                fn: mod.primitives.background,
                options: {
                    colorStops: [
                        [0, "primary", "20"],
                        [0.3, "secondary", "08"],
                        [1, "background", "00"],
                    ],
                },
            },
            {
                fn: mod.primitives.foam,
                options: {
                    spacing: 36,
                    opacity: 0.15,
                    speed: 0.2,
                },
            },
            {
                fn: mod.primitives.grid,
                options: {
                    density: 1.0,
                    amplitude: 60,
                    thickness: 0.4,
                    opacity: 0.9,
                    speed: 0.9,
                },
            },
            {
                fn: mod.primitives.wireframes,
                options: {
                    density: 1.2,
                    cubeSize: 40,
                    opacity: 0.35,
                    speed: 0.15,
                },
            },
            {
                fn: mod.primitives.particles,
                options: {
                    density: 0.7,
                    minRadius: 150,
                    maxRadius: 950,
                    opacity: 0.5,
                    speed: 0.05,
                },
            },
            { fn: mod.primitives.vignette, options: vOpts },
        ]);
    }

    function apply() {
        if (!mod || !canvas) return;
        mod.stop();
        scene = sceneForTheme();
        const p = palette();
        if (isStatic) {
            mod.renderStatic(canvas, scene, p, { fps: 0 });
        } else {
            mod.loop(canvas, scene, p, { fps: 30 });
        }
        window.dispatchEvent(
            new CustomEvent("wallpaper-state", {
                detail: { isStatic, hidden },
            }),
        );
    }

    function toggle() {
        isStatic = !isStatic;
        apply();
    }

    $effect(() => {
        document.body.appendChild(canvas);

        function onToggle() {
            toggle();
        }
        function onHide() {
            hidden = !hidden;
            canvas.style.display = hidden ? "none" : "";
            window.dispatchEvent(
                new CustomEvent("wallpaper-state", {
                    detail: { isStatic, hidden },
                }),
            );
        }
        window.addEventListener("wallpaper-toggle", onToggle);
        window.addEventListener("wallpaper-hide", onHide);

        import(SUBSTRATE).then((m) => {
            mod = m;
            apply();

            mo = new MutationObserver(() => apply());
            mo.observe(document.documentElement, {
                attributes: true,
                attributeFilter: ["data-theme", "data-mode"],
            });
        });

        return () => {
            if (mod) mod.stop();
            if (mo) mo.disconnect();
            window.removeEventListener("wallpaper-toggle", onToggle);
            window.removeEventListener("wallpaper-hide", onHide);
            if (canvas?.parentNode) canvas.parentNode.removeChild(canvas);
        };
    });
</script>

<canvas bind:this={canvas}></canvas>

<style>
    canvas {
        position: fixed;
        inset: 0;
        z-index: -1;
        display: block;
        pointer-events: none;
    }
</style>
