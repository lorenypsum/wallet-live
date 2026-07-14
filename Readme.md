# Wallet Live

Wallet Live é uma aplicação web em Rust para autenticação de usuários e gestão de ativos/investimentos. A interface permite entrar, registrar conta, visualizar um dashboard financeiro, cadastrar e editar posições da carteira e administrar o catálogo de ativos.

## O que o projeto faz

- Autentica usuários com login, registro e logout.
- Exibe uma home pública com atalhos para login e cadastro.
- Mostra um dashboard com métricas da carteira.
- Permite cadastrar e editar investimentos da carteira.
- Permite cadastrar e editar ativos do catálogo pela interface do dashboard.
- Calcula valor investido, valor atual, resultado líquido e percentual de retorno.
- Exibe mensagens de validação e feedback para o usuário.

## Como executar a aplicação

1. Suba o banco PostgreSQL:

```bash
docker compose up -d
```

2. Configure a conexão do banco, se necessário:

```bash
export DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres
```

3. Aplique as migrations:

```bash
cargo sqlx migrate run
```

4. Execute a aplicação:

```bash
cargo run
```

5. Acesse a aplicação em:

```text
http://localhost:3000
```

## Tecnologias usadas

- Rust
- Axum
- Askama
- SQLx
- PostgreSQL
- Tokio
- Tailwind CSS via CDN
- JWT Simple
- password-auth
- Cargo Insta para snapshots de teste

## Qual melhoria foi implementada

Nesta versão, a aplicação foi aproximada do front-end de referência e ganhou um fluxo funcional completo:

- Dashboard redesenhado com métricas financeiras reais.
- Formulário para cadastrar investimento na carteira.
- Formulário para editar investimento existente.
- Formulários para cadastrar e editar ativos do catálogo.
- Cálculo do total investido, valor atual, resultado líquido e retorno percentual.
- Validações para evitar quantidades e valores inválidos.
- Mensagens de erro e sucesso no fluxo de login, registro e dashboard.
- Testes novos para validações e edição de posição.

## Como testar sua versão

```bash
cargo test
```

Se quiser validar apenas a compilação:

```bash
cargo check
```

Se quiser aplicar as migrations antes de testar o fluxo completo:

```bash
cargo sqlx migrate run
```

## Observações

- O dashboard usa a API existente para criar e editar ativos do catálogo.
- O login e o registro agora retornam mensagens de feedback em vez de assumir sucesso silenciosamente.
- O cálculo do histórico de investimento foi ajustado para serializar corretamente o timestamp no payload.
