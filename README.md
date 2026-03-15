# perplexity-web-api

Self-hosted REST API and Rust client for Perplexity's web app endpoints.

This workspace contains two crates:

- `perplexity-api-server`: an Axum server that exposes a simple HTTP API
- `perplexity-web-client`: a typed Rust client for the upstream Perplexity web flow

It uses Perplexity web session cookies. It does not use the official Perplexity API.

## Workspace

```text
crates/
├── perplexity-api-server
└── perplexity-web-client
```

- [perplexity-api-server](./crates/perplexity-api-server/README.md)
- [perplexity-web-client](./crates/perplexity-web-client/README.md)

## Quick start

### 1. Get your Perplexity cookies

1. Download this [chrome web extension](https://chromewebstore.google.com/detail/get-cookiestxt-locally/cclelndahbckbenkjhflpdbgdldlbecc)
2. Open [perplexity.com](https://perplexity.com) in your browser.
3. Log in if needed, and ask a question to generate a conversation.
4. Click the extension icon and select "Export cookies".
5. Open the exported file and copy the values of `__Secure-next-auth.session-token` and `next-auth.csrf-token`.

### 2. Configure environment

Copy the example file and set your Perplexity cookies:

```bash
cp .env.example .env
```

Set the following required values:

- `PERPLEXITY_SESSION_TOKEN`
- `PERPLEXITY_CSRF_TOKEN`

For the full environment reference, request fields, modes, and models, see [crates/perplexity-api-server/README.md](./crates/perplexity-api-server/README.md).

### 3. Run the server

With Cargo:

```bash
cargo run -p perplexity-api-server
```

With Docker Compose:

```bash
docker compose up -d --build
```

By default the server listens on `127.0.0.1:3000`. In Docker, Compose publishes the container port to the host port configured in [`compose.yaml`](./compose.yaml).

## HTTP API

The full API reference, including request fields, modes, models, and environment variables, lives in [crates/perplexity-api-server/README.md](./crates/perplexity-api-server/README.md).

Available endpoints:

- `GET /health`
- `GET /v1/models`
- `POST /v1/search`
- `POST /v1/search/stream`
- `POST /v1/images`
- `POST /v1/attachments`

### Authentication

`PERPLEXITY_API_KEY` is optional. It is only used to provide an API key for the server you are going to run. If it is set, send:

```http
Authorization: Bearer <your-api-key>
```

If `PERPLEXITY_API_KEY` is unset, the API is open.

## Examples

### List models

```bash
curl -sS http://127.0.0.1:3000/v1/models \
  -H 'Authorization: Bearer YOUR_API_KEY'
```

### Search

```bash
curl -sS -X POST http://127.0.0.1:3000/v1/search \
  -H 'Authorization: Bearer YOUR_API_KEY' \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "What is Rust?"
  }'
```

### Reasoning

```bash
curl -sS -X POST http://127.0.0.1:3000/v1/search \
  -H 'Authorization: Bearer YOUR_API_KEY' \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "If all cats are animals and all animals are living things, are all cats living things?",
    "mode": "reason",
    "model": "claude-4.6-sonnet-thinking"
  }'
```

### Streaming

```bash
curl -sN -X POST 'http://127.0.0.1:3000/v1/search/stream?human=1' \
  -H 'Authorization: Bearer YOUR_API_KEY' \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "Name 5 programming languages"
  }'
```

## Image generation

```bash
curl -sS -X POST http://127.0.0.1:3000/v1/images \
  -H 'Authorization: Bearer YOUR_API_KEY' \
  -H 'Content-Type: application/json' \
  -d '{
    "prompt": "Generate an image of a cinematic red fox in neon rain",
    "model": "sonar"
  }'
```

### Attachments

```bash
curl -sS -X POST http://127.0.0.1:3000/v1/attachments \
  -H 'Authorization: Bearer YOUR_API_KEY' \
  -F 'files=@/absolute/path/to/example.png'
```

### Follow-up

Pass the `backend_uuid` and `attachments` returned from the previous response:

```bash
curl -sS -X POST http://127.0.0.1:3000/v1/search \
  -H 'Authorization: Bearer YOUR_API_KEY' \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "Tell me more",
    "follow_up": {
      "backend_uuid": "your-backend-uuid",
      "attachments": []
    }
  }'
```

## More documentation

- server configuration and API reference: [crates/perplexity-api-server/README.md](./crates/perplexity-api-server/README.md)
- Rust client usage: [crates/perplexity-web-client/README.md](./crates/perplexity-web-client/README.md)
