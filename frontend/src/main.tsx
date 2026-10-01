import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./app/App";
import { LoadingScreen } from "./components/LoadingScreen";
import "./styles/tokens.css";

const container = document.getElementById("root");
if (!container) {
  throw new Error("Root element not found");
}

// Show loading screen immediately, let App mount async
const root = createRoot(container);
root.render(
  <StrictMode>
    <LoadingScreen />
  </StrictMode>
);

// After microtask, render the actual app (gives browser time to paint loading state)
queueMicrotask(() => {
  root.render(
    <StrictMode>
      <App />
    </StrictMode>
  );
});
