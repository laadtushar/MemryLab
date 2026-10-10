// Site-wide constants: the canonical URL, product copy and the latest release.
//
// RELEASES: bump LATEST_VERSION when a new MemryLab release ships. It must match
// `gh api repos/laadtushar/MemryLab/releases/latest` (tag_name without the "v") and
// `version` in src-tauri/tauri.conf.json. The docs and the download links read it from here.

export const SITE_URL = "https://www.memrylab.com";
export const SITE_NAME = "MemryLab";
export const TAGLINE = "Your Memory, Visualized";

// One sentence, used for the home page meta description, og:description and JSON-LD.
export const SITE_DESCRIPTION =
  "Free, open-source desktop app that turns your journals, chats and data exports into a searchable timeline of how your thinking evolved. Runs on your device.";

// Alt text for the Open Graph image (rendered by app/opengraph-image.tsx).
export const OG_IMAGE_ALT = `${SITE_NAME}: ${TAGLINE}. A local AI memory app.`;

export const REPO_URL = "https://github.com/laadtushar/MemryLab";
export const LATEST_RELEASE_URL = `${REPO_URL}/releases/latest`;

export const LATEST_VERSION = "0.3.6";
const LATEST_TAG = `v${LATEST_VERSION}`;

// Size of the Windows installer (MemryLab_0.3.6_x64-setup.exe is 5,687,222 bytes).
export const WINDOWS_INSTALLER_SIZE = "5.7MB";

function asset(name: string) {
  return `${REPO_URL}/releases/download/${LATEST_TAG}/${name}`;
}

// Installers attached to the latest GitHub release, named as the release workflow names them.
export const DOWNLOADS = {
  windowsExe: { file: `MemryLab_${LATEST_VERSION}_x64-setup.exe`, url: asset(`MemryLab_${LATEST_VERSION}_x64-setup.exe`) },
  windowsMsi: { file: `MemryLab_${LATEST_VERSION}_x64_en-US.msi`, url: asset(`MemryLab_${LATEST_VERSION}_x64_en-US.msi`) },
  macosDmg: { file: `MemryLab_${LATEST_VERSION}_universal.dmg`, url: asset(`MemryLab_${LATEST_VERSION}_universal.dmg`) },
  linuxAppImage: { file: `MemryLab_${LATEST_VERSION}_amd64.AppImage`, url: asset(`MemryLab_${LATEST_VERSION}_amd64.AppImage`) },
  linuxDeb: { file: `MemryLab_${LATEST_VERSION}_amd64.deb`, url: asset(`MemryLab_${LATEST_VERSION}_amd64.deb`) },
  linuxRpm: { file: `MemryLab-${LATEST_VERSION}-1.x86_64.rpm`, url: asset(`MemryLab-${LATEST_VERSION}-1.x86_64.rpm`) },
} as const;

// Shared with the other LabyNator family sites (see lib/family.ts).
export const PUBLISHER = {
  "@type": "Organization",
  name: "LabyNator",
  url: "https://www.labynator.com",
} as const;

export const AUTHOR = {
  "@type": "Person",
  name: "Tushar Laad",
  url: "https://www.tusharlaad.com",
} as const;
