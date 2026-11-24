import { createHashRouter, type RouteObject } from "react-router";

import { NotFoundPage } from "@/components/common/not-found-page";
import { AppShell } from "@/components/layout/app-shell";
import { AnimePage } from "@/features/anime/pages/anime-page";
import { BookshelfPage } from "@/features/books/pages/bookshelf-page";
import { DashboardPage } from "@/features/dashboard/pages/dashboard-page";
import { NotesPage } from "@/features/notes/pages/notes-page";
import { ReaderPage } from "@/features/reader/pages/reader-page";
import { SearchPage } from "@/features/search/pages/search-page";
import { SettingsPage } from "@/features/settings/pages/settings-page";

import { RouteErrorPage } from "./route-error-page";

export const routes: RouteObject[] = [
  {
    path: "/",
    element: <AppShell />,
    errorElement: <RouteErrorPage />,
    children: [
      { index: true, element: <DashboardPage /> },
      { path: "anime", element: <AnimePage /> },
      { path: "bookshelf", element: <BookshelfPage /> },
      { path: "reader", element: <ReaderPage /> },
      { path: "notes", element: <NotesPage /> },
      { path: "search", element: <SearchPage /> },
      { path: "settings", element: <SettingsPage /> },
      { path: "*", element: <NotFoundPage /> },
    ],
  },
];

export const router = createHashRouter(routes);
