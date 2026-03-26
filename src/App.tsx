import { useEffect, useState } from "react";
import { AppShell } from "@/components/layout/AppShell";
import { OnboardingWizard } from "@/components/onboarding/OnboardingWizard";
import { useKeyboardShortcuts } from "@/hooks/useKeyboardShortcuts";
import { useAppStore } from "@/stores/app-store";
import { commands } from "@/lib/tauri";

function App() {
  useKeyboardShortcuts();

  const isOnboarded = useAppStore((s) => s.isOnboarded);
  const setOnboarded = useAppStore((s) => s.setOnboarded);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;

    async function checkOnboarding() {
      try {
        const [complete, stats] = await Promise.all([
          commands.isOnboardingComplete(),
          commands.getAppStats(),
        ]);
        if (cancelled) return;
        // Show onboarding only if not completed AND no documents exist
        if (complete || stats.total_documents > 0) {
          setOnboarded(true);
        } else {
          setOnboarded(false);
        }
      } catch {
        // On error, assume onboarded to avoid blocking
        if (!cancelled) setOnboarded(true);
      } finally {
        if (!cancelled) setLoading(false);
      }
    }

    checkOnboarding();
    return () => {
      cancelled = true;
    };
  }, [setOnboarded]);

  // PassphraseDialog is available but disabled until SQLCipher is compiled in.
  // To enable: import PassphraseDialog, gate behind useAppStore isUnlocked state.

  if (loading) return null;

  return isOnboarded ? <AppShell /> : <OnboardingWizard />;
}

export default App;
