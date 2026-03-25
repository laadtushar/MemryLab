import { AppShell } from "@/components/layout/AppShell";
import { useKeyboardShortcuts } from "@/hooks/useKeyboardShortcuts";

function App() {
  useKeyboardShortcuts();
  return <AppShell />;
}

export default App;
