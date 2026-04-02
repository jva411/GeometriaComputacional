import json
import glob
import numpy as np
import matplotlib.pyplot as plt

# Usamos o asterisco (*) para lidar com acentos (Ordenação vs Ordenacao) gerados pelo SO
caminho_busca = "target/criterion/Algoritmos de Ordena*/*/*/new/estimates.json"

dados_temp = []
tamanhos_set = set()
cenarios_set = set()
algoritmos_set = set()

# 1. Leitura dos dados JSON
for path in glob.glob(caminho_busca):
    # Divide o caminho para extrair o nome do algoritmo e o tamanho
    partes = path.replace('\\', '/').split('/')
    nome_pasta = partes[-4]
    tamanho = int(partes[-3])

    if " - " in nome_pasta:
      algoritmo, cenario = nome_pasta.split(" - ", 1)
    else:
      algoritmo = nome_pasta
      cenario = "Padrão"

    with open(path, 'r', encoding='utf-8') as f:
      json_data = json.load(f)
      tempo_nanosegundos = json_data['mean']['point_estimate']

    dados_temp.append((algoritmo, cenario, tamanho, tempo_nanosegundos))
    tamanhos_set.add(tamanho)
    cenarios_set.add(cenario)
    algoritmos_set.add(algoritmo)

tamanhos = sorted(list(tamanhos_set))
cenarios = sorted(list(cenarios_set))
algoritmos = sorted(list(algoritmos_set))

row_labels = []
row_mapping = []
for cenario in cenarios:
  for algoritmo in algoritmos:
    cenario_name = cenario
    if cenario_name.startswith("random "):
      cenario_name = cenario_name.replace("random ", "")
    if cenario_name.startswith("with "):
      cenario_name = cenario_name.replace("with ", "")

    row_labels.append(f"{algoritmo}\n({cenario_name})")
    row_mapping.append((cenario, algoritmo))

matriz_tempos = np.full((len(row_labels), len(tamanhos)), np.inf)
for algoritmo, cenario, tamanho, tempo in dados_temp:
  row_idx = row_mapping.index((cenario, algoritmo))
  col_idx = tamanhos.index(tamanho)
  matriz_tempos[row_idx, col_idx] = tempo

melhores_indices = set()
for cenario in cenarios:
  linhas_do_cenario = [i for i, (c, a) in enumerate(row_mapping) if c == cenario]

  if not linhas_do_cenario:
    continue

  for col_idx in range(len(tamanhos)):
    tempos_cenario = matriz_tempos[linhas_do_cenario, col_idx]

    if np.all(np.isinf(tempos_cenario)):
      continue

    idx_menor_local = np.argmin(tempos_cenario)
    idx_menor_global = linhas_do_cenario[idx_menor_local]
    melhores_indices.add((idx_menor_global, col_idx))

def formatar_tempo(nanosegundos: float):
  if nanosegundos < 1000:
    return f"{int(nanosegundos)}ns"

  microsegundos = nanosegundos / 1000
  if microsegundos < 1000:
    return f"{int(microsegundos)}μs"

  milisegundos = microsegundos / 1000
  if milisegundos < 1000:
    return f"{int(milisegundos)}ms"

  segundos = milisegundos / 1000
  if segundos < 60:
    return f"{segundos:.2f}s"

  minutos = segundos / 60
  segundos_restantes = segundos % 60
  return f"{int(minutos)}m{int(segundos_restantes)}s"

textos_tabela = []
for row_idx in range(matriz_tempos.shape[0]):
  linha_texto = []
  for col_idx in range(matriz_tempos.shape[1]):
      val = matriz_tempos[row_idx, col_idx]
      if np.isinf(val):
        linha_texto.append("-")
      else:
        linha_texto.append(formatar_tempo(val))
  textos_tabela.append(linha_texto)

fig, ax = plt.subplots(figsize=(max(12, len(tamanhos) * 1.5), max(5, len(row_labels) * 0.7)))
ax.axis('off')
ax.axis('tight')

tabela = ax.table(
  cellText=textos_tabela,
  rowLabels=row_labels,
  colLabels=[f"N={t}" for t in tamanhos],
  loc='center',
  cellLoc='center',
)

tabela.auto_set_font_size(False)
tabela.set_fontsize(10)
tabela.scale(0.6, 2.2)

# Cor verde pastel para destacar o mais rápido
COR_DESTAQUE = '#d4edda'
# Cinza claro para os cabeçalhos
COR_CABECALHO = '#f2f2f2'
COR_CABECALHO_ALTERNADO = '#c6c6c6'

for key, cell in tabela.get_celld().items():
  row, col = key

  # Se for cabeçalho (linha 0 ou coluna -1 para os rowLabels)
  if row == 0:
    cell.set_text_props(weight='bold')
    cell.set_facecolor(COR_CABECALHO)
  elif col == -1:
    cell.set_text_props(weight='bold')
    cell.set_facecolor(COR_CABECALHO if ((row-1) // len(algoritmos_set)) % 2 == 0 else COR_CABECALHO_ALTERNADO)

  # Se for uma célula de dados que está na nossa lista de vencedores
  elif (row - 1, col) in melhores_indices: # row - 1 porque a linha 0 é o cabeçalho
    cell.set_facecolor(COR_DESTAQUE)
    cell.set_text_props(weight='bold', color='#155724') # Verde escuro no texto

# plt.title("Comparativo de Desempenho: Algoritmos de Ordenação", pad=20, weight='bold', size=16)
plt.tight_layout()

# 6. Salvar como imagem PNG de alta qualidade
nome_arquivo = "tabela_benchmarks.png"
plt.savefig(nome_arquivo, dpi=300, bbox_inches='tight')

print(f"Sucesso! Imagem gerada e salva como '{nome_arquivo}'")
