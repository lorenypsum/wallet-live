# Wallet Live

Wallet Live e uma aplicacao web em Rust para autenticacao de usuarios e gestao de investimentos.

## Producao

- URL: https://wallet-live.onrender.com/

## Funcionalidades

- Login, registro e logout.
- Edicao de perfil (username e senha).
- Dashboard com metricas da carteira.
- Cadastro, edicao e exclusao de posicoes da carteira.
- Atualizacao de preco de ativos selecionados via BRAPI.

## Como rodar localmente

### Pre-requisitos

- Rust toolchain instalado.
- Docker e Docker Compose instalados.
- SQLx CLI instalado:

```bash
cargo install sqlx-cli --no-default-features --features rustls,postgres
```

### 1) Subir o banco local

```bash
docker compose up -d
```

O PostgreSQL local sobe com:

- host: `localhost`
- porta: `5432`
- usuario: `postgres`
- senha: `postgres`
- database: `postgres`

### 2) Configurar variaveis de ambiente

Crie um arquivo `.env` na raiz:

```env
DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres
BRAPI_TOKEN=seu_token_brapi
DB_MAX_CONNECTIONS=5
```

Notas:

- `DATABASE_URL` e obrigatoria.
- `BRAPI_TOKEN` e opcional (ha fallback de desenvolvimento no codigo).
- `DB_MAX_CONNECTIONS` e opcional (padrao: `5`).

### 3) Rodar migrations

```bash
cargo sqlx migrate run
```

### 4) Iniciar a aplicacao

```bash
cargo run
```

Aplicacao local:

- http://localhost:3000

### 5) Testes

```bash
cargo test
```

## Deploy no Render

Este repositorio ja esta preparado para deploy com Docker no Render.

### O que ja existe no projeto

- `Dockerfile` para build e runtime.
- `.github/workflows/ci.yml` para testes automatizados.
- `.github/workflows/deploy-render.yml` para disparar deploy via Deploy Hook apos CI verde na branch `master`.

### Passo a passo

1. Crie um Postgres (Render ou provedor externo).
2. Crie um Web Service no Render conectado a este repositorio.
3. Escolha runtime Docker (o Render detecta o `Dockerfile`).
4. Configure as variaveis de ambiente no Web Service:
   - `DATABASE_URL` (obrigatoria)
   - `BRAPI_TOKEN` (recomendada) obtida em [BRAPI](https://brapi.dev)
   - `DB_MAX_CONNECTIONS` (opcional, recomendado iniciar com `5`)
5. Em Settings do servico, copie o `Deploy Hook`.

### Deploy automatico via GitHub Actions

1. No GitHub: Settings > Secrets and variables > Actions.
2. Crie o secret `RENDER_DEPLOY_HOOK_URL` com o valor do hook do Render.
3. Fluxo de deploy:
   - Push/PR roda CI (`cargo test`).
   - Quando CI em `master` conclui com sucesso, o workflow de deploy chama o hook do Render.

## Workflows

- `.github/workflows/ci.yml`: executa `cargo test` com `SQLX_OFFLINE=true`.
- `.github/workflows/deploy-render.yml`: faz `POST` no Deploy Hook do Render quando a CI da `master` passa.

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
