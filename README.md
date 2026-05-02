# gpt-copy-v8

Generated CodeDungeon Claude E2E example repository for a ChatGPT-style application.

## Local Environment

Create a local `.env` file from `.env.example` and set:

```dotenv
OPENROUTER_API_KEY=<local secret>
OPENROUTER_MODEL=nvidia/nemotron-3-super-120b-a12b:free
DATABASE_URL=sqlite://backend/gpt-copy-v8.sqlite
BACKEND_HOST=127.0.0.1
BACKEND_PORT=8080
FRONTEND_ORIGIN=http://localhost:3000
NEXT_PUBLIC_API_BASE_URL=http://localhost:8080
```

Do not commit real provider keys.

## CodeDungeon

This fixture is intended to verify the Claude provider path for CodeDungeon full workflows.

Project Rules must be approved before the first real run:

```powershell
.\.claude\bin\codedungeon.exe rules status --human
```

The initial full-run prompt lives in `prompts/full-v8.txt`.

