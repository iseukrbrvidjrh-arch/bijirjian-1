import { createBrowserRouter, Navigate } from "react-router-dom";

import { AppLayout } from "@/components/layout/app-layout";
import { DashboardPage } from "@/pages/dashboard/dashboard-page";
import { InboxPage } from "@/pages/inbox/inbox-page";
import { KnowledgePage } from "@/pages/knowledge/knowledge-page";
import { SettingsPage } from "@/pages/settings/settings-page";
import { SourceDetailPage } from "@/pages/sources/source-detail-page";

export const router = createBrowserRouter([
  {
    path: "/",
    element: <AppLayout />,
    children: [
      {
        index: true,
        element: <Navigate to="/dashboard" replace />,
      },
      {
        path: "dashboard",
        element: <DashboardPage />,
      },
      {
        path: "inbox",
        element: <InboxPage />,
      },
      {
        path: "sources/:sourceId",
        element: <SourceDetailPage />,
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
