import { restartApplication, validateRecoveryBackup, type ManagedBackupCandidate, type StartupStatus } from "./native";

function bindRetry(root: HTMLElement) {
  const retry = root.querySelector<HTMLButtonElement>("[data-retry-startup]");
  const note = root.querySelector<HTMLElement>("[data-recovery-note]");
  retry?.addEventListener("click", () => {
    retry.disabled = true;
    retry.textContent = "Restarting…";

    void restartApplication().catch(() => {
      retry.disabled = false;
      retry.textContent = "Try again";
      if (note) {
        note.textContent = "B.O.B. could not restart automatically. Close and reopen B.O.B. to try again.";
      }
    });
  });
}

function bindBackupValidation(root: HTMLElement, candidates: ManagedBackupCandidate[]) {
  root.querySelectorAll<HTMLButtonElement>("[data-validate-backup]").forEach((button) => {
    button.addEventListener("click", () => {
      const candidateIndex = Number(button.dataset.validateBackup);
      const candidate = candidates[candidateIndex];
      const result = root.querySelector<HTMLElement>(`[data-backup-result="${candidateIndex}"]`);
      if (!candidate || !result) return;

      button.disabled = true;
      button.textContent = "Checking…";
      result.textContent = "B.O.B. is checking this backup without changing your saved work.";

      void validateRecoveryBackup(candidate.id)
        .then((preview) => {
          if (preview.validation === "usable") {
            const activeCopy = preview.hasActiveItem ? " It includes an active item." : " It does not mark an active item.";
            result.textContent = `${preview.message} It contains ${preview.workItemCount ?? 0} work item${preview.workItemCount === 1 ? "" : "s"}.${activeCopy}`;
            button.textContent = "Checked";
            return;
          }
          result.textContent = preview.message;
          button.disabled = false;
          button.textContent = "Check backup";
        })
        .catch(() => {
          result.textContent = "B.O.B. could not complete the backup check. Nothing was changed.";
          button.disabled = false;
          button.textContent = "Check backup";
        });
    });
  });
}

function focusRecoveryTitle(root: HTMLElement) {
  root.querySelector<HTMLElement>("#startup-recovery-title")?.focus({ preventScroll: true });
}

function formatBackupDate(candidate: ManagedBackupCandidate) {
  if (candidate.modifiedUnixMs === null) return "Saved time unavailable";
  return new Date(candidate.modifiedUnixMs).toLocaleString();
}

function formatBackupSize(sizeBytes: number) {
  if (sizeBytes < 1024) return `${sizeBytes} B`;
  if (sizeBytes < 1024 * 1024) return `${Math.max(1, Math.round(sizeBytes / 1024))} KB`;
  return `${(sizeBytes / (1024 * 1024)).toFixed(1)} MB`;
}

function renderBackupCandidates(status: StartupStatus) {
  if (status.managedBackupCandidates === null || status.managedBackupCandidates.length === 0) return "";

  const limitedCopy = status.managedBackupCount !== null && status.managedBackupCount > status.managedBackupCandidates.length
    ? ` Showing the ${status.managedBackupCandidates.length} newest.`
    : "";

  return `
    <section class="startup-recovery__candidates" aria-labelledby="startup-recovery-backups-title">
      <h2 id="startup-recovery-backups-title">Managed backups found</h2>
      <p>Check a backup to see whether B.O.B. can safely read it. Checking happens in isolation and does not restore or replace anything.${limitedCopy}</p>
      <ul>
        ${status.managedBackupCandidates.map((candidate, index) => `
          <li>
            <div>
              <b>Backup candidate ${index + 1}</b>
              <span>${formatBackupDate(candidate)} · ${formatBackupSize(candidate.sizeBytes)}</span>
            </div>
            <button class="button secondary" type="button" data-validate-backup="${index}">Check backup</button>
            <p class="startup-recovery__candidate-result" data-backup-result="${index}" aria-live="polite"></p>
          </li>
        `).join("")}
      </ul>
    </section>
  `;
}

export function renderStartupRecovery(status: StartupStatus) {
  const root = document.querySelector<HTMLElement>("#app");
  if (!root) return;

  const backupCopy = status.managedBackupCount === null
    ? "B.O.B. could not check the managed backup folder, so backup availability is unknown."
    : status.managedBackupCount > 0
      ? `B.O.B. found ${status.managedBackupCount} managed backup file${status.managedBackupCount === 1 ? "" : "s"} to evaluate for recovery.`
      : "B.O.B. did not find a managed backup file to evaluate for recovery.";

  root.innerHTML = `
    <main class="startup-recovery" aria-labelledby="startup-recovery-title">
      <section class="startup-recovery__card" role="alert">
        <p class="startup-recovery__eyebrow">B.O.B. protected your data</p>
        <h1 id="startup-recovery-title" tabindex="-1">B.O.B. could not open your saved work.</h1>
        <p class="startup-recovery__lead">
          Your existing data has not been reset or replaced. B.O.B. stopped normal startup so the original can stay intact.
        </p>
        <p class="startup-recovery__backup">${backupCopy}</p>
        ${renderBackupCandidates(status)}
        <div class="startup-recovery__actions">
          <button class="button primary" type="button" data-retry-startup>Try again</button>
        </div>
        <p class="startup-recovery__note" data-recovery-note>
          Recovery is fail-closed. B.O.B. will not choose or restore a backup automatically.
        </p>
      </section>
    </main>
  `;

  bindRetry(root);
  bindBackupValidation(root, status.managedBackupCandidates ?? []);
  focusRecoveryTitle(root);
}

export function renderStartupStatusUnavailable() {
  const root = document.querySelector<HTMLElement>("#app");
  if (!root) return;

  root.innerHTML = `
    <main class="startup-recovery" aria-labelledby="startup-recovery-title">
      <section class="startup-recovery__card" role="alert">
        <p class="startup-recovery__eyebrow">B.O.B. stopped startup safely</p>
        <h1 id="startup-recovery-title" tabindex="-1">B.O.B. could not confirm your saved-work status.</h1>
        <p class="startup-recovery__lead">
          Normal work loading was stopped because B.O.B. could not verify whether canonical state was ready to use.
        </p>
        <p class="startup-recovery__backup">
          Backup availability could not be checked in this state. B.O.B. will not guess or restore anything automatically.
        </p>
        <div class="startup-recovery__actions">
          <button class="button primary" type="button" data-retry-startup>Try again</button>
        </div>
        <p class="startup-recovery__note" data-recovery-note>
          If automatic restart is unavailable, close and reopen B.O.B. Normal loading will be attempted again from the beginning.
        </p>
      </section>
    </main>
  `;

  bindRetry(root);
  focusRecoveryTitle(root);
}
