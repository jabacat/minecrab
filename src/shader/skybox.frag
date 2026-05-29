#version 330 core

in vec2 fragTexCoord;
in vec3 fragVertexNormal;

// Produce a fragment color.
out vec4 fragColor;

uniform sampler2D tex;
uniform float dayAmount;

const vec4 topDayColor = vec4(0.5, 0.8, 1.0, 1.0);
const vec4 bottomDayColor = vec4(0.9, 0.9, 0.9, 1.0);
const vec4 nightColor = vec4(0.1, 0.1, 0.1, 1.0);

void main() {
    float up = dot(fragVertexNormal, vec3(0.0, -1.0, 0.0));
    vec4 topColor = dayAmount * topDayColor + (1.0 - dayAmount) * nightColor;
    vec4 bottomColor = dayAmount * bottomDayColor + (1.0 - dayAmount) * nightColor;
    // Top and bottom faces
    if (up > 0.9) {
        fragColor = topColor;
    } else if (up < -0.9) {
        fragColor = bottomColor;
    } else {
        // Otherwise, we use the texture coords to determine where we are.
        float amountUp = fragTexCoord[1];
        fragColor = amountUp * topColor + (1 - amountUp) * bottomColor;
    }
}
