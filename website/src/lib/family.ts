// The LabyNator family of sites, shown in the footer block (components/FamilyFooter.tsx).
//
// Source of truth: labynator `src/lib/apps.ts` (laadtushar/labynator, origin/master).
// Names, taglines, blurbs and canonical URLs here must match it. Change them in one
// place only: this file.

export const FAMILY_STUDIO = {
  name: "LabyNator",
  url: "https://www.labynator.com",
} as const;

export const FAMILY_BUILDER = {
  name: "Tushar Laad",
  url: "https://www.tusharlaad.com",
} as const;

export type FamilyApp = {
  id: "memrylab" | "xpenselab" | "hyredlab" | "edytlab";
  name: string;
  tagline: string;
  blurb: string;
  /** Canonical URL: the host that actually serves the page, after redirects. */
  url: string;
};

// Order is fixed across the family: MemryLab, XpenseLab, HyredLab, edytlab.
export const FAMILY_APPS: readonly FamilyApp[] = [
  { id: "memrylab", name: "MemryLab", tagline: "Your Memory, Visualized", blurb: "Local AI Memory", url: "https://www.memrylab.com" },
  { id: "xpenselab", name: "XpenseLab", tagline: "Smart Expense Tracking", blurb: "Expense tracking", url: "https://xpenselab.com" },
  { id: "hyredlab", name: "HyredLab", tagline: "Smart Job Tracking", blurb: "Job tracking", url: "https://www.hyredlab.com" },
  { id: "edytlab", name: "edytlab", tagline: "Describe it. Get pro-grade audio edits.", blurb: "AI audio editor", url: "https://www.edytlab.com" },
];

// This site's own entry, left out of the footer list.
export const CURRENT_APP_ID: FamilyApp["id"] = "memrylab";
