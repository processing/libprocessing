// Mouse 2D.
//
// Moving the mouse changes the position and size of each box.
function setup() {
    size(640, 360);
    noStroke();
    rectMode(CENTER);
}

function draw() {
    background(0.2);
    fill(1.0, 0.8);
    rect(mouseX, height / 2, mouseY / 2 + 10, mouseY / 2 + 10);
    fill(1.0, 0.8);
    const inverseX = width - mouseX;
    const inverseY = height - mouseY;
    rect(inverseX, height / 2, inverseY / 2 + 10, inverseY / 2 + 10);
}
