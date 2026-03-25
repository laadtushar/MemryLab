import { useState, useEffect, type FormEvent } from "react";
import { commands } from "@/lib/tauri";
import { useAppStore } from "@/stores/app-store";

export function PassphraseDialog() {
  const setUnlocked = useAppStore((s) => s.setUnlocked);

  const [firstRun, setFirstRun] = useState<boolean | null>(null);
  const [passphrase, setPassphrase] = useState("");
  const [confirmPassphrase, setConfirmPassphrase] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    commands.isFirstRun().then(setFirstRun).catch(() => setFirstRun(true));
  }, []);

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    setError("");

    if (firstRun && passphrase !== confirmPassphrase) {
      setError("Passphrases do not match.");
      return;
    }

    if (passphrase.length === 0) {
      setError("Passphrase cannot be empty.");
      return;
    }

    setLoading(true);
    try {
      if (firstRun) {
        await commands.setPassphrase(passphrase);
      } else {
        await commands.unlockDatabase(passphrase);
      }
      setUnlocked(true);
    } catch (err) {
      setError(typeof err === "string" ? err : String(err));
    } finally {
      setLoading(false);
    }
  }

  if (firstRun === null) {
    return (
      <div className="flex h-screen w-screen items-center justify-center bg-background">
        <p className="text-muted-foreground animate-pulse">Loading...</p>
      </div>
    );
  }

  return (
    <div className="flex h-screen w-screen items-center justify-center bg-background">
      <div className="w-full max-w-md space-y-8 rounded-2xl border border-border bg-card p-8 shadow-xl">
        {/* Header */}
        <div className="text-center space-y-2">
          <h1 className="text-3xl font-bold tracking-tight text-foreground">
            Memory Palace
          </h1>
          <p className="text-sm text-muted-foreground">
            {firstRun
              ? "Create a passphrase to encrypt your data"
              : "Enter your passphrase to unlock"}
          </p>
        </div>

        {/* Form */}
        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="space-y-2">
            <label
              htmlFor="passphrase"
              className="text-sm font-medium text-foreground"
            >
              Passphrase
            </label>
            <input
              id="passphrase"
              type="password"
              autoFocus
              value={passphrase}
              onChange={(e) => setPassphrase(e.target.value)}
              className="w-full rounded-lg border border-border bg-background px-3 py-2 text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
              placeholder="Enter passphrase"
            />
          </div>

          {firstRun && (
            <div className="space-y-2">
              <label
                htmlFor="confirm-passphrase"
                className="text-sm font-medium text-foreground"
              >
                Confirm Passphrase
              </label>
              <input
                id="confirm-passphrase"
                type="password"
                value={confirmPassphrase}
                onChange={(e) => setConfirmPassphrase(e.target.value)}
                className="w-full rounded-lg border border-border bg-background px-3 py-2 text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
                placeholder="Confirm passphrase"
              />
            </div>
          )}

          {error && (
            <p className="text-sm text-destructive">{error}</p>
          )}

          <button
            type="submit"
            disabled={loading}
            className="w-full rounded-lg bg-primary px-4 py-2 font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50 transition-colors"
          >
            {loading
              ? "Please wait..."
              : firstRun
                ? "Create"
                : "Unlock"}
          </button>
        </form>

        {firstRun && (
          <p className="text-xs text-center text-muted-foreground">
            Your data is encrypted locally with SQLCipher. If you forget your
            passphrase, your data cannot be recovered.
          </p>
        )}
      </div>
    </div>
  );
}
