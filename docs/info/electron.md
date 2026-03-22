# Electron — technical documentation

The Electron wrapper packages the Starbound frontend as a native desktop application for Windows and Linux. It loads the compiled Yew/WASM frontend bundle and communicates with the same Go gateway and Python microservices running locally — no code changes to the frontend are required between the browser and desktop versions.

---

## Technology

| | |
|---|---|
| Electron | 28 |
| Node.js | 24 |
| electron-builder | 24 |
| mime-types | latest |

---

## Directory structure
```
electron/
├── main.js          # Main process — window creation, static server, routing
├── preload.js       # Preload script — exposes safe APIs to renderer
├── package.json     # App config and build settings
└── node_modules/    # Dependencies (gitignored)
```

---

## How it works

### Development mode

In development, Electron simply points a `BrowserWindow` at the Trunk dev server running on `http://localhost:8080`. This means you get hot reload, the same experience as the browser, and full DevTools support.
```bash
# Start the Trunk dev server first
cd frontend && trunk serve

# Then in a separate terminal
cd electron
NODE_ENV=development npm start
```

### Production mode

In production, Electron can't use `file://` URLs to load the frontend because:

1. Yew's `BrowserRouter` uses the History API which doesn't work with `file://` URLs — the router sees the full filesystem path instead of a URL path and can't match any routes
2. WASM loading via `file://` has restrictions in Chromium
3. Relative asset paths behave differently under `file://`

The solution is to bundle a lightweight Node.js HTTP server inside the Electron main process. On startup, the server reads files from the packaged `frontend/dist/` folder and serves them over `http://localhost:8090`. The `BrowserWindow` then loads `http://localhost:8090` — exactly the same as a browser hitting a web server.

This means:
- `BrowserRouter` works correctly (it sees normal HTTP URLs)
- WASM loads without restrictions
- SPA routing works — any path that isn't a real file falls back to `index.html`, and Yew's router handles it client-side
- The frontend can still make API calls to the Go gateway on `http://localhost:8000`
```javascript
// Simplified version of the static server in main.js
function startStaticServer(distPath) {
    const server = http.createServer((req, res) => {
        let filePath = path.join(distPath, req.url.split("?")[0]);

        // SPA fallback — unknown paths serve index.html
        if (!fs.existsSync(filePath) || fs.statSync(filePath).isDirectory()) {
            filePath = path.join(distPath, "index.html");
        }

        const contentType = mime.lookup(filePath) || "application/octet-stream";
        res.writeHead(200, { "Content-Type": contentType });
        res.end(fs.readFileSync(filePath));
    });

    server.listen(8090);
}
```

---

## Building the frontend for Electron

Before building the Electron app, the frontend must be compiled with relative asset paths. This is configured in `frontend/Trunk.toml`:
```toml
[build]
target     = "index.html"
public_url = "./"
```

The `public_url = "./"` setting is critical — it makes Trunk output relative paths like `./starbound-frontend-xxx.js` instead of absolute paths like `/starbound-frontend-xxx.js`. Without this, the embedded HTTP server can't find the assets.

Build the frontend first:
```bash
cd frontend
trunk build --release
# Output in frontend/dist/
```

---

## Building the Electron app
```bash
cd electron
npm install
npm run build:win    # Windows NSIS installer + unpacked folder
npm run build:lin    # Linux AppImage
```

Output goes to `electron/dist/`. The `win-unpacked/` folder contains the app ready to run without installation. The NSIS installer requires administrator privileges for code signing on Windows — if you don't have those, use the unpacked folder directly.

### Code signing note

On Windows, `electron-builder` attempts to download `winCodeSign` for code signing. If you see errors like:
```
symbolic link : A required privilege is not held by the client
```

This means the code signing step failed due to missing administrator privileges. The app still builds successfully — the `win-unpacked/` folder will be present and functional. The error only affects the creation of a signed `.exe` installer.

For portfolio purposes, the unpacked folder is sufficient. For a production release, you would need a code signing certificate and administrator privileges.

---

## Packaging details

The `package.json` build config uses `extraResources` to copy the frontend dist folder into the packaged app:
```json
"extraResources": [
    {
        "from": "../frontend/dist",
        "to":   "frontend/dist",
        "filter": ["**/*"]
    }
]
```

At runtime, `process.resourcesPath` points to the resources folder inside the packaged app, so the dist path is:
```javascript
path.join(process.resourcesPath, "frontend", "dist")
```

---

## Running the packaged app

The packaged desktop app requires all backend services to be running locally:
```bash
# Terminal 1 — Go gateway
cd gateway && go run cmd/main.go

# Terminal 2 — RAG service (optional — needed for chatbot)
cd rag-service && source venv/Scripts/activate
uvicorn app.main:app --reload --port 8001

# Terminal 3 — CV service (optional — needed for refunds)
cd cv-service && source venv/Scripts/activate
uvicorn app.main:app --reload --port 8002

# Then launch the app
electron/dist/win-unpacked/Starbound.exe
```

The frontend makes all API calls to `http://localhost:8000` (the gateway), which in turn proxies to the microservices. This is identical to the browser experience.

---

## Preload script

The preload script exposes a small safe API to the renderer process via `contextBridge`:
```javascript
contextBridge.exposeInMainWorld("starbound", {
    platform: process.platform,
    version:  process.env.npm_package_version,
});
```

This gives the frontend access to `window.starbound.platform` (e.g. `"win32"`) and `window.starbound.version` if needed, without exposing the full Node.js API.

---

## Security configuration

The `BrowserWindow` is configured with:

- `contextIsolation: true` — renderer process is isolated from the main process
- `nodeIntegration: false` — Node.js APIs are not available in the renderer
- `setWindowOpenHandler` — external `http://` links open in the default browser instead of a new Electron window; `http://localhost` links (gateway API calls) are allowed through

---

## What is not included in the packaged app

The following are not bundled with the Electron app and must run separately:

- Go gateway (`localhost:8000`)
- RAG service (`localhost:8001`)
- CV service (`localhost:8002`)

For a fully self-contained desktop app, these services would need to be bundled as child processes spawned by the Electron main process. This is out of scope for the current portfolio implementation.

---

## gitignore

The following Electron-related paths are gitignored:
```
electron/dist/        # Built app — up to 168MB, not suitable for git
electron/node_modules/
```

Always run `npm install` after cloning to restore node_modules.
