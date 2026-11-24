import { createHashRouter, type RouteObject } from "react-router";

import { NotFoundPage } from "@/components/common/not-found-page";
import { AppShell } from "@/components/layout/app-shell";
import { AnimeDetailPage } from "@/features/anime/pages/anime-detail-page";
import { AnimePage } from "@/features/anime/pages/anime-page";
import { BookDetailPage } from "@/features/books/pages/book-detail-page";
import { BookshelfPage } from "@/features/books/pages/bookshelf-page";
import { DashboardPage } from "@/features/dashboard/pages/dashboard-page";
import { FavoritesPage } from "@/features/favorites/pages/favorites-page";
import { NoteEditorPage } from "@/features/notes/pages/note-editor-page";
import { NotesPage } from "@/features/notes/pages/notes-page";
import { BookReaderPage } from "@/features/reader/pages/book-reader-page";
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
      { path: "anime/:animeId", element: <AnimeDetailPage /> },
      { path: "bookshelf", element: <BookshelfPage /> },
      { path: "bookshelf/:bookId", element: <BookDetailPage /> },
      { path: "reader", element: <ReaderPage /> },
      { path: "reader/:bookId", element: <BookReaderPage /> },
      { path: "notes", element: <NotesPage /> },
      { path: "notes/new", element: <NoteEditorPage /> },
      { path: "notes/:noteId", element: <NoteEditorPage /> },
      { path: "favorites", element: <FavoritesPage /> },
      { path: "search", element: <SearchPage /> },
      { path: "settings", element: <SettingsPage /> },
      { path: "*", element: <NotFoundPage /> },
    ],
  },
];

export const router = createHashRouter(routes);
