#version 330 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform bool uSimplex = false;
uniform float uPointSize = 10.0;

out vec3 normal;
out vec3 fragmentPosition;
out vec3 vertexColor;

void main() {
  vec4 worldPos = model * vec4(aPos, 1.0);
  gl_Position = projection * view * worldPos;
  fragmentPosition = vec3(worldPos);
  normal = normalize(mat3(transpose(inverse(model))) * aNormal);

  vertexColor = vec3(1.0, 1.0, 1.0);
  if (uSimplex) {
    gl_PointSize = uPointSize;
    vertexColor = aNormal;
  }
}
