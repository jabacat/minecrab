#version 330 core

in vec2 fragTexCoord;
in vec3 fragVertexNormal;

// Produce a fragment color.
out vec4 fragColor;

uniform sampler2D tex;

void main() {
    fragColor = texture(tex, fragTexCoord);
    float up = dot(fragVertexNormal, vec3(0.0, 1.0, 0.0));
    float bottom_face_bright = 0.6;
    float side_face_bright = 0.75;
    // Bottom face: make darker
    if (up < -0.1) {
        fragColor *= vec4(bottom_face_bright, bottom_face_bright, bottom_face_bright, 1.0);
    }
    // Side faces: make somewhat darker
    else if (up < 0.1) {
        fragColor *= vec4(side_face_bright, side_face_bright, side_face_bright, 1.0);
    }
}
