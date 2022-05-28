#version 460 core

in vec3 v_position;
in vec3 v_normal;
uniform vec4 u_color;
uniform mat4 u_model;

out vec4 color;

void main()
{
    vec3 light_dir = normalize(vec3(1.0, -2.0, 1.0));
    //vec3 normal = transpose(inverse(mat3(u_model))) * v_normal;
    vec3 normal = v_normal;
    float diffuse = max(dot(normalize(normal), normalize(light_dir)), 0.0);
    color = vec4(u_color.xyz * 0.05 + u_color.xyz * diffuse, 1.0); //vec4(1.0f, 0.0f, 1.0f, 1.0f);
}