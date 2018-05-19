#version 150 core

in vec3 v_Colour;
out vec4 Target0;

void main() {
    Target0 = vec4(v_Colour, 1);
}
