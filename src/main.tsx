import React from "react";
import ReactDOM from "react-dom/client";
import "@fontsource-variable/roboto-condensed";
import "./styles.css";
import { App } from "./app/App";

const root = ReactDOM.createRoot(document.getElementById("root")!);
const query = new URLSearchParams(window.location.search);
const responsiveHarness = query.get("responsive-harness");
if (import.meta.env.DEV && responsiveHarness) {
  const width = Math.max(320, Math.min(1920, Number(responsiveHarness) || 820));
  root.render(<main style={{ width: "100%", height: "100%", overflow: "auto", padding: 12, background: "#24211d" }}><iframe title={`Vista responsive de ${width}px`} src="/?responsive-preview" style={{ display: "block", width, height: 680, border: "1px solid #777", margin: "0 auto", background: "white" }} /></main>);
} else if (import.meta.env.DEV && query.has("responsive-preview")) {
  void import("./dev/ResponsivePreview").then(({ ResponsivePreview }) => root.render(<ResponsivePreview />));
} else {
  root.render(<React.StrictMode><App /></React.StrictMode>);
}
