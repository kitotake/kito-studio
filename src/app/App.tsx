import { ProjectProvider } from "@/features/project/ProjectProvider";
import { SystemStatusProvider } from "@/features/system/SystemStatusProvider";
import { ErrorBoundary } from "./ErrorBoundary";
import { Workspace } from "./Workspace";

export default function App() {
  return (
    <ErrorBoundary>
      <SystemStatusProvider>
        <ProjectProvider>
          <Workspace />
        </ProjectProvider>
      </SystemStatusProvider>
    </ErrorBoundary>
  );
}
