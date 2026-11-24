# Filens — MVP

Filens é um explorador de arquivos para o terminal escrito em Rust.

## Objetivo

Navegar por um projeto no terminal e abrir qualquer arquivo de texto local em uma prévia somente leitura, com numeração de linhas e destaque de sintaxe quando disponível.

Para abrir o diretório atual, use `filens .`.

## Escopo

- **Árvore de arquivos à esquerda.** Começa no diretório atual e permite navegar, expandir e recolher pastas pelo teclado.
- **Prévia à direita.** Abre o arquivo selecionado, mostra seu caminho e permite rolagem vertical. Todas as linhas originais têm número, inclusive as vazias.
- **Destaque automático.** Identifica a sintaxe por nome, extensão ou primeira linha e usa todas as gramáticas disponíveis. O MVP deve garantir destaque para Markdown, HTML, CSS, JavaScript, TypeScript, JSX, Python, Rust, Java, C, C++, Go, SQL, Bash, JSON, YAML, TOML e dotenv. Markdown aparece como fonte colorida. Para Vue e Astro, destaque de HTML é suficiente no MVP. Quando houver uma gramática dedicada, ela pode distinguir template, script e estilo. Sem gramática aplicável, o arquivo abre como texto puro numerado.
- **Leitura de texto.** Suporte inicial a UTF-8 e UTF-16 com BOM. Mostra uma mensagem no painel para arquivos binários (como `.class`), codificações não reconhecidas e erros de leitura. A navegação continua disponível.

## Isolamento do destaque

Cada arquivo começa com um estado de destaque novo. Esse estado é preservado entre as linhas do mesmo arquivo e descartado ao trocar de arquivo. Se uma gramática falhar, a prévia daquele arquivo volta para texto puro numerado. O destaque dos outros arquivos continua funcionando.

## Quebra de linha

| Modo | Comportamento |
|---|---|
| Ligada | Ajusta o texto à largura da prévia. Só o primeiro trecho visual de cada linha recebe seu número original. |
| Desligada | Cada linha do arquivo ocupa uma linha na tela. A rolagem horizontal revela o restante e a coluna de números fica fixa. |

O atalho `w` alterna entre os modos. A prévia mantém a mesma linha original no topo e o rodapé indica o modo ativo. O padrão proposto é iniciar com a quebra desligada.

## Controles propostos

| Tecla | Ação |
|---|---|
| `Tab` | Alternar foco entre árvore e prévia. |
| `↑` / `↓` | Navegar na árvore ou rolar a prévia. |
| `Enter` | Abrir o arquivo ou expandir ou recolher a pasta selecionada. |
| `←` / `→` | Rolar a prévia horizontalmente com a quebra desligada. |
| `w` | Alternar a quebra de linha. |
| `q` | Sair. |

## Implementação

- **Ratatui + Crossterm** para layout, foco e eventos do teclado.
- **Syntect** para detectar a sintaxe e gerar trechos coloridos. Aproveitar o catálogo padrão e adicionar gramáticas para os formatos obrigatórios que estiverem faltando. Usar a gramática de HTML para Vue e Astro quando não houver gramática dedicada. Aplicar o destaque sobre as linhas originais antes de distribuí-las na tela, preservando o estado da sintaxe entre linhas.
- **Prévia** com coluna fixa de números e posição vertical ancorada na linha original. Calcular a largura em colunas do terminal para tratar tabs e caracteres Unicode.

## Critérios de aceite

- [ ] Expandir uma pasta e abrir um arquivo local usando apenas o teclado.
- [ ] Abrir Markdown como fonte colorida e verificar que a numeração corresponde ao arquivo.
- [ ] Abrir código de uma linguagem reconhecida e conferir o destaque em um bloco de várias linhas.
- [ ] Abrir um exemplo de cada linguagem ou formato obrigatório e verificar que pelo menos um elemento de sintaxe recebe cor diferente do texto comum.
- [ ] Abrir arquivos Vue e Astro e verificar que as tags HTML recebem destaque.
- [ ] Abrir um arquivo com comentário ou string multilinha sem fechamento e depois abrir um arquivo de outra linguagem. Verificar que o segundo arquivo recebe o mesmo destaque de quando é aberto diretamente.
- [ ] Simular uma falha de gramática e verificar que o arquivo aparece como texto puro numerado e que o próximo arquivo ainda recebe destaque.
- [ ] Abrir um arquivo UTF-16 com BOM e um arquivo de texto sem extensão. Ambos exibem números de linha.
- [ ] Alternar a quebra após rolar a prévia e verificar que a linha original no topo é preservada.
- [ ] Com quebra ligada, reduzir a largura da janela e verificar que os trechos de continuação ficam sem número.
- [ ] Com quebra desligada, rolar horizontalmente e verificar que os números continuam visíveis.
- [ ] Selecionar um binário ou um arquivo ilegível e confirmar que a mensagem aparece sem interromper a navegação.

## Fora do escopo

Edição de arquivos, renderização visual de Markdown ou HTML e desassemblagem de bytecode.
