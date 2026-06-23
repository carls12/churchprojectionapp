import type { ReactNode } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { X } from "lucide-react";
import type { MediaFile, ProjectionSettings, ProjectionSlide } from "./types";

export function PageHeader({ eyebrow, title, description, action }: { eyebrow: string; title: string; description: string; action?: ReactNode }) {
  return <header className="mb-7 flex items-end justify-between gap-5"><div><div className="eyebrow">{eyebrow}</div><h1 className="mt-2 text-3xl font-bold tracking-tight">{title}</h1><p className="mt-2 max-w-2xl text-sm text-slate-400">{description}</p></div>{action}</header>;
}
export function Modal({ title, children, onClose, wide = false }: { title: string; children: ReactNode; onClose: () => void; wide?: boolean }) {
  return <div className="fixed inset-0 z-50 grid place-items-center bg-black/75 p-5 backdrop-blur-sm" onMouseDown={e => e.target === e.currentTarget && onClose()}><div className={`card max-h-[92vh] w-full overflow-y-auto ${wide ? "max-w-5xl" : "max-w-2xl"}`}><div className="sticky top-0 z-10 flex items-center justify-between border-b bg-panel px-6 py-4"><h2 className="text-lg font-bold">{title}</h2><button className="rounded-lg p-2 hover:bg-slate-800" onClick={onClose}><X size={18}/></button></div><div className="p-6">{children}</div></div></div>;
}
export function MediaVisual({ media, className = "" }: { media?: MediaFile; className?: string }) {
  if (!media) return <div className={`bg-gradient-to-br from-slate-800 via-indigo-950 to-amber-950 ${className}`} />;
  const src = convertFileSrc(media.path);
  return media.mediaType === "video" ? <video src={src} muted className={`object-cover ${className}`} /> : <img src={src} className={`object-cover ${className}`} />;
}
export function Preview({ slide, settings }: { slide: ProjectionSlide; settings: ProjectionSettings }) {
  return <div className="relative aspect-video overflow-hidden rounded-xl bg-black shadow-2xl"><MediaVisual media={slide.media} className="absolute inset-0 h-full w-full"/><div className="absolute inset-0 bg-black" style={{ opacity: slide.kind === "blank" ? 1 : settings.overlayOpacity }}/>{slide.kind !== "blank" && <div className="absolute inset-0 flex flex-col justify-center px-[7%] py-[6%]" style={{ color: settings.textColor, textAlign: settings.alignment, fontFamily: settings.fontFamily }}>{slide.kind === "bible" && slide.passages?.length ? <><div className="mb-[4%] text-[clamp(15px,2.5vw,32px)] font-bold">{slide.title}</div><div className="space-y-[2.5%]">{slide.passages.map((p,i)=><div className="text-[clamp(10px,1.35vw,18px)] leading-tight" key={`${p.version}-${i}`}><span className="mr-2 font-bold text-amber-300">[{p.version}]</span>{p.text}</div>)}</div></>:<><div className="whitespace-pre-line font-semibold leading-tight" style={{ fontSize: `${Math.max(13, settings.fontSize / 4.5)}px` }}>{slide.lines.join("\n") || (slide.kind === "logo" ? slide.title : "No content selected")}</div>{slide.kind !== "logo" && <div className="mt-[5%] text-xs text-white/70">{slide.title} {slide.subtitle}</div>}</>}</div>}</div>;
}
