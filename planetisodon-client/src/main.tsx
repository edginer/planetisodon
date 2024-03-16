import React, { Suspense } from "react";
import ReactDOM from "react-dom/client";
import App from "./pages/App.tsx";
import "./index.css";
import { createBrowserRouter, RouterProvider } from "react-router-dom";
import ThreadListView from "./pages/ThreadListView.tsx";
import ThreadView from "./pages/ThreadView.tsx";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ErrorBoundary } from "react-error-boundary";

const queryClient = new QueryClient({});

const router = createBrowserRouter([
  {
    path: "/",
    element: <App />,
    children: [
      {
        path: "/:boardKey",
        element: (
          <Suspense fallback={<div>Loading...</div>}>
            <ThreadListView />
          </Suspense>
        ),
        children: [
          {
            path: "/:boardKey/:threadKey",
            element: (
              <Suspense fallback={<div>Loading...</div>}>
                <ThreadView />
              </Suspense>
            ),
          },
        ],
      },
    ],
  },
]);

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <ErrorBoundary fallback={<div>Error</div>}>
      <Suspense fallback={<div>Loading...</div>}>
        <QueryClientProvider client={queryClient}>
          <RouterProvider router={router} />
        </QueryClientProvider>
      </Suspense>
    </ErrorBoundary>
  </React.StrictMode>
);
