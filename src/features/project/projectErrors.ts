import type { KitoError } from "@/lib/errors";
import type { ErrorCode } from "@/types/errors";

export type ProjectErrorContext =
  | "create"
  | "open"
  | "save"
  | "autosave"
  | "settings"
  | "relink"
  | "other";

export type DescribedError = {
  code: ErrorCode;
  context: ProjectErrorContext;
  title: string;
  /** Message du backend (déjà en français et sans jargon). */
  message: string;
  /** Conseil sur la suite à donner. */
  suggestion: string | null;
};

const TITLES: Record<ProjectErrorContext, string> = {
  create: "Impossible de créer le projet",
  open: "Impossible d'ouvrir le projet",
  save: "L'enregistrement a échoué",
  autosave: "La sauvegarde automatique a échoué",
  settings: "Impossible d'appliquer les paramètres",
  relink: "Impossible de relier le média",
  other: "Une erreur est survenue"
};

function suggestionFor(code: ErrorCode, context: ProjectErrorContext): string | null {
  const keepWork =
    context === "save"
      ? " Vos modifications sont toujours en mémoire : réessayez ou utilisez « Enregistrer sous… »."
      : "";
  switch (code) {
    case "PERMISSION_DENIED":
      return `Choisissez un autre emplacement ou vérifiez les droits d'accès du dossier.${keepWork}`;
    case "DISK_FULL":
      return `Libérez de l'espace ou choisissez un autre disque.${keepWork}`;
    case "NOT_FOUND":
      return context === "open"
        ? "Le projet a peut-être été déplacé ou supprimé. Retirez-le des projets récents ou cherchez-le avec « Ouvrir… »."
        : `Vérifiez que le dossier existe toujours (disque externe déconnecté ?).${keepWork}`;
    case "ALREADY_EXISTS":
      return "Choisissez un autre nom de projet ou un autre emplacement.";
    case "INVALID_FORMAT":
      return "Si une copie automatique (dossier Autosave) ou une copie de secours (.bak) existe, elle peut contenir votre travail.";
    case "INCOMPATIBLE_VERSION":
      return "Mettez à jour Kito Studio pour ouvrir ce projet. Le fichier n'a pas été modifié.";
    case "INVALID_INPUT":
      return "Corrigez la valeur indiquée puis recommencez.";
    case "UNAVAILABLE":
      return null;
    case "IO":
    case "INTERNAL":
    case "UNKNOWN":
      return `Si le problème persiste, redémarrez l'application. Vos médias originaux ne sont jamais modifiés.${keepWork}`;
  }
}

export function describeProjectError(error: KitoError, context: ProjectErrorContext): DescribedError {
  return {
    code: error.code,
    context,
    title: TITLES[context],
    message: error.message,
    suggestion: suggestionFor(error.code, context)
  };
}
