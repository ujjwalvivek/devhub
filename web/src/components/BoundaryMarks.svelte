<script lang="ts">
    type Variant = "a" | "b" | "c" | "d" | "e";

    let { variant = "a" }: { variant?: Variant } = $props();
</script>

<div class={`marks variant-${variant}`} aria-hidden="true">
    <span class="lattice primary">
        {#each Array(12) as _, index}<i class:accent={index === 1 || index === 10}></i>{/each}
    </span>
    <span class="lattice echo">
        {#each Array(6) as _, index}<i class:accent={index === 4}></i>{/each}
    </span>
    <span class="lattice echo-secondary">
        {#each Array(8) as _, index}<i class:accent={index === 1}></i>{/each}
    </span>
    <span class="dot-column rail-primary">
        {#each Array(6) as _, index}<i class:accent={index === 1}></i>{/each}
    </span>
    <span class="dot-column rail-secondary">
        {#each Array(4) as _, index}<i class:accent={index === 3}></i>{/each}
    </span>
    <span class="calibration calibration-primary"><i></i><b></b><i></i><b></b></span>
    <span class="calibration calibration-secondary"><i></i><b></b><i></i><b></b></span>
    <span class="baseline">
        {#each Array(7) as _, index}<i class:accent={index === 2 || index === 6}></i>{/each}
    </span>
</div>

<style>
    .marks {
        position: absolute;
        inset: 0;
        z-index: 4;
        overflow: visible;
        pointer-events: none;
    }

    .lattice,
    .dot-column,
    .calibration,
    .baseline {
        position: absolute;
        display: grid;
        transform: scale(1.1);
        transform-origin: center;
    }

    .lattice {
        gap: 3px;
    }

    .lattice.primary {
        grid-template-columns: repeat(6, 2px);
    }

    .lattice.echo {
        grid-template-columns: repeat(3, 2px);
    }

    .lattice.echo-secondary {
        grid-template-columns: repeat(4, 2px);
    }

    .dot-column {
        grid-template-columns: 2px;
        gap: 3px;
    }

    .baseline {
        grid-template-columns: repeat(7, 2px);
        gap: 3px;
    }

    .lattice i,
    .dot-column i,
    .baseline i {
        width: 2px;
        height: 2px;
        background: var(--text-muted);
        opacity: 0.48;
    }

    .lattice i.accent,
    .dot-column i.accent,
    .baseline i.accent {
        background: var(--accent);
        opacity: 0.9;
    }

    .calibration {
        grid-template-columns: 5px 1px 10px 1px;
        align-items: center;
        gap: 3px;
        width: 26px;
        height: 7px;
    }

    .calibration i,
    .calibration b {
        display: block;
        background: var(--text-muted);
        opacity: 0.5;
    }

    .calibration i {
        width: 5px;
        height: 1px;
    }

    .calibration b {
        width: 1px;
        height: 5px;
    }

    /* A 12-column placement system, mirrored rather than randomized. */
    .variant-a .primary { top: -16px; left: 66.667%; }
    .variant-a .echo { top: 62px; right: -21px; }
    .variant-a .echo-secondary { bottom: 28%; left: -26px; }
    .variant-a .rail-primary { bottom: 72px; left: -11px; }
    .variant-a .rail-secondary { top: 32%; right: -11px; }
    .variant-a .calibration-primary { top: -16px; left: 16.667%; }
    .variant-a .calibration-secondary { right: 25%; bottom: -16px; }
    .variant-a .baseline { bottom: -11px; left: 50%; }

    .variant-b .primary { top: -16px; right: 8.333%; }
    .variant-b .echo { top: 78px; left: -21px; }
    .variant-b .echo-secondary { right: -26px; bottom: 34%; }
    .variant-b .rail-primary { right: -11px; bottom: 88px; }
    .variant-b .rail-secondary { top: 38%; left: -11px; }
    .variant-b .calibration-primary { top: -16px; left: 41.667%; }
    .variant-b .calibration-secondary { bottom: -16px; left: 16.667%; }
    .variant-b .baseline { right: 33.333%; bottom: -11px; }

    .variant-c .primary { top: -16px; left: 8.333%; }
    .variant-c .echo { top: 66px; right: -21px; }
    .variant-c .echo-secondary { bottom: 30%; left: -26px; }
    .variant-c .rail-primary { bottom: 80px; left: -11px; }
    .variant-c .rail-secondary { top: 36%; right: -11px; }
    .variant-c .calibration-primary { top: -16px; right: 33.333%; }
    .variant-c .calibration-secondary { right: 8.333%; bottom: -16px; }
    .variant-c .baseline { bottom: -11px; left: 33.333%; }

    .variant-d .primary { top: -16px; right: 25%; }
    .variant-d .echo { top: 66px; left: -21px; }
    .variant-d .echo-secondary { right: -26px; bottom: 26%; }
    .variant-d .rail-primary { right: -11px; bottom: 88px; }
    .variant-d .rail-secondary { top: 34%; left: -11px; }
    .variant-d .calibration-primary { top: -16px; left: 16.667%; }
    .variant-d .calibration-secondary { bottom: -16px; left: 41.667%; }
    .variant-d .baseline { right: 16.667%; bottom: -11px; }

    .variant-e .primary { top: -16px; left: 58.333%; }
    .variant-e .echo { top: 62px; right: -21px; }
    .variant-e .echo-secondary { bottom: 32%; left: -26px; }
    .variant-e .rail-primary { bottom: 72px; left: -11px; }
    .variant-e .rail-secondary { top: 40%; right: -11px; }
    .variant-e .calibration-primary { top: -16px; left: 25%; }
    .variant-e .calibration-secondary { right: 16.667%; bottom: -16px; }
    .variant-e .baseline { bottom: -11px; left: 41.667%; }

    @media (max-width: 680px) {
        .marks {
            inset: 0;
        }

        .echo,
        .echo-secondary,
        .rail-primary,
        .rail-secondary,
        .calibration-secondary {
            display: none;
        }
    }
</style>
