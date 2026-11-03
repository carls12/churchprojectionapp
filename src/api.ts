import { invoke } from "@tauri-apps/api/core";
import type { BibleBook, BibleVerse, BibleVersion, DashboardStats, MediaFile, ProjectionPayload, ProjectionSettings, Song } from "./types";

const isTauri = () => "__TAURI_INTERNALS__" in window;
export async function command<T>(name: string, args: Record<string, unknown> = {}): Promise<T> {
  if (!isTauri()) throw new Error("Desktop features require the Tauri app. Run npm run tauri dev.");
  return invoke<T>(name, args);
}
export const api = {
  stats: () => command<DashboardStats>("dashboard_stats"),
  songs: (query = "") => command<Song[]>("list_songs", { query }),
  saveSong: (song: Song) => command<number>("save_song", { song }),
  deleteSong: (id: number) => command<void>("delete_song", { id }),
  importSongText: (path: string) => command<number>("import_song_text", { path }),
  exportSongs: (path: string) => command<void>("export_songs", { path }),
  versions: () => command<BibleVersion[]>("list_bible_versions"),
  bibleBooks: (versionId: number) => command<BibleBook[]>("list_bible_books", { versionId }),
  bibleChapterCount: (versionId: number, bookNumber: number) => command<number>("bible_chapter_count", { versionId, bookNumber }),
  bibleVerseCount: (versionId: number, bookNumber: number, chapter: number) => command<number>("bible_verse_count", { versionId, bookNumber, chapter }),
  searchBible: (query: string, versionIds: number[]) => command<BibleVerse[]>("search_bible", { query, versionIds }),
  importBible: (path: string) => command<number>("import_bible", { path }),
  deleteVersion: (id: number) => command<void>("delete_bible_version", { id }),
  media: () => command<MediaFile[]>("list_media"),
  importMedia: (source: string) => command<number>("import_media", { source }),
  deleteMedia: (id: number) => command<void>("delete_media", { id }),
  setDefaultMedia: (id: number | null) => command<void>("set_default_media", { id }),
  settings: () => command<ProjectionSettings>("get_projection_settings"),
  saveSettings: (s: ProjectionSettings) => command<void>("save_projection_settings", { s }),
  openProjection: () => command<boolean>("open_projection"),
  fullscreen: (fullscreen: boolean) => command<void>("set_projection_fullscreen", { fullscreen }),
  publishProjection: (payload: ProjectionPayload) => command<void>("publish_projection", { payload }),
  projectionState: () => command<ProjectionPayload | null>("get_projection_state"),
  appVersion: () => command<string>("get_app_version"),
  installUpdate: (path: string) => command<void>("install_update", { path }),
  exportDatabase: (path: string) => command<void>("export_database", { path }),
};
