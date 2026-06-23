export type NavPage = "dashboard" | "songs" | "bible" | "media" | "projection" | "settings";
export interface Translation { language: string; content: string }
export interface SongSection { id?: number; sectionType: string; label: string; content: string; position: number; translations: Translation[] }
export interface Song { id?: number; title: string; author: string; category: string; language: string; copyrightInfo: string; mediaId: number | null; sections: SongSection[] }
export interface BibleVersion { id: number; name: string; abbreviation: string; language: string }
export interface BibleBook { bookNumber: number; name: string; abbreviation: string }
export interface BibleVerse { id: number; versionId: number; version: string; language: string; book: string; bookNumber: number; chapter: number; verse: number; text: string; reference: string }
export interface MediaFile { id: number; name: string; fileName: string; path: string; mediaType: "image" | "video"; isDefault: boolean }
export interface ProjectionSettings { fontFamily: string; fontSize: number; textColor: string; alignment: "left" | "center" | "right"; overlayOpacity: number; transitionMs: number; defaultMediaId: number | null }
export interface BiblePassage { version: string; language: string; text: string }
export interface ProjectionSlide { kind: "song" | "bible" | "logo" | "blank"; title: string; subtitle?: string; label?: string; lines: string[]; passages?: BiblePassage[]; media?: MediaFile; }
export interface ProjectionPayload { slide: ProjectionSlide; settings: ProjectionSettings; clear: boolean }
export interface DashboardStats { songs: number; sections: number; bibleVersions: number; bibleVerses: number; media: number }
