// Background Image.
//
// Loads an image, draws it as the background and animates a rounded
// rectangle over it.
let logo;

async function setup() {
    size(600, 600);
    logo = await loadImage('images/logo.png');
}

function draw() {
    background(logo);

    const x = 250 + Math.sin(frameCount * 0.02) * 200;
    fill(0.9, 0.1, 0.3);
    rect(x, 50, 100, 100, 20);
}
