#version 330 core

#define MAX_LIGHTS 16
#define SPECULAR_FUNCTION CalcBlinnPhongSpecular

struct Material {
  vec3 ambient;
  vec3 diffuse;
  vec3 specular;
  float shininess;
};

struct PointLight {
  vec3 position;

  vec3 ambient;
  vec3 diffuse;
  vec3 specular;
};

in vec3 normal;
in vec3 fragmentPosition;

uniform Material material;
uniform PointLight[MAX_LIGHTS] lights;
uniform int n_lights;
uniform vec3 cameraPosition;

// Color override
uniform bool uUseOverrideColor;
uniform vec3 uOverrideColor;

out vec4 FragColor;

vec3 CalcPointLight(PointLight light, vec3 cameraDirection);
float CalcPhongSpecular(vec3 lightDirection, vec3 cameraDirection);
float CalcBlinnPhongSpecular(vec3 lightDirection, vec3 cameraDirection);

void main() {
  vec3 cameraDirection = normalize(cameraPosition - fragmentPosition);

  if (uUseOverrideColor) {
    FragColor = vec4(uOverrideColor, 1.0);
  } else {
    vec3 finalColor = vec3(0.0, 0.0, 0.0);
    for (int i=0; i<n_lights; i++) {
      finalColor += CalcPointLight(lights[i], cameraDirection);
    }

    FragColor = vec4(finalColor, 1.0);
  }
}


vec3 CalcPointLight(PointLight light, vec3 cameraDirection) {
  vec3 ambientLight = light.ambient * material.ambient;

  vec3 lightDirection = normalize(light.position - fragmentPosition);
  float diffuse = max(dot(normal, lightDirection), 0.0);
  float distance = length(light.position - fragmentPosition);
  vec3 diffuseLight = light.diffuse * material.diffuse * (4 * diffuse / (distance * distance));

  vec3 specularLight = vec3(0.0, 0.0, 0.0);
  if (diffuse > 0.000001) {
    float specular = SPECULAR_FUNCTION(lightDirection, cameraDirection);
    specularLight = light.specular * specular * material.specular;
  }

  return (ambientLight + diffuseLight + specularLight);
}

float CalcPhongSpecular(vec3 lightDirection, vec3 cameraDirection) {
  vec3 reflectedLightDirection = reflect(-lightDirection, normal);
  float specular = pow(max(dot(cameraDirection, reflectedLightDirection), 0.0), material.shininess);
  return specular;
}

float CalcBlinnPhongSpecular(vec3 lightDirection, vec3 cameraDirection) {
  vec3 halfwayDirection = normalize(lightDirection + cameraDirection);
  float specular = pow(max(dot(normal, halfwayDirection), 0.0), material.shininess * 4.0);
  return specular;
}
