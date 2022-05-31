#version 460 core

in vec3 v_position;
in vec3 v_normal;
uniform vec4 u_color;
uniform mat4 u_model;
uniform mat4 u_view;
uniform mat4 u_mvp;

out vec4 color;

void main()
{
    vec4 col = u_color;
    vec3 light_dir = normalize(vec3(1.0, 2.0, 1.0));
    vec3 normal = transpose(inverse(mat3(u_model))) * v_normal;
    //vec3 normal = v_normal;
    // if (dot(normal, vec3(0.0, 0.0, 1.0)) < 0.0) {
    //     color = vec4(0.0, 0.0, 0.3, 1.0);
    //     return;
    // }
    float diffuse = max(dot(normalize(normal), normalize(light_dir)), 0.0);
    if (!gl_FrontFacing) {
        diffuse = (1.0 - diffuse);
        col.xyz = vec3(0.0, 0.0, 0.8);
    }
    color = vec4(col.xyz * 0.2 + col.xyz * diffuse, 1.0); //vec4(1.0f, 0.0f, 1.0f, 1.0f);
}