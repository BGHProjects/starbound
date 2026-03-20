const { app, BrowserWindow, shell } = require("electron");
const path = require("path");
const http = require("http");
const fs   = require("fs");
const mime = require("mime-types");

const isDev = process.env.NODE_ENV === "development";
const PORT  = 8090;

function startStaticServer(distPath) {
    const server = http.createServer((req, res) => {
        let filePath = path.join(distPath, req.url.split("?")[0]);

        // SPA fallback — serve index.html for any path that isn't a real file
        if (!fs.existsSync(filePath) || fs.statSync(filePath).isDirectory()) {
            filePath = path.join(distPath, "index.html");
        }

        const contentType = mime.lookup(filePath) || "application/octet-stream";
        const content     = fs.readFileSync(filePath);

        res.writeHead(200, { "Content-Type": contentType });
        res.end(content);
    });

    server.listen(PORT);
    return server;
}

function createWindow() {
    const win = new BrowserWindow({
        width:           1280,
        height:          800,
        minWidth:        800,
        minHeight:       600,
        title:           "Starbound",
        backgroundColor: "#0a0f1e",
        webPreferences: {
            preload:          path.join(__dirname, "preload.js"),
            contextIsolation: true,
            nodeIntegration:  false,
        },
    });

    win.webContents.setWindowOpenHandler(({ url }) => {
        // Allow localhost URLs (our static server + gateway)
        if (url.startsWith("http://localhost")) return { action: "allow" };
        if (url.startsWith("http")) {
            shell.openExternal(url);
            return { action: "deny" };
        }
        return { action: "allow" };
    });

    if (isDev) {
        win.loadURL("http://localhost:8080");
        win.webContents.openDevTools();
    } else {
        win.loadURL(`http://localhost:${PORT}`);
    }

    win.setMenuBarVisibility(false);
}

app.whenReady().then(() => {
    if (!isDev) {
        const distPath = path.join(process.resourcesPath, "frontend", "dist");
        startStaticServer(distPath);
    }

    createWindow();

    app.on("activate", () => {
        if (BrowserWindow.getAllWindows().length === 0) createWindow();
    });
});

app.on("window-all-closed", () => {
    if (process.platform !== "darwin") app.quit();
});
