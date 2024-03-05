import React from "react";
import ReactDOM from "react-dom/client";
import App from "./pages/App.tsx";
import "./index.css";
import { createBrowserRouter, RouterProvider } from "react-router-dom";
import ThreadListView from "./pages/ThreadListView.tsx";
import ThreadView from "./pages/ThreadView.tsx";

const router = createBrowserRouter([
  {
    path: "/",
    element: <App />,
    children: [
      {
        path: "/:boardKey",
        element: <ThreadListView />,
        children: [
          {
            path: "/:boardKey/:threadKey",
            element: <ThreadView />,
          },
        ],
      },
    ],
  },
]);

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <RouterProvider router={router} />
  </React.StrictMode>
);
