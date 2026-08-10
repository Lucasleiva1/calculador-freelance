import React from "react";
import ReactDOM from "react-dom/client";
import "@fontsource-variable/roboto-condensed";
import "./styles.css";
import { App } from "./app/App";

const root = ReactDOM.createRoot(document.getElementById("root")!);
if (import.meta.env.DEV && new URLSearchParams(window.location.search).has("responsive-preview")) {
  void import("./dev/ResponsivePreview").then(({ ResponsivePreview }) => root.render(<ResponsivePreview />));
} else {
  root.render(<React.StrictMode><App /></React.StrictMode>);
}
