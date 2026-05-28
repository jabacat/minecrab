#version 330 core

in vec2 fragTexCoord;
in vec3 fragVertexNormal;

// Produce a fragment color.
out vec4 fragColor;

uniform sampler2D tex;

const vec4 dayColor = vec4(0.5, 0.8, 1.0, 1.0);
const vec4 nightColor = vec4(0.1, 0.1, 0.1, 1.0);

void main() {
    fragColor = nightColor;
}
