// A thin Processing-style layer over the raw wasm bindings: global drawing
// functions that act on a default canvas, and a setup()/draw() loop. The
// surface and graphics ids stay internal.
import initWasm, * as p from '../../../target/wasm/processing_wasm.js';

let surface, graphics;
let frameCount = 0;
let maxes = [1, 1, 1, 1]; // channel maximums, mirrors colorMode()
const SRGB = 0;

// color args -> [c1, c2, c3, a], following Processing's 1/2/3/4 arg forms
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

const api = {
    size(width, height) {
        const canvas = document.querySelector('canvas');
        canvas.id ||= 'processing-canvas';
        canvas.width = width;
        canvas.height = height;
        surface = p.surfaceCreateFromCanvas(canvas.id, width, height);
        graphics = p.graphicsCreate(surface, width, height);
        canvas.addEventListener('mousemove', (e) => p.inputMouseMove(surface, e.offsetX, e.offsetY));
        canvas.addEventListener('mousedown', (e) => p.inputMouseButton(surface, e.button, true));
        canvas.addEventListener('mouseup', (e) => p.inputMouseButton(surface, e.button, false));
        Object.defineProperties(globalThis, {
            width: { value: width, configurable: true },
            height: { value: height, configurable: true },
        });
    },
    loadImage: (path) => p.imageLoad(path),

    colorMode(space, ...max) {
        maxes = [max[0] ?? 1, max[1] ?? max[0] ?? 1, max[2] ?? max[0] ?? 1, max[3] ?? max[0] ?? 1];
        p.colorMode(graphics, space, ...maxes);
    },
    // background(image) or background(...color)
    background: (...args) =>
        typeof args[0] === 'bigint'
            ? p.backgroundImage(graphics, args[0])
            : p.backgroundColor(graphics, ...color(args), SRGB),
    fill: (...args) => p.setFill(graphics, ...color(args), SRGB),
    stroke: (...args) => p.setStrokeColor(graphics, ...color(args), SRGB),
    noFill: () => p.noFill(graphics),
    noStroke: () => p.noStroke(graphics),
    strokeWeight: (w) => p.setStrokeWeight(graphics, w),
    rectMode: (mode) => p.rectMode(graphics, mode),

    push: () => p.pushMatrix(graphics),
    pop: () => p.popMatrix(graphics),
    translate: (x, y) => p.translate(graphics, x, y),
    rotate: (angle) => p.rotate(graphics, angle),
    scale: (x, y = x) => p.scale(graphics, x, y),

    // rect(x, y, w, h), rect(x, y, w, h, r) or rect(x, y, w, h, tl, tr, br, bl)
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
    mouseX: { get: () => p.mouseX(surface), configurable: true },
    mouseY: { get: () => p.mouseY(surface), configurable: true },
});

// The wasm side has no loop of its own, so we drive frames from JS. Loop state
// mirrors the pyo3 host: noLoop()/loop()/redraw(), plus frameRate() since
// requestAnimationFrame follows the display (60, 120, 144 Hz...).
let looping = true;
let redrawRequested = true; // draw once even if setup() calls noLoop()
let frameInterval = 1000 / 60;
let lastFrame = -Infinity;
let handle;

// one frame: flush input, then begin -> draw -> end
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
    // schedule first so nothing below can silently end the loop
    handle = requestAnimationFrame(tick);
    if (!(looping || redrawRequested) || now - lastFrame < frameInterval - 1) return;
    lastFrame = now;
    redrawRequested = false;
    try {
        step();
    } catch (e) {
        cancelAnimationFrame(handle); // stop on error, like Processing
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
handle = requestAnimationFrame(tick);
