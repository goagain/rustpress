import { hydrateRoot, createRoot } from "react-dom/client";
import App from "./App";
import "./styles.css";

const container = document.getElementById("rp-react-root");

if (!container) {
  console.warn("React root not found");
} else if (container.hasChildNodes()) {
  hydrateRoot(container, <App />);
} else {
  createRoot(container).render(<App />);
}