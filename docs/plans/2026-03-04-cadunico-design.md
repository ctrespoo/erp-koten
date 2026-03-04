# Cadastro Unico Design

## Context

O repositório está no bootstrap inicial, sem estrutura de Axum, Askama, HTMX, assets ou módulos de domínio. O primeiro corte do módulo `Cadastro Unico` deve criar a experiência da tela em `/cadunico/criar`, sem persistência real neste momento.

## Objetivo

Entregar um formulário completo para o schema `CadUnico`, dividido em tabs para não ultrapassar a viewport, com navegação integral por teclado, máscaras visuais no frontend, envio por atalho `Ctrl+S`, exibição de erros de backend em modal e redirecionamento para `/cadunico` em caso de sucesso.

## Decisões Confirmadas

- Todos os campos do schema entram já na primeira versão.
- Não haverá gravação em Postgres neste corte.
- Campos como `cpf_cnpj`, `cep` e telefones terão máscara visual no frontend e envio normalizado.
- O sucesso do envio redireciona para `/cadunico`.
- Erros vindos do backend aparecem em modal e fecham com `Esc`.

## Abordagem Escolhida

Foi escolhida uma página server-rendered única com Askama e um módulo JavaScript pequeno para controlar tabs, atalhos, foco, máscaras e modal. Essa abordagem mantém o stack coerente com Axum + Askama + HTMX, evita fragmentação de estado e permite controle fino da navegação por teclado sem depender do comportamento padrão do navegador.

## Rotas e Fluxo

- `GET /cadunico`
  - Renderiza uma listagem inicial simples, servindo como destino do fluxo de sucesso.
- `GET /cadunico/criar`
  - Renderiza a página completa do formulário.
- `POST /cadunico`
  - Recebe o formulário normalizado.
  - Neste primeiro corte, valida o payload e responde:
    - sucesso: redirecionamento para `/cadunico`
    - erro: resposta estruturada para abrir o modal de erro

O formulário será único, cobrindo todas as tabs. As tabs serão apenas uma divisão visual e de foco.

## Organização das Tabs

### Dados principais

- `cpf_cnpj`
- `inscricao_estadual`
- `inscricao_municipal`
- `fantasia`
- `inss`
- `crea`
- `email`
- `telefones`
- `aniversario`
- `id_estrangeiro`
- `codigo_pais`

### Endereço

- `cep`
- `endereco`
- `numero`
- `complemento`
- `bairro`
- `cidade`
- `uf`
- `codigo_ibge`

### Parâmetros

- `enviar_nfe`
- `enviar_boleto`
- `enviar_extrato`
- `etiqueta`
- `comissao`
- `construcao_civil`

### Cobrança

- `cep_cobranca`
- `endereco_cobranca`
- `numero_cobranca`
- `complemento_cobranca`
- `bairro_cobranca`
- `cidade_cobranca`
- `uf_cobranca`
- `codigo_ibge_cobranca`
- `referencia_cobranca`

## Direção Visual

O frontend seguirá uma linha minimalista e funcional de uso interno:

- fundo claro quente
- contraste alto
- tipografia séria e legível
- blocos compactos
- foco visual forte nos estados de navegação
- quase nenhum ornamento decorativo

O diferencial da interface será a sensação de controle operacional: tabs claras, foco sempre evidente, instruções curtas de atalhos e comportamento previsível.

## Navegação por Teclado

- `ArrowDown` e `Tab` avançam para o próximo campo navegável da aba atual
- `ArrowUp` e `Shift+Tab` voltam para o campo anterior
- `Ctrl+ArrowRight` e `Ctrl+ArrowLeft` mudam entre tabs
- ao trocar de tab, o foco vai para o primeiro campo navegável da nova aba
- `Ctrl+S` impede o comportamento padrão do navegador e dispara o submit
- quando o modal estiver aberto, ele captura o foco
- `Esc` fecha o modal e devolve o foco ao último elemento ativo

A ordem de foco será controlada explicitamente por script, sem depender apenas da navegação nativa do navegador.

## Máscaras e Normalização

- `cpf_cnpj`: máscara adaptativa de CPF/CNPJ, envio apenas com dígitos
- `cep` e `cep_cobranca`: máscara de CEP, envio apenas com dígitos
- `uf` e `uf_cobranca`: normalização para maiúsculas com limite de 2 caracteres
- `codigo_ibge` e `codigo_ibge_cobranca`: entrada numérica com limite de 7 dígitos
- `aniversario`: valor ISO no envio
- `telefones`: máscara visual brasileira com envio normalizado

## Validação e Erros

Neste primeiro corte:

- o frontend valida obrigatórios e formatos básicos para reduzir erro evitável
- o backend também valida, sendo a fonte final do contrato
- erros de backend não aparecem inline
- qualquer erro vindo do backend abre o modal

O modal terá título curto, corpo simples, foco inicial no próprio diálogo e fechamento por `Esc`.

## Estrutura Técnica Proposta

### Rust

- `src/main.rs`: bootstrap da aplicação
- `src/app.rs`: montagem do `Router`
- `src/modules/cadunico/mod.rs`
- `src/modules/cadunico/routes.rs`
- `src/modules/cadunico/forms.rs`
- `src/modules/cadunico/templates.rs`
- `src/modules/cadunico/service.rs`
- `src/modules/cadunico/errors.rs`

### Templates

- `templates/layouts/app.html`
- `templates/cadunico/index.html`
- `templates/cadunico/create.html`
- fragmento opcional para modal de erro

### Assets

- `assets/styles/app.css`
- `assets/scripts/cadunico-form.js`

## Testes Esperados

- testes unitários de normalização de entrada
- testes unitários de validação do formulário
- testes de integração para `GET /cadunico`, `GET /cadunico/criar` e `POST /cadunico`
- testes do comportamento central do JavaScript de navegação e atalhos

## Fora de Escopo Neste Corte

- persistência com SQLX/Postgres
- migrations
- listagem real com dados do banco
- busca de CEP, IBGE ou integrações externas
- regras avançadas de negócio além da validação básica do payload
