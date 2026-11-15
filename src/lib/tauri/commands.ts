import type { MediaToolsStatus, RuntimeInfo } from "@/types/system";
import { invokeCommand } from "./client";

/**
 * API native typée : une fonction par commande Rust.
 * Les composants React n'utilisent jamais `invoke` directement.
 */
export const nativeApi = {
  getRuntimeInfo: () => invokeCommand<RuntimeInfo>("get_runtime_info"),
  checkMediaTools: () => invokeCommand<MediaToolsStatus>("check_media_tools")
} as const;
