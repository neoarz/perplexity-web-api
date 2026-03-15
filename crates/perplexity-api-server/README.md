# perplexity-api-server

HTTP server for the `perplexity-web-api` workspace.

It exposes a small REST API on top of `perplexity-web-client` using Axum.

## Endpoints

- `GET /health`
- `GET /v1/models`
- `POST /v1/attachments`
- `POST /v1/search`
- `POST /v1/search/stream`
- `POST /v1/images`

## Configuration

| Variable | Required | Default | Description |
| --- | --- | --- | --- |
| `PERPLEXITY_SESSION_TOKEN` | yes | none | Perplexity browser session cookie |
| `PERPLEXITY_CSRF_TOKEN` | yes | none | Perplexity browser CSRF cookie |
| `PERPLEXITY_API_KEY` | no | unset | bearer token required for `/v1/*` when set |
| `PERPLEXITY_SEARCH_MODEL` | no | `sonar` | default model for `mode = "search"` |
| `PERPLEXITY_REASON_MODEL` | no | `sonar-reasoning` | default model for `mode = "reason"` |
| `HOST` | no | `127.0.0.1` | bind address |
| `PORT` | no | `3000` | bind port |
| `SEARCH_TIMEOUT_SECS` | no | `30` | timeout for `search` requests |
| `REASON_TIMEOUT_SECS` | no | `120` | timeout for `reason` requests |
| `RESEARCH_TIMEOUT_SECS` | no | `300` | timeout for `research` requests |
| `LOG_FILE` | no | unset | optional log file path |

If `PERPLEXITY_API_KEY` is set, requests to `/v1/*` must send:

```http
Authorization: Bearer <your-api-key>
```

## Run

From the workspace root:

```bash
cargo run -p perplexity-api-server
```

## Text requests

### Request shape

```json
{
  "query": "What is Rust?",
  "mode": "search",
  "model": "sonar",
  "sources": ["web"],
  "language": "en-US",
  "incognito": true,
  "attachments": [],
  "follow_up": {
    "backend_uuid": null,
    "attachments": []
  }
}
```

### Request fields

| Field | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| `query` | `string` | yes | none | prompt or question sent to Perplexity |
| `mode` | `string` | no | `search` | request mode |
| `model` | `string` | no | server default | model override for the selected mode |
| `sources` | `string[]` | no | `["web"]` | source filters |
| `language` | `string` | no | `en-US` | language sent upstream |
| `incognito` | `bool` | no | `true` | whether to run the request in incognito mode |
| `attachments` | `string[]` | no | `[]` | uploaded attachment URLs for first-turn image analysis |
| `follow_up` | `object` | no | `null` | conversation state from a previous response |

### Follow-up fields

| Field | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| `follow_up.backend_uuid` | `string \| null` | no | `null` | conversation id returned by the previous response |
| `follow_up.attachments` | `string[]` | no | `[]` | attachment URLs that should carry into the next turn |

### Attachment behavior

| Behavior | Value |
| --- | --- |
| top-level `attachments` | supported only in `search` mode |
| `reason` or `research` with top-level `attachments` | rejected |
| plain public image URLs | not a supported substitute for uploaded attachments |
| canonical next-turn attachments | use `follow_up.attachments` from the previous response |

### Modes

| Mode | Description | Model behavior |
| --- | --- | --- |
| `search` | standard search mode | accepts search models |
| `reason` | reasoning mode | accepts reasoning models |
| `research` | deep research mode | fixed to `pplx_alpha`, does not accept `model` |

### Sources

| Value | Description |
| --- | --- |
| `web` | general web results |
| `scholar` | academic and paper-heavy sources |
| `social` | social platforms and community content |

### Models

| Search | Reasoning |
| --- | --- |
| `turbo` | `sonar-reasoning` |
| `sonar` | `gemini-3-flash-thinking` |
| `sonar-pro` | `gemini-3.1-pro` |
| `gemini-3-flash` | `gpt-5.4-thinking` |
| `gpt-5.4` | `gpt-5.2-thinking` |
| `gpt-5.2` | `claude-4.6-sonnet-thinking` |
| `claude-4.6-sonnet` | `grok-4.1-reasoning` |
| `grok-4.1` | `kimi-k2.5-thinking` |
|  | `nemotron-3-super-thinking` |

Use `GET /v1/models` to discover the current defaults and supported names at runtime.

### Example

```bash
curl -sS -X POST http://127.0.0.1:3000/v1/search \
  -H 'Authorization: Bearer YOUR_API_KEY' \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "What is Rust?"
  }'
```

## Attachments

`POST /v1/attachments` uploads image files through Perplexity's real attachment pipeline and returns attachment URLs you can send to `POST /v1/search` or `POST /v1/search/stream`.

### Request shape

Send `multipart/form-data` with repeated `files` fields.

### Request fields

| Field | Type | Required | Description |
| --- | --- | --- | --- |
| `files` | multipart file field | yes | repeated image files to upload |

### Upload rules

| Behavior | Value |
| --- | --- |
| accepted file types | `image/*` only |
| repeated fields | use repeated `files` parts |
| file order | preserved |
| MIME detection | server validates the actual file bytes and normalizes the upload MIME type |
| empty files | rejected |
| unsupported MIME types | rejected |

### Response shape

```json
{
  "attachments": [
    {
      "url": "https://ppl-ai-file-upload.s3.amazonaws.com/...",
      "file_uuid": "uuid",
      "filename": "example.png",
      "content_type": "image/png",
      "size_bytes": 12345
    }
  ]
}
```

### Response fields

| Field | Type | Description |
| --- | --- | --- |
| `attachments` | `object[]` | uploaded attachments in request order |
| `attachments[].url` | `string` | attachment URL to send in `search.attachments` |
| `attachments[].file_uuid` | `string` | upstream file id for the uploaded asset |
| `attachments[].filename` | `string` | original filename |
| `attachments[].content_type` | `string` | uploaded MIME type |
| `attachments[].size_bytes` | `number` | uploaded file size in bytes |

### Upload example

```bash
curl -sS -X POST http://127.0.0.1:3000/v1/attachments \
  -H 'Authorization: Bearer YOUR_API_KEY' \
  -F 'files=@/absolute/path/to/example.png'
```

### Search with attachment example

```bash
curl -sS -X POST http://127.0.0.1:3000/v1/search \
  -H 'Authorization: Bearer YOUR_API_KEY' \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "Analyze the attached image",
    "mode": "search",
    "model": "sonar",
    "attachments": [
      "https://ppl-ai-file-upload.s3.amazonaws.com/..."
    ]
  }'
```

## Image generation

`POST /v1/images` generates images through the same upstream Perplexity ask flow used by search, then returns the generated image assets.

### Request shape

```json
{
  "prompt": "Generate an image of a cinematic red fox in neon rain",
  "model": "sonar",
  "language": "en-US",
  "incognito": true
}
```

### Request fields

| Field | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| `prompt` | `string` | yes | none | image generation prompt sent to Perplexity |
| `model` | `string` | no | server default search model | optional search-model override |
| `language` | `string` | no | `en-US` | language sent upstream |
| `incognito` | `bool` | no | `true` | whether to run the request in incognito mode |

`/v1/images` is generation-only in this release. It does not accept uploaded input images or image-edit requests.

### Model behavior

| Behavior | Value |
| --- | --- |
| accepted models | search models only |
| omitted `model` | uses the configured default search model |
| reasoning models | rejected |
| explicit upstream image model selector | not exposed |

### Search models

| Model | Result |
| --- | --- |
| `turbo` | works |
| `sonar` | works |
| `sonar-pro` | works |
| `gemini-3-flash` | works |
| `gpt-5.4` | works |
| `gpt-5.2` | works |
| `claude-4.6-sonnet` | works |
| `grok-4.1` | works |

### Observed behavior

| Behavior | Value |
| --- | --- |
| omitted `model` | uses `sonar` by default |
| successful `generation_model` | `seedream` |
| successful `source` | `seedream-router` |

### Response shape

```json
{
  "id": "req_...",
  "model": "sonar",
  "prompt": "Generate an image of a cinematic red fox in neon rain",
  "image_generation": true,
  "images": [
    {
      "url": "https://...",
      "thumbnail_url": "https://...",
      "download_url": "https://...",
      "mime_type": "image/png",
      "source": "seedream-router",
      "generation_model": "seedream",
      "prompt": "Cinematic red fox in neon rain"
    }
  ],
  "answer": "Media generated: '...'",
  "follow_up": {
    "backend_uuid": "backend-uuid",
    "attachments": []
  }
}
```

### Response fields

| Field | Type | Description |
| --- | --- | --- |
| `id` | `string` | server-generated request id |
| `model` | `string` | search model used for the request |
| `prompt` | `string` | original generation prompt |
| `image_generation` | `bool` | whether the upstream request was classified as image generation |
| `images` | `object[]` | generated image assets |
| `answer` | `string \| null` | upstream text answer, when present |
| `follow_up` | `object` | follow-up values returned by Perplexity |

### Generated image fields

| Field | Type | Description |
| --- | --- | --- |
| `url` | `string` | direct image URL |
| `thumbnail_url` | `string \| null` | thumbnail URL, when present |
| `download_url` | `string \| null` | download URL, when present |
| `mime_type` | `string \| null` | MIME type, when present |
| `source` | `string \| null` | upstream router or image source |
| `generation_model` | `string \| null` | upstream generation model name |
| `prompt` | `string \| null` | prompt-like description attached to the asset |

### Example

```bash
curl -sS -X POST http://127.0.0.1:3000/v1/images \
  -H 'Authorization: Bearer YOUR_API_KEY' \
  -H 'Content-Type: application/json' \
  -d '{
    "prompt": "Generate an image of a cinematic red fox in neon rain",
    "model": "sonar"
  }'
```

## Notes

- `PERPLEXITY_API_KEY` is optional. If it is unset, auth is disabled.
- `GET /v1/models` is the source of truth for supported model names.
- `POST /v1/search/stream?human=1` is intended for terminal use.
