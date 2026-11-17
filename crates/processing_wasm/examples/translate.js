// Translate.
//
// The translate() function allows objects to be moved
// to any location within the window. The first parameter
// sets the x-axis offset and the second parameter sets the
// y-axis offset.
let x = 0;
const dim = 80.0;

function setup() {
    size(640, 360);
    noStroke();
}

function draw() {
    background(0.4);

    x = x + 0.8;

    if (x > width + dim) {
        x = -dim;
    }

    translate(x, height / 2 - dim / 2);
    fill(1.0);
    rect(-dim / 2, -dim / 2, dim, dim);

    // Transforms accumulate. Notice how this rect moves
    // twice as fast as the other, but it has the same
    // parameter for the x-axis value
    translate(x, dim);
    fill(0.0);
    rect(-dim / 2, -dim / 2, dim, dim);
}
