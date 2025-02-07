#version 300 es
in vec2 xy;
uniform mat4 mvp;
out vec2 uv;

void main()
{
    gl_Position = mvp*vec4(xy,0.0, 1.0);
    uv = (xy+1.0)*0.5;
    //uv = vec2(0.5,0);
}
