#version 330 core

// This adapted from "advanced shader debugging with raylib"

in vec3 vertexPosition;
in vec2 vertexTexCoord;            
in vec4 vertexColor;
in vec3 vertexNormal;
out vec2 fragTexCoord;   
out vec3 fragVertexNormal;          
// out vec4 fragColor;                
uniform mat4 mvp;                  

void main() {             
    fragTexCoord = vertexTexCoord; 
    fragVertexNormal = vertexNormal;
    // fragColor = vertexColor;       
    gl_Position = mvp*vec4(vertexPosition, 1.0); 
}