import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { markPlatform } from "./platform";
import "./styles.css";
import "./styles.macos.css";

markPlatform();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
