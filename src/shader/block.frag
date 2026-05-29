#version 330 core

in vec2 fragTexCoord;
in vec3 fragVertexNormal;

// Produce a fragment color.
out vec4 fragColor;

uniform sampler2D tex;

uniform float dayAmount;

void main() {

    fragColor = texture(tex, fragTexCoord);
    float up = dot(fragVertexNormal, vec3(0.0, 1.0, 0.0));
    float east = dot(fragVertexNormal, vec3(1.0, 0.0, 0.0));
    float bottom_face_bright = 0.25;
    float side_face_bright = 0.75;
    // Bottom face: make darker
    if (up < -0.1) {
        fragColor *= vec4(bottom_face_bright, bottom_face_bright, bottom_face_bright, 1.0);
    }
    // Side faces: make somewhat darker
    else if (up < 0.1) {
        float shade = side_face_bright;
        if (east > 0.1 || east < -0.1) {
            shade *= 0.6;
        }
        fragColor *= vec4(shade, shade, shade, 1.0);
    }

    // Things are brighter during the day and darker at night.
    // 1.0 in the day and 0.25 at night.
    float lightModifier = dayAmount * 0.75 + 0.25;
    fragColor *= vec4(lightModifier, lightModifier, lightModifier, 1.0);

}
