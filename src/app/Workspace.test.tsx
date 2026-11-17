import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { AutosaveFailedEvent, AutosavedEvent, OpenOutcome, ProjectSnapshot } from "@/types/project";
import { makeMedia, makeRecent, makeSnapshot } from "@/test/fixtures";

const api = vi.hoisted(() => ({
  isNativeRuntime: vi.fn(),
  listenAutosave: vi.fn(),
  onCloseRequested: vi.fn(),
  destroyWindow: vi.fn(),
  nativeApi: {
    getRuntimeInfo: vi.fn(),
    checkMediaTools: vi.fn(),
    project: {
      create: vi.fn(), open: vi.fn(), current: vi.fn(), save: vi.fn(), saveAs: vi.fn(),
      updateSettings: vi.fn(), close: vi.fn(), listAutosaves: vi.fn(), restoreAutosave: vi.fn(),
      relinkMedia: vi.fn(), relinkMediaInFolder: vi.fn(), recent: vi.fn(), removeRecent: vi.fn(), clearRecent: vi.fn()
    },
    preferences: { get: vi.fn(), set: vi.fn() },
    dialogs: { pickFolder: vi.fn(), pickProjectFile: vi.fn(), pickMediaFile: vi.fn(), pickSaveProjectPath: vi.fn() }
  }
}));
vi.mock("@/lib/tauri", () => api);

import App from "./App";

const { project, preferences, dialogs } = api.nativeApi;
let autosaveHandlers: { onSaved: (e: AutosavedEvent) => void; onFailed: (e: AutosaveFailedEvent) => void };

beforeEach(() => {
  api.isNativeRuntime.mockReturnValue(true);
  api.listenAutosave.mockImplementation(async (handlers: typeof autosaveHandlers) => {
    autosaveHandlers = handlers;
    return () => undefined;
  });
  api.onCloseRequested.mockResolvedValue(() => undefined);
  api.nativeApi.getRuntimeInfo.mockResolvedValue({ appName: "Kito Studio", appVersion: "0.1.0", os: "windows", architecture: "x86_64" });
  api.nativeApi.checkMediaTools.mockResolvedValue({ ffmpegFound: true, ffprobeFound: true });
  project.recent.mockResolvedValue([makeRecent()]);
  project.listAutosaves.mockResolvedValue([]);
  project.close.mockResolvedValue(undefined);
  project.removeRecent.mockResolvedValue(undefined);
  project.clearRecent.mockResolvedValue(undefined);
  preferences.get.mockResolvedValue({ version: 1, autosave: { enabled: true, intervalMinutes: 5, keepCopies: 10 } });
});

/** Ouvre le projet récent « Mon film » avec le résultat donné et attend l'éditeur. */
async function openFromRecents(snapshot: ProjectSnapshot, recovery: OpenOutcome["recovery"] = null) {
  project.open.mockResolvedValue({ snapshot, recovery } satisfies OpenOutcome);
  render(<App />);
  fireEvent.click(await screen.findByText("Mon film"));
  await screen.findByRole("heading", { name: snapshot.name, level: 1 });
}

describe("accueil", () => {
  it("affiche les vrais projets récents renvoyés par le backend", async () => {
    project.recent.mockResolvedValue([makeRecent(), makeRecent({ path: "/x/Autre.kito", name: "Autre" })]);
    render(<App />);
    expect(await screen.findByText("Autre")).toBeInTheDocument();
    expect(screen.getByText("2 projets")).toBeInTheDocument();
  });

  it("n'affiche aucun projet fictif quand la liste est vide", async () => {
    project.recent.mockResolvedValue([]);
    render(<App />);
    expect(await screen.findByText("Aucun projet récent")).toBeInTheDocument();
  });

  it("retire un projet des récents", async () => {
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Retirer Mon film des projets récents" }));
    await waitFor(() => expect(project.removeRecent).toHaveBeenCalledWith("/projets/Mon film/Mon film.kito"));
  });
});

describe("création", () => {
  it("crée un projet avec les réglages choisis puis ouvre l'éditeur", async () => {
    project.create.mockResolvedValue(makeSnapshot({ name: "Mon premier montage", width: 3840, height: 2160, frameRateNum: 25, sampleRate: 96_000 }));
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: /Nouveau projet/ }));

    fireEvent.change(screen.getByLabelText("Résolution"), { target: { value: "2160p" } });
    fireEvent.change(screen.getByLabelText("Fréquence d'images"), { target: { value: "25/1" } });
    fireEvent.change(screen.getByLabelText("Fréquence d'échantillonnage audio"), { target: { value: "96000" } });
    fireEvent.change(screen.getByLabelText("Emplacement du projet"), { target: { value: "/mes/projets" } });
    fireEvent.click(screen.getByRole("button", { name: "Créer le projet" }));

    await screen.findByRole("heading", { name: "Mon premier montage", level: 1 });
    expect(project.create).toHaveBeenCalledWith({
      name: "Mon premier montage", parentDir: "/mes/projets",
      width: 3840, height: 2160, frameRateNum: 25, frameRateDen: 1, sampleRate: 96_000
    });
    expect(screen.getByText("3840 × 2160 · 25 FPS · Audio 96 kHz")).toBeInTheDocument();
    expect(localStorage.getItem("kito:lastProjectLocation")).toBe("/mes/projets");
  });

  it("affiche l'erreur dans la fenêtre sans la fermer quand le dossier existe déjà", async () => {
    project.create.mockRejectedValue({ code: "ALREADY_EXISTS", message: "Un dossier « Film » existe déjà à cet emplacement." });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: /Nouveau projet/ }));
    fireEvent.change(screen.getByLabelText("Emplacement du projet"), { target: { value: "/p" } });
    fireEvent.click(screen.getByRole("button", { name: "Créer le projet" }));

    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent("Impossible de créer le projet");
    expect(alert).toHaveTextContent("Un dossier « Film » existe déjà");
    expect(alert).toHaveTextContent("Choisissez un autre nom");
    expect(screen.getByRole("dialog")).toBeInTheDocument();
    expect(screen.queryByRole("heading", { level: 1, name: "Film" })).not.toBeInTheDocument();
  });
});

describe("ouverture", () => {
  it("ouvre un projet récent et affiche son vrai contenu", async () => {
    await openFromRecents(makeSnapshot({ width: 2560, height: 1440, frameRateNum: 50 }));
    expect(project.open).toHaveBeenCalledWith("/projets/Mon film/Mon film.kito");
    expect(screen.getByText("2560 × 1440 · 50 FPS · Audio 48 kHz")).toBeInTheDocument();
  });

  it("ouvre un fichier choisi dans le sélecteur natif", async () => {
    dialogs.pickProjectFile.mockResolvedValue("/ailleurs/Autre.kito");
    project.open.mockResolvedValue({ snapshot: makeSnapshot({ name: "Autre" }), recovery: null });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Ouvrir…" }));
    await screen.findByRole("heading", { name: "Autre", level: 1 });
    expect(project.open).toHaveBeenCalledWith("/ailleurs/Autre.kito");
  });

  it("ne fait rien si le sélecteur est annulé", async () => {
    dialogs.pickProjectFile.mockResolvedValue(null);
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Ouvrir…" }));
    await waitFor(() => expect(dialogs.pickProjectFile).toHaveBeenCalled());
    expect(project.open).not.toHaveBeenCalled();
  });

  it("explique un fichier invalide et propose les copies de récupération", async () => {
    project.open.mockRejectedValue({ code: "INVALID_FORMAT", message: "Fichier illisible ou incomplet (JSON invalide, ligne 3, colonne 1)." });
    render(<App />);
    fireEvent.click(await screen.findByText("Mon film"));
    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent("Impossible d'ouvrir le projet");
    expect(alert).toHaveTextContent("Fichier illisible ou incomplet");
    expect(alert).toHaveTextContent(/Autosave|\.bak/);
    expect(screen.getByRole("heading", { name: "Donnez vie à vos idées." })).toBeInTheDocument();
  });

  it("explique une version incompatible", async () => {
    project.open.mockRejectedValue({ code: "INCOMPATIBLE_VERSION", message: "Ce projet a été créé par une version plus récente de Kito Studio." });
    render(<App />);
    fireEvent.click(await screen.findByText("Mon film"));
    expect(await screen.findByRole("alert")).toHaveTextContent("Mettez à jour Kito Studio");
  });

  it("informe qu'un projet a été converti depuis un ancien format", async () => {
    await openFromRecents(makeSnapshot({ migratedFrom: 1 }));
    expect(screen.getByText(/converti depuis le format 1/)).toBeInTheDocument();
  });
});

describe("projet qui ne s'ouvre plus", () => {
  const copy = { path: "/projets/Mon film/Autosave/Mon film-20261115-101500.kito", modifiedAt: "2026-11-15T10:15:00Z" };

  it("propose la dernière copie automatique quand le fichier est corrompu, et la restaure", async () => {
    project.open.mockRejectedValue({ code: "INVALID_FORMAT", message: "Fichier illisible ou incomplet (JSON invalide, ligne 3, colonne 1)." });
    project.listAutosaves.mockResolvedValue([copy]);
    render(<App />);
    fireEvent.click(await screen.findByText("Mon film"));
    const button = await screen.findByRole("button", { name: /Restaurer la copie automatique du/ });
    expect(project.listAutosaves).toHaveBeenCalledWith("/projets/Mon film/Mon film.kito");

    project.restoreAutosave.mockResolvedValue(makeSnapshot({ dirty: true, width: 1280, height: 720 }));
    fireEvent.click(button);
    await waitFor(() => expect(project.restoreAutosave).toHaveBeenCalledWith("/projets/Mon film/Mon film.kito", copy.path));
    expect(await screen.findByText("1280 × 720 · 30 FPS · Audio 48 kHz")).toBeInTheDocument();
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("ne propose rien quand il n'existe aucune copie automatique", async () => {
    project.open.mockRejectedValue({ code: "NOT_FOUND", message: "Lecture de « Mon film.kito » : élément introuvable" });
    render(<App />);
    fireEvent.click(await screen.findByText("Mon film"));
    expect(await screen.findByRole("alert")).toHaveTextContent("introuvable");
    expect(screen.queryByRole("button", { name: /Restaurer la copie automatique/ })).not.toBeInTheDocument();
  });
});

describe("navigation", () => {
  it("revenir à l'accueil ne ferme pas le projet", async () => {
    await openFromRecents(makeSnapshot());
    fireEvent.click(screen.getByRole("button", { name: /Accueil/ }));
    expect(screen.getByRole("heading", { name: "Donnez vie à vos idées." })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /Éditeur/ }));
    expect(screen.getByRole("heading", { name: "Mon film", level: 1 })).toBeInTheDocument();
    expect(project.close).not.toHaveBeenCalled();
  });

  it("l'éditeur est inaccessible sans projet ouvert", async () => {
    render(<App />);
    expect(await screen.findByRole("button", { name: /Éditeur/ })).toBeDisabled();
  });
});

describe("enregistrement", () => {
  it("enregistre avec le bouton et met à jour l'indicateur", async () => {
    await openFromRecents(makeSnapshot({ dirty: true }));
    expect(screen.getAllByText(/Modifications non enregistrées/).length).toBeGreaterThan(0);
    project.save.mockResolvedValue(makeSnapshot({ dirty: false, lastSavedAt: "2026-11-15T12:30:00Z" }));
    fireEvent.click(screen.getByRole("button", { name: "Enregistrer" }));
    await waitFor(() => expect(screen.getByText(/^Enregistré à \d{2}:\d{2}$/)).toBeInTheDocument());
    expect(project.save).toHaveBeenCalledTimes(1);
  });

  it("le bouton Enregistrer est inactif quand rien n'a changé", async () => {
    await openFromRecents(makeSnapshot({ dirty: false }));
    expect(screen.getByRole("button", { name: "Enregistrer" })).toBeDisabled();
  });

  it("Ctrl+S enregistre", async () => {
    await openFromRecents(makeSnapshot({ dirty: true }));
    project.save.mockResolvedValue(makeSnapshot());
    fireEvent.keyDown(window, { key: "s", ctrlKey: true });
    await waitFor(() => expect(project.save).toHaveBeenCalled());
  });

  it("affiche un échec d'écriture compréhensible et permet de réessayer", async () => {
    await openFromRecents(makeSnapshot({ dirty: true }));
    project.save.mockRejectedValueOnce({ code: "DISK_FULL", message: "Écriture de « Mon film.kito » : espace disque insuffisant" });
    fireEvent.click(screen.getByRole("button", { name: "Enregistrer" }));

    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent("L'enregistrement a échoué");
    expect(alert).toHaveTextContent("espace disque insuffisant");
    expect(alert).toHaveTextContent("Vos modifications sont toujours en mémoire");
    expect(screen.getAllByText(/Modifications non enregistrées/).length).toBeGreaterThan(0);

    project.save.mockResolvedValueOnce(makeSnapshot({ dirty: false }));
    fireEvent.click(within(alert).getByRole("button", { name: "Réessayer" }));
    await waitFor(() => expect(screen.queryByRole("alert")).not.toBeInTheDocument());
    expect(project.save).toHaveBeenCalledTimes(2);
  });

  it("« Enregistrer sous… » utilise le chemin choisi", async () => {
    await openFromRecents(makeSnapshot({ dirty: true }));
    dialogs.pickSaveProjectPath.mockResolvedValue("/archives/Copie.kito");
    project.saveAs.mockResolvedValue(makeSnapshot({ path: "/archives/Copie.kito", dirty: false }));
    fireEvent.click(screen.getByRole("button", { name: "Enregistrer sous…" }));
    await waitFor(() => expect(project.saveAs).toHaveBeenCalledWith("/archives/Copie.kito"));
    expect(dialogs.pickSaveProjectPath).toHaveBeenCalledWith("/projets/Mon film/Mon film.kito");
  });

  it("affiche l'état de la sauvegarde automatique (réussie puis échouée)", async () => {
    await openFromRecents(makeSnapshot({ dirty: true }));
    act(() => autosaveHandlers.onSaved({ savedAt: "2026-11-15T10:05:00Z", path: "/x/Autosave/a.kito" }));
    expect(await screen.findByText(/copie auto \d{2}:\d{2}/)).toBeInTheDocument();
    act(() => autosaveHandlers.onFailed({ message: "Disque plein" }));
    expect(await screen.findByText("Sauvegarde automatique échouée")).toHaveAttribute("title", "Disque plein");
  });
});

describe("modifications non enregistrées", () => {
  async function askToClose() {
    await openFromRecents(makeSnapshot({ dirty: true }));
    fireEvent.click(screen.getByRole("button", { name: "Fermer le projet" }));
    return screen.findByRole("dialog");
  }

  it("demande confirmation avant de fermer un projet modifié", async () => {
    const dialog = await askToClose();
    expect(within(dialog).getByRole("heading", { name: "Enregistrer « Mon film » ?" })).toBeInTheDocument();
    expect(project.close).not.toHaveBeenCalled();
  });

  it("« Annuler » garde le projet ouvert", async () => {
    const dialog = await askToClose();
    fireEvent.click(within(dialog).getByRole("button", { name: "Annuler" }));
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Mon film", level: 1 })).toBeInTheDocument();
    expect(project.close).not.toHaveBeenCalled();
  });

  it("« Ne pas enregistrer » abandonne les modifications et retourne à l'accueil", async () => {
    const dialog = await askToClose();
    fireEvent.click(within(dialog).getByRole("button", { name: "Ne pas enregistrer" }));
    expect(await screen.findByRole("heading", { name: "Donnez vie à vos idées." })).toBeInTheDocument();
    expect(project.close.mock.calls[0]).toEqual([true]);
    expect(project.save).not.toHaveBeenCalled();
  });

  it("« Enregistrer » sauvegarde puis ferme", async () => {
    const dialog = await askToClose();
    project.save.mockResolvedValue(makeSnapshot({ dirty: false }));
    fireEvent.click(within(dialog).getByRole("button", { name: "Enregistrer" }));
    expect(await screen.findByRole("heading", { name: "Donnez vie à vos idées." })).toBeInTheDocument();
    expect(project.save).toHaveBeenCalledTimes(1);
    expect(project.close).toHaveBeenCalledWith(false);
  });

  it("si l'enregistrement échoue, le projet reste ouvert et l'erreur s'affiche dans la fenêtre", async () => {
    const dialog = await askToClose();
    project.save.mockRejectedValue({ code: "PERMISSION_DENIED", message: "Écriture de « Mon film.kito » : accès refusé" });
    fireEvent.click(within(dialog).getByRole("button", { name: "Enregistrer" }));
    expect(await within(dialog).findByRole("alert")).toHaveTextContent("accès refusé");
    expect(project.close).not.toHaveBeenCalled();
    expect(screen.getByRole("dialog")).toBeInTheDocument();
  });

  it("ouvrir un autre projet demande aussi confirmation", async () => {
    await openFromRecents(makeSnapshot({ dirty: true }));
    dialogs.pickProjectFile.mockResolvedValue("/ailleurs/Autre.kito");
    fireEvent.click(screen.getByRole("button", { name: /Accueil/ }));
    fireEvent.click(screen.getByRole("button", { name: "Ouvrir…" }));
    expect(await screen.findByRole("heading", { name: "Enregistrer « Mon film » ?" })).toBeInTheDocument();
    expect(project.open).toHaveBeenCalledTimes(1);
  });

  it("n'interrompt pas la fermeture d'un projet propre", async () => {
    await openFromRecents(makeSnapshot({ dirty: false }));
    fireEvent.click(screen.getByRole("button", { name: "Fermer le projet" }));
    expect(await screen.findByRole("heading", { name: "Donnez vie à vos idées." })).toBeInTheDocument();
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });
});

describe("récupération après crash", () => {
  const offer = { autosavePath: "/projets/Mon film/Autosave/Mon film-20261115-101500.kito", autosavedAt: "2026-11-15T10:15:00Z" };

  it("propose la copie automatique et la restaure à la demande", async () => {
    await openFromRecents(makeSnapshot(), offer);
    const dialog = await screen.findByRole("dialog");
    expect(within(dialog).getByRole("heading", { name: "Récupérer un travail non enregistré ?" })).toBeInTheDocument();
    project.restoreAutosave.mockResolvedValue(makeSnapshot({ dirty: true, width: 1280, height: 720 }));
    fireEvent.click(within(dialog).getByRole("button", { name: "Restaurer la copie" }));
    await waitFor(() => expect(project.restoreAutosave).toHaveBeenCalledWith("/projets/Mon film/Mon film.kito", offer.autosavePath));
    expect(await screen.findByText("1280 × 720 · 30 FPS · Audio 48 kHz")).toBeInTheDocument();
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    expect(screen.getAllByText(/Modifications non enregistrées/).length).toBeGreaterThan(0);
  });

  it("« Ignorer » garde la version enregistrée sans rien restaurer", async () => {
    await openFromRecents(makeSnapshot(), offer);
    fireEvent.click(await screen.findByRole("button", { name: "Ignorer" }));
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    expect(project.restoreAutosave).not.toHaveBeenCalled();
    expect(screen.getByText("1920 × 1080 · 30 FPS · Audio 48 kHz")).toBeInTheDocument();
  });
});

describe("médias manquants", () => {
  const missing = makeMedia({ id: "m1", fileName: "plan1.mp4", status: "missing", resolvedPath: null });

  it("signale les médias introuvables à l'ouverture et permet de les localiser", async () => {
    await openFromRecents(makeSnapshot({ media: [missing, makeMedia({ id: "m2", fileName: "ok.mp4" })] }));
    const dialog = await screen.findByRole("dialog");
    expect(within(dialog).getByRole("heading", { name: "1 média à retrouver" })).toBeInTheDocument();
    expect(within(dialog).queryByText("ok.mp4")).not.toBeInTheDocument();

    dialogs.pickMediaFile.mockResolvedValue("/nouveau/plan1.mp4");
    project.relinkMedia.mockResolvedValue(makeSnapshot({ dirty: true, media: [makeMedia({ id: "m1", path: "/nouveau/plan1.mp4" }), makeMedia({ id: "m2", fileName: "ok.mp4" })] }));
    fireEvent.click(within(dialog).getByRole("button", { name: "Localiser…" }));
    await waitFor(() => expect(project.relinkMedia).toHaveBeenCalledWith("m1", "/nouveau/plan1.mp4"));
    await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
  });

  it("relie automatiquement depuis un dossier", async () => {
    await openFromRecents(makeSnapshot({ media: [missing] }));
    dialogs.pickFolder.mockResolvedValue("/nouveaux-rushes");
    project.relinkMediaInFolder.mockResolvedValue({ relinked: 1, snapshot: makeSnapshot({ dirty: true, media: [makeMedia()] }) });
    fireEvent.click(await screen.findByRole("button", { name: "Rechercher dans un dossier…" }));
    await waitFor(() => expect(project.relinkMediaInFolder).toHaveBeenCalledWith("/nouveaux-rushes"));
  });

  it("peut être reporté puis rouvert depuis la barre d'outils", async () => {
    await openFromRecents(makeSnapshot({ media: [missing] }));
    fireEvent.click(await screen.findByRole("button", { name: "Plus tard" }));
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "1 média à retrouver" }));
    expect(await screen.findByRole("dialog")).toBeInTheDocument();
  });

  it("n'affiche aucune alerte quand tous les médias sont présents", async () => {
    await openFromRecents(makeSnapshot({ media: [makeMedia()] }));
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    expect(screen.getByText("plan1.mp4")).toBeInTheDocument();
  });
});

describe("paramètres du projet", () => {
  it("applique de nouveaux paramètres et marque le projet modifié", async () => {
    await openFromRecents(makeSnapshot());
    fireEvent.click(screen.getByRole("button", { name: "Paramètres" }));
    const dialog = await screen.findByRole("dialog");
    fireEvent.change(within(dialog).getByLabelText("Résolution"), { target: { value: "2160p" } });
    fireEvent.change(within(dialog).getByLabelText("Fréquence d'images"), { target: { value: "60/1" } });
    project.updateSettings.mockResolvedValue(makeSnapshot({ dirty: true, width: 3840, height: 2160, frameRateNum: 60 }));
    fireEvent.click(within(dialog).getByRole("button", { name: "Appliquer" }));
    await waitFor(() => expect(project.updateSettings).toHaveBeenCalledWith({
      name: "Mon film", width: 3840, height: 2160, frameRateNum: 60, frameRateDen: 1, sampleRate: 48_000
    }));
    expect(await screen.findByText("3840 × 2160 · 60 FPS · Audio 48 kHz")).toBeInTheDocument();
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("garde la fenêtre ouverte et explique l'erreur si le backend refuse", async () => {
    await openFromRecents(makeSnapshot());
    fireEvent.click(screen.getByRole("button", { name: "Paramètres" }));
    const dialog = await screen.findByRole("dialog");
    project.updateSettings.mockRejectedValue({ code: "INVALID_FORMAT", message: "settings.video.width : doit être compris entre 16 et 16384" });
    fireEvent.click(within(dialog).getByRole("button", { name: "Appliquer" }));
    expect(await within(dialog).findByRole("alert")).toHaveTextContent("doit être compris entre 16 et 16384");
  });
});

describe("préférences", () => {
  it("modifie et enregistre la sauvegarde automatique", async () => {
    preferences.set.mockImplementation(async (p: unknown) => p);
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: /Préférences/ }));
    const dialog = await screen.findByRole("dialog");
    const interval = await within(dialog).findByLabelText("Intervalle");
    fireEvent.change(interval, { target: { value: "15" } });
    fireEvent.change(within(dialog).getByLabelText("Copies conservées par projet"), { target: { value: "3" } });
    fireEvent.click(within(dialog).getByLabelText("Activer la sauvegarde automatique"));
    fireEvent.click(within(dialog).getByRole("button", { name: "Enregistrer" }));
    await waitFor(() => expect(preferences.set).toHaveBeenCalledWith({
      version: 1, autosave: { enabled: false, intervalMinutes: 15, keepCopies: 3 }
    }));
    await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
  });

  it("affiche l'erreur du backend pour une valeur refusée", async () => {
    preferences.set.mockRejectedValue({ code: "INVALID_INPUT", message: "Le nombre de copies conservées doit être compris entre 1 et 100." });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: /Préférences/ }));
    const dialog = await screen.findByRole("dialog");
    await within(dialog).findByLabelText("Intervalle");
    fireEvent.click(within(dialog).getByRole("button", { name: "Enregistrer" }));
    expect(await within(dialog).findByRole("alert")).toHaveTextContent("entre 1 et 100");
  });
});
