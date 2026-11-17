import { use } from "react";
import { ProjectContext, type ProjectContextValue } from "./ProjectContext";

export function useProject(): ProjectContextValue {
  const value = use(ProjectContext);
  if (value === null) {
    throw new Error("useProject doit être utilisé dans <ProjectProvider>.");
  }
  return value;
}
