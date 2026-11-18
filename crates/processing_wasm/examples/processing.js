// Processing-style globals over the wasm bindings. Sketches call rect(),
// fill() etc. directly and never see the surface or graphics ids.
import initWasm, * as p from '../../../target/wasm/processing_wasm.js';

let surface, graphics;
let frameCount = 0;
let maxes = [1, 1, 1, 1];
const SRGB = 0;

function color(args) {
    const [a, b, c, d] = args;
    switch (args.length) {
        case 1: return [a, a, a, maxes[3]];
        case 2: return [a, a, a, b];
        case 3: return [a, b, c, maxes[3]];
        case 4: return [a, b, c, d];
        default: throw new TypeError('expected 1 to 4 color arguments');
    }
}

const rgba = (args) => [...color(args), SRGB];

// browsers also report back/forward buttons, which libprocessing doesn't know
const isProcessingButton = (button) => button <= 2;

const api = {
    size(width, height) {
        // a second call would leak the surface and double up the mouse listeners
        if (surface) throw new Error('size() can only be called once');
        const canvas = document.querySelector('canvas');
        canvas.id ||= 'processing-canvas';
        canvas.width = width;
        canvas.height = height;
        surface = p.surfaceCreateFromCanvas(canvas.id, width, height);
        graphics = p.graphicsCreate(surface, width, height);
        canvas.addEventListener('mousemove', (e) => p.inputMouseMove(surface, e.offsetX, e.offsetY));
        canvas.addEventListener('mousedown', (e) => {
            if (isProcessingButton(e.button)) p.inputMouseButton(surface, e.button, true);
        });
        canvas.addEventListener('mouseup', (e) => {
            if (isProcessingButton(e.button)) p.inputMouseButton(surface, e.button, false);
        });
        Object.defineProperties(globalThis, {
            width: { value: width, configurable: true },
            height: { value: height, configurable: true },
        });
    },
    loadImage: (path) => p.imageLoad(path),

    colorMode(space, ...max) {
        switch (max.length) {
            case 0: break;
            case 1: maxes = [max[0], max[0], max[0], max[0]]; break;
            case 3: maxes = [max[0], max[1], max[2], maxes[3]]; break;
            case 4: maxes = [...max]; break;
            default: throw new TypeError('colorMode() takes 0, 1, 3 or 4 maximums');
        }
        p.colorMode(graphics, space, ...maxes);
    },

    // the user should ideally be exposed to an image object that wraps the bigint
    background: (...args) =>
        typeof args[0] === 'bigint'
            ? p.backgroundImage(graphics, args[0])
            : p.backgroundColor(graphics, ...rgba(args)),
    fill: (...args) => p.setFill(graphics, ...rgba(args)),
    stroke: (...args) => p.setStrokeColor(graphics, ...rgba(args)),
    noFill: () => p.noFill(graphics),
    noStroke: () => p.noStroke(graphics),
    strokeWeight: (w) => p.setStrokeWeight(graphics, w),
    rectMode: (mode) => p.rectMode(graphics, mode),

    push() {
        p.pushStyle(graphics);
        p.pushMatrix(graphics);
    },
    pop() {
        p.popStyle(graphics);
        p.popMatrix(graphics);
    },
    translate: (x, y) => p.translate(graphics, x, y),
    rotate: (angle) => p.rotate(graphics, angle),
    scale: (x, y = x) => p.scale(graphics, x, y),

    rect(x, y, w, h, ...r) {
        const radii = r.length === 1 ? [r[0], r[0], r[0], r[0]] : r.length === 4 ? r : [0, 0, 0, 0];
        p.rect(graphics, x, y, w, h, ...radii);
    },
    ellipse: (x, y, w, h) => p.ellipse(graphics, x, y, w, h),
    circle: (x, y, d) => p.circle(graphics, x, y, d),
    line: (x1, y1, x2, y2) => p.line(graphics, x1, y1, x2, y2),
    triangle: (...v) => p.triangle(graphics, ...v),
    point: (x, y) => p.point(graphics, x, y),
};

Object.assign(globalThis, api, { CORNER: 0, CORNERS: 1, CENTER: 2, RADIUS: 3 });
Object.defineProperties(globalThis, {
    frameCount: { get: () => frameCount, configurable: true },
    mouseX: { get: () => (surface ? p.mouseX(surface) : 0), configurable: true },
    mouseY: { get: () => (surface ? p.mouseY(surface) : 0), configurable: true },
});

// The wasm side has no loop of its own, so we drive frames from JS.
// requestAnimationFrame follows the display's refresh rate, so frameRate() skips ticks.
let looping = true;
let redrawRequested = true; // so we still draw once if setup() calls noLoop()
let frameInterval = 1000 / 60;
let lastFrame = -Infinity;
let handle;

function step() {
    p.inputFlush();
    p.beginDraw(graphics);
    try {
        globalThis.draw?.();
    } finally {
        p.endDraw(graphics);
    }
    frameCount++;
}

function tick(now) {
    handle = requestAnimationFrame(tick);
    if (!(looping || redrawRequested) || now - lastFrame < frameInterval - 1) return;
    lastFrame += frameInterval;
    if (now - lastFrame > frameInterval) lastFrame = now; // if we're too far behind
    redrawRequested = false;
    try {
        step();
    } catch (e) {
        cancelAnimationFrame(handle);
        throw e;
    }
}

Object.assign(globalThis, {
    noLoop: () => { looping = false; },
    loop: () => { looping = true; },
    redraw: () => { redrawRequested = true; },
    frameRate: (fps) => { frameInterval = 1000 / fps; },
});

await initWasm();
await p.init();
await globalThis.setup?.();
if (!graphics) api.size(100, 100);
handle = requestAnimationFrame(tick);
