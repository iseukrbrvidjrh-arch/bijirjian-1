import { createBrowserRouter, Navigate } from "react-router-dom";

import { AppLayout } from "@/components/layout/app-layout";
import { InboxPage } from "@/pages/inbox/inbox-page";
import { KnowledgePage } from "@/pages/knowledge/knowledge-page";
import { SettingsPage } from "@/pages/settings/settings-page";

export const router = createBrowserRouter([
  {
    path: "/",
    element: <AppLayout />,
    children: [
      {
        index: true,
        element: <Navigate to="/inbox" replace />,
      },
      {
        path: "inbox",
        element: <InboxPage />,
      },
      {
        path: "knowledge",
        element: <KnowledgePage />,
      },
      {
        path: "settings",
        element: <SettingsPage />,
      },
    ],
  },
]);
