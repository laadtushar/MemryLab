import { useEffect } from "react";
import { useAppStore } from "@/stores/app-store";

export function useKeyboardShortcuts() {
  const setView = useAppStore((s) => s.setView);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const mod = e.metaKey || e.ctrlKey;

      if (mod && e.key === "k") {
        e.preventDefault();
        setView("search");
      } else if (mod && e.key === "/") {
        e.preventDefault();
        setView("ask");
      } else if (mod && e.key === "i") {
        e.preventDefault();
        setView("import");
      } else if (mod && e.key === "t") {
        e.preventDefault();
        setView("timeline");
      }
    };

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [setView]);
}
