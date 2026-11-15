import { formatFrameRate, formatResolution, formatSampleRateKhz } from "@/features/project/format";
import type { ProjectProfile } from "@/types/project";
import { InspectorPanel } from "./panels/InspectorPanel";
import { MediaPanel } from "./panels/MediaPanel";
import { PreviewPanel } from "./panels/PreviewPanel";
import { TimelinePanel } from "./panels/TimelinePanel";

export function EditorScreen({ project }: { project: ProjectProfile }) {
  return (
    <div className="editor-content">
      <div className="editor-heading">
        <div>
          <div className="eyebrow">ESPACE DE MONTAGE</div>
          <h1>{project.name}</h1>
          <p>
            {formatResolution(project)} · {formatFrameRate(project)} · Audio{" "}
            {formatSampleRateKhz(project)}
          </p>
        </div>
        <span className="phase-tag">ESQUISSE ÉDITEUR</span>
      </div>
      <div className="editor-grid">
        <MediaPanel />
        <PreviewPanel />
        <InspectorPanel project={project} />
        <TimelinePanel />
      </div>
    </div>
  );
}
