import { AppShell } from "@/components/layout/AppShell";
import { PassphraseDialog } from "@/components/auth/PassphraseDialog";
import { useKeyboardShortcuts } from "@/hooks/useKeyboardShortcuts";
import { useAppStore } from "@/stores/app-store";

function App() {
  useKeyboardShortcuts();
  const isUnlocked = useAppStore((s) => s.isUnlocked);

  if (!isUnlocked) {
    return <PassphraseDialog />;
  }

  return <AppShell />;
}

export default App;
