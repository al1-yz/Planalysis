import type { RouteObject } from "react-router";

import { DashboardPage } from "@/features/dashboard/dashboard-page";

import { AppShell } from "./layout/app-shell";
import { footerNavigation, primaryNavigation } from "./navigation";
import { ComingSoonPage } from "./pages/coming-soon-page";
import { NotFoundPage } from "./pages/not-found-page";
import { RouteErrorPage } from "./pages/route-error-page";

// Sections without an implemented milestone render a placeholder.
// Replace an entry here when its feature page exists.
const placeholderRoutes: RouteObject[] = [
  ...primaryNavigation.flatMap((group) => group.items),
  ...footerNavigation,
]
  .filter((item) => item.path !== "/")
  .map((item) => ({
    path: item.path,
    element: <ComingSoonPage title={item.label} />,
  }));

export const routes: RouteObject[] = [
  {
    element: <AppShell />,
    errorElement: <RouteErrorPage />,
    children: [
      { index: true, element: <DashboardPage /> },
      ...placeholderRoutes,
      { path: "*", element: <NotFoundPage /> },
    ],
  },
];
