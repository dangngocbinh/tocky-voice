import React from "react";
import ReactDOM from "react-dom/client";
import { AppRoot } from "./app-root";
import "./styles.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <AppRoot />
  </React.StrictMode>,
);
