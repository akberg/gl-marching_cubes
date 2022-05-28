#version 460 core

in vec3 v_position;
in vec3 v_normal;
uniform vec4 u_color;

out vec4 color;

void main()
{
    vec3 light_dir = normalize(vec3(1.0, -2.0, 1.0));
    float diffuse = max(dot(normalize(v_normal), normalize(light_dir)), 0.0);
    color = vec4(u_color.xyz * 0.05 + u_color.xyz * diffuse, 1.0); //vec4(1.0f, 0.0f, 1.0f, 1.0f);
}