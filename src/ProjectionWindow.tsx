import { useEffect, useState } from "react";
import { emitTo, listen } from "@tauri-apps/api/event";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import type { ProjectionPayload } from "./types";
import { api } from "./api";

const defaults: ProjectionPayload = { slide: { kind: "logo", title: "ChurchProjection Studio", subtitle: "Welcome", lines: [] }, settings: { fontFamily: "Georgia", fontSize: 64, textColor: "#ffffff", alignment: "center", overlayOpacity: .46, transitionMs: 450, defaultMediaId: null }, clear: false };
export default function ProjectionWindow() {
  const [payload, setPayload] = useState(defaults);
  const [visible, setVisible] = useState(true);
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen<ProjectionPayload>("projection:update", e => { setVisible(false); window.setTimeout(() => { setPayload(e.payload); setVisible(true); }, Math.min(e.payload.settings.transitionMs / 2, 220)); invoke("report_projection_status", { title: e.payload.slide.title }).catch(() => {}); }).then(fn => {
      unlisten = fn;
      emitTo("main", "projection:ready").catch(() => {});
    });
    return () => unlisten?.();
  }, []);
  useEffect(() => {
    let disposed = false;
    const sync = async () => {
      try { const current = await api.projectionState(); if (!disposed && current) setPayload(current); }
      catch { /* The event path remains available during startup. */ }
    };
    sync();
    const timer = window.setInterval(sync, 300);
    return () => { disposed = true; window.clearInterval(timer); };
  }, []);
  const { slide, settings, clear } = payload;
  const mediaUrl = slide.media ? convertFileSrc(slide.media.path) : "";
  return <main className="relative flex h-screen w-screen overflow-hidden bg-[radial-gradient(circle_at_center,#17213d_0%,#080d17_58%,#000_100%)]">
    {slide.media?.mediaType === "image" && <img className="absolute inset-0 h-full w-full object-cover" src={mediaUrl} />}
    {slide.media?.mediaType === "video" && <video className="absolute inset-0 h-full w-full object-cover" src={mediaUrl} autoPlay muted loop />}
    <div className="absolute inset-0 bg-black" style={{ opacity: slide.kind === "blank" ? 1 : settings.overlayOpacity }} />
    {!clear && slide.kind !== "blank" && <section className="relative z-10 m-auto w-full px-[7vw] py-[6vh]" style={{ color: settings.textColor, textAlign: settings.alignment, fontFamily: settings.fontFamily, opacity: visible ? 1 : 0, transition: `opacity ${settings.transitionMs / 2}ms ease` }}>
      {slide.kind === "logo" ? <><div className="mb-5 text-[clamp(1rem,2vw,2rem)] uppercase tracking-[.5em] text-amber-300">Welcome</div><h1 className="font-display text-[clamp(3rem,7vw,8rem)] leading-none">{slide.title}</h1><p className="mt-8 text-[clamp(1.2rem,2.5vw,2.5rem)] text-white/70">{slide.subtitle}</p></> : slide.kind === "bible" && slide.passages?.length ? <>
        <h1 className="mb-[5vh] font-display text-[clamp(30px,4vw,68px)] font-bold leading-tight drop-shadow-2xl">{slide.title}</h1>
        <div className="mx-auto flex max-w-[1500px] flex-col gap-[3.5vh]">
          {slide.passages.map((passage, i) => <div key={`${passage.version}-${i}`} className="leading-[1.25] drop-shadow-xl" style={{ fontSize: `clamp(22px, ${slide.passages!.length >= 3 ? settings.fontSize / 26 : settings.fontSize / 20}vw, ${slide.passages!.length >= 3 ? settings.fontSize * .7 : settings.fontSize}px)` }}><span className="mr-3 font-bold text-amber-300">[{passage.version}]</span><span className="font-semibold">{passage.text}</span></div>)}
        </div>
      </> : <>
        <div className="whitespace-pre-line font-semibold leading-[1.22] drop-shadow-2xl" style={{ fontSize: `clamp(28px, ${settings.fontSize / 16}vw, ${settings.fontSize * 1.35}px)` }}>{slide.lines.join("\n")}</div>
        <footer className="mt-[5vh] text-[clamp(16px,1.8vw,30px)] font-medium tracking-wide text-white/80">{slide.title}{slide.subtitle && <span className="ml-4 text-white/55">{slide.subtitle}</span>}</footer>
      </>}
    </section>}
  </main>;
}
