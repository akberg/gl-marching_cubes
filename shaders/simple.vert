#version 460 core

in vec3 position;
in vec3 normal;
out vec3 v_position;
out vec3 v_normal;
uniform mat4 u_mvp;
uniform float u_aspect;

void main()
{
    v_position = position;
    v_normal = normal;
    gl_Position = u_mvp * vec4(position, 1.0f);
}