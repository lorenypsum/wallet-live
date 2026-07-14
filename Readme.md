# Wallet Live

Wallet Live é uma aplicação web em Rust para gestão de carteira de investimentos, facilita a atualização de preços dinamicamente via BRAPI e apresenta métricas da carteira.

## Producao

- URL: https://wallet-live.onrender.com/

## Funcionalidades

- Login, registro e logout.
- Edição de perfil (username e senha).
- Dashboard com métricas da carteira.
- Cadastro, edição e exclusão de posições da carteira.
- Atualização de preço de ativos selecionados via BRAPI.

## Como rodar localmente

### Pré-requisitos

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

### 2) Configurar variáveis de ambiente

Crie um arquivo `.env` na raíz:

```env
DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres
BRAPI_TOKEN=seu_token_brapi
DB_MAX_CONNECTIONS=5
```

Notas:

- `DATABASE_URL` é obrigatória.
- `BRAPI_TOKEN` é opcional (há fallback de desenvolvimento no código).
- `DB_MAX_CONNECTIONS` é opcional (padrão: `5`).

### 3) Rodar migrations

```bash
cargo sqlx migrate run
```

### 4) Iniciar a aplicação

```bash
cargo run
```

Aplicação local:

- http://localhost:3000

### 5) Testes

```bash
cargo test
```

## Deploy no Render

Este repositório já está preparado para deploy com Docker no Render.

### O que já existe no projeto

- `Dockerfile` para build e runtime.
- `.github/workflows/ci.yml` para testes automatizados.
- `.github/workflows/deploy-render.yml` para disparar deploy via Deploy Hook após CI verde na branch `master`.

### Passo a passo

1. Crie um Postgres (Render ou provedor externo).
2. Crie um Web Service no Render conectado a este repositório.
3. Escolha runtime Docker (o Render detecta o `Dockerfile`).
4. Configure as variáveis de ambiente no Web Service:
   - `DATABASE_URL` (obrigatória)
   - `BRAPI_TOKEN` (recomendada) obtida em [BRAPI](https://brapi.dev)
   - `DB_MAX_CONNECTIONS` (opcional, recomendado iniciar com `5`)
5. Em Settings do serviço, copie o `Deploy Hook`.

### Deploy automático via GitHub Actions

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
