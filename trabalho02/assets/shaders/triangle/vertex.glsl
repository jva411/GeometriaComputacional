#version 330 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;
// layout (location = 2) in vec2 aTextureCoords;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform bool uSimplex;

// out vec2 textureCoords;
out vec3 normal;
out vec3 fragmentPosition;
out vec3 vertexColor;

void main() {
  vec4 worldPos = model * vec4(aPos, 1.0);
  gl_Position = projection * view * worldPos;
  fragmentPosition = vec3(worldPos);
  // textureCoords = aTextureCoords;
  normal = normalize(mat3(transpose(inverse(model))) * aNormal);

  vertexColor = vec3(1.0, 1.0, 1.0);
  if (uSimplex) {
    gl_PointSize = 10.0;
    vertexColor = aNormal;
  }
}
