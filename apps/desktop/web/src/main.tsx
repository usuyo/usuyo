import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider } from "@tanstack/react-router";
import { DaemonUnreachableError } from "./api/client";
import { createAppRouter } from "./router";
import "./styles.css";

// 开发窗口里点底部的小工具条再点界面上的一处（或指着它按 ⌘C），复制出这个元素、它所在的组件和源文件；安装包里没有这一段
if (import.meta.env.DEV) void import("react-grab");

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: (count, error) => !(error instanceof DaemonUnreachableError) && count < 2,
      refetchOnWindowFocus: false,
    },
  },
});
const router = createAppRouter();

const root = document.getElementById("root");
if (!root) throw new Error("#root missing in index.html");

createRoot(root).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  </StrictMode>,
);
