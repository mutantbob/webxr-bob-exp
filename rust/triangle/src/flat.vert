#version 300 es
in vec2 xy;
in vec3 rgb;
uniform mat4 mvp;
out vec3 rgb2;

void main()
{
    gl_Position = mvp*vec4(xy,0.0, 1.0);
    rgb2 = rgb;
}
