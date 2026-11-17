const KEY = "kito:lastProjectLocation";

/** Dernier emplacement de projet utilisé (le stockage du navigateur peut être indisponible). */
export function loadLastLocation(): string {
  try {
    return localStorage.getItem(KEY) ?? "";
  } catch {
    return "";
  }
}

export function saveLastLocation(location: string): void {
  try {
    localStorage.setItem(KEY, location);
  } catch {
    // Sans conséquence : l'emplacement ne sera simplement pas mémorisé.
  }
}
