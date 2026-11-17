import type { Preferences } from "@/types/project";
import { invokeCommand } from "./client";

export const preferencesApi = {
  get: () => invokeCommand<Preferences>("get_preferences"),
  set: (preferences: Preferences) =>
    invokeCommand<Preferences>("set_preferences", { preferences })
} as const;
