# Wallet Live

Wallet Live é uma aplicação web em Rust para autenticação de usuários e gestão de investimentos. A interface permite login, cadastro, edição de perfil, visualização da carteira e atualização de posições com cotações da BRAPI.

## Funcionalidades

- Login, registro e logout.
- Edição de perfil (username e senha).
- Dashboard com métricas da carteira.
- Cadastro, edição e exclusão de posições da carteira.
- Atualização de preço de ativos selecionados via BRAPI.

## Executar localmente

1. Suba o PostgreSQL local com Docker Compose:

```bash
docker compose up -d
```

1. Configure as variáveis de ambiente no arquivo .env:

```env
DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres
BRAPI_TOKEN=seu_token_brapi
```

Se BRAPI_TOKEN não for informado, a aplicação usa um valor padrão de desenvolvimento.

1. Instale SQLx CLI se ainda não tiver:

```bash
cargo install sqlx-cli --no-default-features --features rustls,postgres
```

1. Rode as migrations:

```bash
cargo sqlx migrate run
```

1. Inicie o app:

```bash
cargo run
```

1. Acesse:

[http://localhost:3000](http://localhost:3000)

## Testes

```bash
cargo test
```

## Deploy em produção (Render)

Este repositório já inclui:

- Dockerfile para build e execução da aplicação.
- Workflow de CI no GitHub Actions.
- Workflow de deploy automático para Render após CI verde na branch master.

### Passo a passo no Render

1. Crie um Web Service no Render conectando este repositório.
2. Selecione Docker como runtime (o Render detecta o Dockerfile).
3. Configure as variáveis de ambiente no serviço:
   - DATABASE_URL
   - BRAPI_TOKEN
4. Configure um banco PostgreSQL (Render Postgres, Neon, Supabase etc.) e use a URL em DATABASE_URL.
5. Em Settings do serviço Render, copie o Deploy Hook URL.

### Configurar deploy automático pelo GitHub Actions

1. No GitHub do repositório, abra Settings > Secrets and variables > Actions.
2. Crie o secret:
   - RENDER_DEPLOY_HOOK_URL
3. A cada push em master:
   - CI roda testes.
   - Se CI passar, o workflow de deploy chama o hook do Render.

## Workflows disponíveis

- .github/workflows/ci.yml
  - Executa cargo test em push e pull request.
- .github/workflows/deploy-render.yml
  - Dispara deploy no Render quando o workflow CI conclui com sucesso na master.

## Tecnologias

- Rust
- Axum
- Askama
- SQLx
- PostgreSQL
- Tokio
- Tailwind CSS (CDN)
- JWT Simple
- password-auth
