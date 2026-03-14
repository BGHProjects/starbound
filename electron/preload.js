const { contextBridge } = require("electron");

contextBridge.exposeInMainWorld("starbound", {
  platform: process.platform,
  version: process.env.npm_package_version,
});
