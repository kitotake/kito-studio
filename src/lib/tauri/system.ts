import type { MediaToolsStatus, RuntimeInfo } from "@/types/system";
import { invokeCommand } from "./client";

export const systemApi = {
  getRuntimeInfo: () => invokeCommand<RuntimeInfo>("get_runtime_info"),
  checkMediaTools: () => invokeCommand<MediaToolsStatus>("check_media_tools")
} as const;
