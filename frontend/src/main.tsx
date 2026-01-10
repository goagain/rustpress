import { hydrateRoot, createRoot } from "react-dom/client";
import { BrowserRouter } from 'react-router-dom';
import App from "./App";
import "./styles.css";

const container = document.getElementById("rp-react-root");

const AppWithRouter = () => (
  <BrowserRouter>
    <App />
  </BrowserRouter>
);

if (!container) {
  console.warn("React root not found");
} else if (container.hasChildNodes()) {
  hydrateRoot(container, <AppWithRouter />);
} else {
  createRoot(container).render(<AppWithRouter />);
}