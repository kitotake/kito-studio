import { createContext } from "react";
import type { KitoError } from "@/lib/errors";
import type { MediaToolsStatus, RuntimeInfo } from "@/types/system";

export type SystemStatus =
  | { kind: "loading" }
  | { kind: "browser" }
  | { kind: "ready"; runtime: RuntimeInfo; tools: MediaToolsStatus }
  | { kind: "error"; error: KitoError };

export const SystemStatusContext = createContext<SystemStatus | null>(null);
