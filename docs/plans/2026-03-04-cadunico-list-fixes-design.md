# CadUnico List Fixes Design

## Goal

Corrigir os problemas funcionais e de interação em `/cadunico` sem mudar a linguagem visual da tela:

- a lista deve carregar registros logo na entrada da página
- a navegação por teclado deve continuar funcionando
- o menu de ações deve abrir como overlay flutuante
- a exclusão deve funcionar após swaps HTMX
- a paginação deve ser fixa em 8 itens por página para manter a tela estável

## Problem Summary

Os sintomas observados têm duas causas estruturais principais:

1. A rota inicial `/cadunico` renderiza a página com `CadUnicoListPageView::empty()`, enquanto a busca e a paginação usam `/cadunico/lista`, que consulta o banco.
2. Elementos interativos com estado próprio, como o dialog de exclusão, estão dentro do fragmento HTMX substituído por `outerHTML`, o que deixa referências stale no JavaScript após cada swap.

Também há dois problemas de UI:

- o destaque de foco da linha é aplicado ao `tr` inteiro e vaza no encontro com as bordas da tabela
- o menu de ações está inline na célula, então expande a row em vez de abrir como dropdown flutuante

## Chosen Approach

Seguir uma abordagem server-first:

- `GET /cadunico` passa a carregar a primeira página real da lista
- `GET /cadunico/lista` continua sendo a fonte do fragmento HTMX
- o contrato de listagem é unificado para carregamento inicial, busca, paginação e exclusão
- o dialog de exclusão e o container do menu flutuante passam a existir fora do fragmento trocado por HTMX

Essa abordagem mantém HTML + HTMX como fonte da verdade, reduz a quantidade de estado no frontend e resolve os bugs sem reescrever o fluxo como SPA.

## Architecture

### Backend

- Reaproveitar `CadUnicoService::list` para `index()` e `list_fragment()`.
- Ajustar o default de `page_size` para `8`.
- Preservar `search_value` no `CadUnicoListPageView`.
- Garantir que exclusão, paginação e busca mantenham o mesmo conjunto de query params relevantes.

### Templates

- `templates/cadunico/index.html` continua sendo o shell completo da página.
- `templates/cadunico/_list.html` fica responsável apenas pela região trocável da lista e paginação.
- `templates/cadunico/_delete_dialog.html` passa a ser incluído no shell fixo, fora do fragmento da lista.
- O menu flutuante deve usar um container persistente no root da página para não ser destruído em swaps HTMX.

### Frontend

- O script da lista deve reidratar eventos com base no DOM atual depois de cada swap.
- `Enter` na row ativa abre um menu flutuante ancorado na coluna `Ações`.
- `Escape` fecha menu ou dialog e devolve o foco para a row ativa quando aplicável.
- `Ctrl+N` continua sendo capturado no nível do documento com `preventDefault()` antes da navegação para `/cadunico/criar`.

## Interaction Design

### Initial Load

- Entrar em `/cadunico` deve renderizar registros existentes sem exigir busca manual.
- A primeira página usa paginação fixa de 8 itens.

### Search and Pagination

- A busca continua incremental.
- Quando a busca muda, a lista volta para a primeira página.
- `ArrowLeft` e `ArrowRight` trocam páginas.
- O layout não deve crescer verticalmente além da área prevista da listagem.

### Row Focus

- `ArrowUp` e `ArrowDown` movem o foco entre rows visíveis.
- O destaque visual deve ficar contido à área interna da linha, sem pintar topo e lateral esquerda da tabela.

### Actions Menu

- `Enter` abre um menu flutuante ancorado à row ativa.
- O menu exibe `Editar (em breve)` desabilitado e `Excluir` ativo.
- O menu não pode empurrar o layout da tabela.

### Delete Flow

- Clicar em `Excluir` abre um dialog persistente com o nome do cadastro selecionado.
- Confirmar exclusão faz `DELETE` e atualiza a região da lista mantendo a busca atual.
- O fluxo deve continuar funcionando após qualquer swap HTMX.

## Testing Strategy

### Rust Integration Tests

Adicionar cobertura para:

- `/cadunico` renderizar registros existentes na carga inicial
- o default de paginação respeitar 8 itens
- a busca preservar o valor exibido no shell/fragmento quando necessário

### Frontend Tests

Adicionar cobertura para:

- `Enter` abrir o menu flutuante na row ativa
- `Escape` fechar o menu e restaurar foco
- exclusão continuar funcionando após swap/rebootstrap
- `Ctrl+N` disparar a navegação esperada

## Acceptance Criteria

- Ao abrir `/cadunico`, registros já existentes são exibidos sem interação adicional.
- O destaque da row focada não vaza no topo nem na lateral esquerda.
- `Ctrl+N` navega para `/cadunico/criar`.
- `Enter` abre um menu flutuante correto na coluna `Ações`.
- `Excluir` remove o item e recarrega a lista corretamente.
- Com mais de 8 itens, a próxima página concentra os itens excedentes sem estourar a tela.

## Non-Goals

- Implementar edição de cadastro.
- Redesenhar visualmente a página fora do necessário para corrigir foco, menu e estabilidade do layout.
- Migrar a tela para um frontend stateful mais pesado.
