#version 300 es
precision highp float;
in vec3 rgb2;
out vec4 color;

void main() {
    color = vec4(rgb2, 1.0);
}
