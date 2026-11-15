import { use } from "react";
import { SystemStatusContext, type SystemStatus } from "./SystemStatusContext";

export function useSystemStatus(): SystemStatus {
  const status = use(SystemStatusContext);
  if (status === null) {
    throw new Error("useSystemStatus doit être utilisé dans <SystemStatusProvider>.");
  }
  return status;
}
