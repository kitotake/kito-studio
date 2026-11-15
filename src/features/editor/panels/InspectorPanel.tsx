import { Panel } from "@/components/ui/Panel";
import { formatFrameRate, formatResolution, formatSampleRateHz } from "@/features/project/format";
import type { ProjectProfile } from "@/types/project";

export function InspectorPanel({ project }: { project: ProjectProfile }) {
  const properties: ReadonlyArray<readonly [label: string, value: string]> = [
    ["Nom du projet", project.name],
    ["Résolution", formatResolution(project)],
    ["Fréquence", formatFrameRate(project)],
    ["Échantillonnage audio", formatSampleRateHz(project)]
  ];

  return (
    <Panel className="inspector-panel" title="PROPRIÉTÉS">
      {properties.map(([label, value]) => (
        <div className="property" key={label}>
          <small>{label}</small>
          <strong>{value}</strong>
        </div>
      ))}
    </Panel>
  );
}
