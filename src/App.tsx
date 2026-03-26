import { AppShell } from "@/components/layout/AppShell";
import { useKeyboardShortcuts } from "@/hooks/useKeyboardShortcuts";

function App() {
  useKeyboardShortcuts();

  // PassphraseDialog is available but disabled until SQLCipher is compiled in.
  // To enable: import PassphraseDialog, gate behind useAppStore isUnlocked state.
  return <AppShell />;
}

export default App;
