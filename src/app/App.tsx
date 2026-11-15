import { Sidebar } from "@/components/layout/Sidebar";
import { Topbar } from "@/components/layout/Topbar";
import { EditorScreen } from "@/features/editor/EditorScreen";
import { HomeScreen } from "@/features/home/HomeScreen";
import { CreateProjectDialog } from "@/features/project/CreateProjectDialog";
import { createProjectProfile } from "@/features/project/createProjectProfile";
import { SystemStatusProvider } from "@/features/system/SystemStatusProvider";
import { ErrorBoundary } from "./ErrorBoundary";
import { useWorkspace } from "./useWorkspace";

function Workspace() {
  const { project, view, isCreateOpen, openProject, navigate, openCreateDialog, closeCreateDialog } =
    useWorkspace();

  const showEditor = view === "editor" && project !== null;

  return (
    <main className="app-shell">
      <Sidebar view={view} onNavigate={navigate} />

      <section className="main-area">
        <Topbar title={showEditor ? project.name : "Accueil"} />
        {showEditor ? <EditorScreen project={project} /> : <HomeScreen onCreateProject={openCreateDialog} />}
      </section>

      {isCreateOpen && (
        <CreateProjectDialog
          onSubmit={(input) => openProject(createProjectProfile(input))}
          onClose={closeCreateDialog}
        />
      )}
    </main>
  );
}

export default function App() {
  return (
    <ErrorBoundary>
      <SystemStatusProvider>
        <Workspace />
      </SystemStatusProvider>
    </ErrorBoundary>
  );
}
