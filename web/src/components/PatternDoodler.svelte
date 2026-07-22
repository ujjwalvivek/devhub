<script lang="ts">
    import { onMount } from "svelte";

    type Point = { x: number; y: number };
    type Pattern = {
        coords: Point[];
        maxLength: number;
        currentLength: number;
        allowLimit: boolean;
        index: number;
        indexAscending: boolean;
        random?: boolean;
    };

    const SPEED = 1;
    const LINE_WIDTH = 1.5;
    const LINE_DISTANCE = 2.5;
    const WIDTH = 1000;
    const MIN_HEIGHT = 200;

    let canvas: HTMLCanvasElement;

    const point = (x: number, y: number): Point => ({ x, y });
    const repeat = (direction: Point, length: number) =>
        Array.from({ length }, () => point(direction.x, direction.y));

    function decode(sequence: string) {
        const directions: Record<string, Point> = {
            R: point(1, 0),
            L: point(-1, 0),
            U: point(0, -1),
            D: point(0, 1),
            O: point(0, 0),
        };
        return [...sequence].map((value) => directions[value]);
    }

    function spiral(directions: Point[], turns: number) {
        const coords = [point(0, 0)];
        let direction = 0;
        for (let length = 1; length <= turns; length += 1) {
            coords.push(...repeat(directions[direction % 4], length));
            direction += 1;
            coords.push(...repeat(directions[direction % 4], length));
            direction += 1;
        }
        return coords;
    }

    function segmentedSpiral(directions: Point[], lengths: number[]) {
        const coords = [point(0, 0)];
        lengths.forEach((length, index) => {
            coords.push(...repeat(directions[index % directions.length], length));
        });
        return coords;
    }

    function horseshoe(direction: "up" | "down" | "left" | "right", size: number) {
        const routes: Record<typeof direction, Point[]> = {
            up: [point(0, 1), point(1, 0), point(0, -1)],
            down: [point(0, -1), point(1, 0), point(0, 1)],
            left: [point(1, 0), point(0, 1), point(-1, 0)],
            right: [point(-1, 0), point(0, 1), point(1, 0)],
        };
        return [point(0, 0), ...routes[direction].flatMap((step) => repeat(step, size))];
    }

    function zigzag(axis: "vertical" | "horizontal", first: number, count = 26) {
        const coords = [point(0, 0)];
        for (let index = 0; index < count; index += 1) {
            const direction = index % 2 === 0 ? -1 : 1;
            coords.push(
                axis === "vertical" ? point(index % 2 === 0 ? first : -first, 0) : point(0, direction),
                axis === "vertical" ? point(0, first) : point(first, 0),
            );
        }
        return coords;
    }

    function diagonal(horizontal: number, vertical: number, count = 17) {
        return [
            point(0, 0),
            ...Array.from({ length: count }, () => [point(horizontal, 0), point(0, vertical)]).flat(),
        ];
    }

    function makePattern(
        coords: Point[],
        allowLimit: boolean,
        indexAscending: boolean,
        maxLength = coords.length,
        random = false,
    ): Pattern {
        return {
            coords,
            maxLength,
            currentLength: 0,
            allowLimit,
            index: 0,
            indexAscending,
            random,
        };
    }

    function createPatterns() {
        const horizontalLimit = Math.floor(WIDTH / (LINE_WIDTH + LINE_DISTANCE)) / 2;
        const randomLimit = Math.floor(WIDTH / (LINE_WIDTH + LINE_DISTANCE));
        const patterns: Pattern[] = [
            makePattern([point(0, -1)], true, false, horizontalLimit),
            makePattern([point(0, 1)], true, false, horizontalLimit),
            makePattern([point(-1, 0)], true, false, horizontalLimit),
            makePattern([point(1, 0)], true, false, horizontalLimit),
            makePattern([point(0, 0)], true, false, randomLimit, true),
            makePattern(spiral([point(-1, 0), point(0, -1), point(1, 0), point(0, 1)], 10), false, true),
            makePattern(spiral([point(1, 0), point(0, 1), point(-1, 0), point(0, -1)], 10), false, true),
            makePattern(decode("RRDDLLUU"), true, true),
            makePattern(decode("ORDLLDDLUULLURRUURDD"), false, true),
            makePattern(decode("ORRDDLLDDLLUULLUURRUURRD"), false, true),
            makePattern(decode("ORRRDDLLLDDDLLUUULLLUURRRUUURRDD"), false, true),
            makePattern(decode("ORRRDDDLLLDDDLLLUUULLLUUURRRUUURRRDD"), false, true),
            makePattern(horseshoe("up", 1), false, true),
            makePattern(horseshoe("down", 1), false, true),
            makePattern(horseshoe("left", 1), false, true),
            makePattern(horseshoe("right", 1), false, true),
            makePattern(horseshoe("up", 2), true, true),
            makePattern(horseshoe("down", 2), true, true),
            makePattern(horseshoe("left", 2), true, true),
            makePattern(horseshoe("right", 2), true, true),
            makePattern(horseshoe("up", 3), true, true),
            makePattern(horseshoe("down", 3), true, true),
            makePattern(horseshoe("left", 3), true, true),
            makePattern(horseshoe("right", 3), true, true),
            makePattern(zigzag("vertical", -1), true, true),
            makePattern(zigzag("vertical", 1, 28), true, true),
            makePattern(zigzag("horizontal", -1), true, true),
            makePattern(zigzag("horizontal", 1), true, true),
            makePattern(
                segmentedSpiral(
                    [point(1, 0), point(0, 1), point(-1, 0), point(0, -1)],
                    [2, 2, 4, 4, 6, 6, 8, 8, 10, 10, 12],
                ),
                false,
                true,
            ),
            makePattern(
                segmentedSpiral(
                    [point(-1, 0), point(0, 1), point(1, 0), point(0, -1)],
                    [3, 3, 6, 6, 9, 9, 12, 12, 12],
                ),
                false,
                true,
            ),
            makePattern(
                decode("OLLURUULULULULULULULURRRRRRRULURURRDDDRRRRRRRDLDLDLDLDLDLDDRDL"),
                false,
                true,
            ),
            makePattern(diagonal(-1, -1), true, true),
            makePattern(diagonal(1, -1), true, true),
            makePattern(diagonal(-1, 1), true, true),
            makePattern(diagonal(1, 1), true, true),
        ];
        return { patterns, randomPattern: patterns[4] };
    }

    function startDoodler(onComplete: () => void) {
        const bounds = canvas.getBoundingClientRect();
        const pixelRatio = Math.min(window.devicePixelRatio || 1, 2);
        const logicalHeight = Math.max(
            MIN_HEIGHT,
            Math.round(WIDTH * (bounds.height / Math.max(bounds.width, 1))),
        );
        canvas.width = Math.max(1, Math.round(bounds.width * pixelRatio));
        canvas.height = Math.max(1, Math.round(bounds.height * pixelRatio));

        const context = canvas.getContext("2d");
        if (!context) return () => {};

        const scale = canvas.width / WIDTH;
        context.setTransform(scale, 0, 0, scale, 0, 0);

        const stepSize = LINE_WIDTH + LINE_DISTANCE;
        const columns = Math.floor(WIDTH / stepSize);
        const rows = Math.floor(logicalHeight / stepSize);
        const total = columns * rows;
        const lineHolder = new Uint8Array(total);
        const openCells = Array.from({ length: total }, (_, index) => index);
        const openPositions = new Int32Array(total);
        openCells.forEach((cell, index) => (openPositions[cell] = index));
        const { patterns, randomPattern } = createPatterns();

        let patternCurrent: Pattern | null = null;
        let position: Point;
        let counter = 0;
        let halftime = false;

        context.clearRect(0, 0, WIDTH, logicalHeight);
        const rootStyles = getComputedStyle(document.documentElement);
        context.strokeStyle = rootStyles.getPropertyValue("--text-muted").trim() || "#747474";
        context.lineWidth = LINE_WIDTH;
        context.lineCap = "square";
        context.lineJoin = "miter";

        const indexOf = (value: Point) => value.y * columns + value.x;
        const isOpen = (value: Point) =>
            value.x >= 0 &&
            value.x < columns &&
            value.y >= 0 &&
            value.y < rows &&
            lineHolder[indexOf(value)] === 0;

        function occupy(value: Point) {
            const cell = indexOf(value);
            if (lineHolder[cell]) return;
            lineHolder[cell] = 1;
            const openIndex = openPositions[cell];
            const lastCell = openCells[openCells.length - 1];
            openCells[openIndex] = lastCell;
            openPositions[lastCell] = openIndex;
            openCells.pop();
        }

        function randomOpen() {
            if (!openCells.length) return null;
            const cell = openCells[Math.floor(Math.random() * openCells.length)];
            return point(cell % columns, Math.floor(cell / columns));
        }

        function nearbyOpen(origin: Point) {
            for (let radius = 1; radius < total; radius += 1) {
                const candidates: Point[] = [];
                const xs = origin.x - radius;
                const ys = origin.y - radius;
                const xe = origin.x + radius;
                const ye = origin.y + radius;
                for (let x = xs; x < xe; x += 1) {
                    for (let y = ys; y < ye; y += 1) {
                        const candidate = point(x, y);
                        if (isOpen(candidate)) candidates.push(candidate);
                    }
                }
                if (candidates.length) {
                    return candidates[Math.floor(Math.random() * candidates.length)];
                }
            }
            return null;
        }

        function tidyPatterns() {
            patterns.forEach((pattern) => {
                pattern.index = 0;
                pattern.currentLength = 0;
            });
        }

        function setRandomPattern() {
            patternCurrent = halftime
                ? randomPattern
                : patterns[Math.floor(Math.random() * patterns.length)];
            patternCurrent.currentLength = patternCurrent.allowLimit
                ? Math.floor(Math.random() * (patternCurrent.maxLength - 5)) + 5
                : patternCurrent.maxLength;
        }

        function getRandomDirection(origin: Point) {
            if (!patternCurrent) setRandomPattern();
            const active = patternCurrent as Pattern;
            const candidates: Point[] = [];

            if (active.random) {
                [point(-1, 0), point(1, 0), point(0, -1), point(0, 1)].forEach((direction) => {
                    const candidate = point(origin.x + direction.x, origin.y + direction.y);
                    if (isOpen(candidate)) candidates.push(candidate);
                });
            } else {
                const direction = active.indexAscending ? active.coords[active.index] : active.coords[0];
                const candidate = point(origin.x + direction.x, origin.y + direction.y);
                if (isOpen(candidate)) candidates.push(candidate);
            }

            active.index += 1;
            if (active.index === active.currentLength) {
                active.index = 0;
                active.currentLength = 0;
                patternCurrent = null;
                tidyPatterns();
            }

            const draw = candidates.length > 0;
            const next = draw
                ? candidates[Math.floor(Math.random() * candidates.length)]
                : halftime
                  ? nearbyOpen(origin)
                  : randomOpen();
            if (!next) return null;
            occupy(next);
            return { draw, position: next };
        }

        function canvasPoint(value: Point) {
            return point(value.x * stepSize + stepSize / 2, value.y * stepSize + stepSize / 2);
        }

        function drawLine(origin: Point) {
            const next = getRandomDirection(origin);
            if (!next) return false;
            if (next.draw) {
                const from = canvasPoint(origin);
                const to = canvasPoint(next.position);
                context.beginPath();
                context.moveTo(from.x, from.y);
                context.lineTo(to.x, to.y);
                context.stroke();
            }
            position = next.position;
            return true;
        }

        position = point(Math.floor(Math.random() * columns), Math.floor(Math.random() * rows));
        drawLine(position);

        let completionTimer = 0;
        const interval = window.setInterval(() => {
            if (!drawLine(position)) {
                window.clearInterval(interval);
                completionTimer = window.setTimeout(onComplete, 360);
                return;
            }
            counter += 1;
            if (counter > total / 2) halftime = true;
            if (counter >= total - 2 && openCells.length === 0) {
                window.clearInterval(interval);
                completionTimer = window.setTimeout(onComplete, 360);
            }
        }, SPEED);

        return () => {
            window.clearInterval(interval);
            window.clearTimeout(completionTimer);
        };
    }

    onMount(() => {
        let stop = () => {};
        let restartFrame = 0;
        const start = () => {
            stop();
            stop = startDoodler(start);
        };
        const restart = () => {
            window.cancelAnimationFrame(restartFrame);
            restartFrame = window.requestAnimationFrame(() => {
                start();
            });
        };
        const observer = new MutationObserver(() => {
            restart();
        });
        const resizeObserver = new ResizeObserver(restart);
        observer.observe(document.documentElement, {
            attributes: true,
            attributeFilter: ["data-mode"],
        });
        resizeObserver.observe(canvas);
        start();
        return () => {
            window.cancelAnimationFrame(restartFrame);
            stop();
            observer.disconnect();
            resizeObserver.disconnect();
        };
    });
</script>

<canvas bind:this={canvas} aria-hidden="true"></canvas>

<style>
    canvas {
        position: absolute;
        inset: 0;
        z-index: 0;
        width: 100%;
        height: 100%;
        color: var(--accent);
        opacity: 0.18;
        pointer-events: none;
        mask-image: radial-gradient(
            ellipse 62% 72% at 50% 48%,
            rgba(0, 0, 0, 0.18) 0%,
            rgba(0, 0, 0, 0.24) 48%,
            #000 100%
        );
    }
</style>
