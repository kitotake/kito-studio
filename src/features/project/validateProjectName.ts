const FORBIDDEN = /[<>:"/\\|?*]/;
// eslint-disable-next-line no-control-regex
const CONTROL = /[\u0000-\u001f\u007f]/;
const RESERVED = new Set([
  "CON", "PRN", "AUX", "NUL",
  "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
  "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"
]);

export const MAX_PROJECT_NAME_LENGTH = 100;

/**
 * Retourne un message d'erreur lisible, ou `null` si le nom est valide.
 * Miroir de `src-tauri/src/project/names.rs` (le backend reste l'autorité).
 */
export function validateProjectName(raw: string): string | null {
  const name = raw.trim();
  if (name === "") return "Le nom est vide.";
  if ([...name].length > MAX_PROJECT_NAME_LENGTH) {
    return `${MAX_PROJECT_NAME_LENGTH} caractères maximum.`;
  }
  const forbidden = FORBIDDEN.exec(name);
  if (forbidden) return `Le caractère « ${forbidden[0]} » est interdit.`;
  if (CONTROL.test(name)) return "Les caractères de contrôle sont interdits.";
  if (name.endsWith(".")) return "Le nom ne peut pas se terminer par un point.";
  const base = (name.split(".")[0] ?? name).toUpperCase();
  if (RESERVED.has(base)) return "Ce nom est réservé par Windows.";
  return null;
}
