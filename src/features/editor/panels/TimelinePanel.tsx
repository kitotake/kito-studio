import { Panel } from "@/components/ui/Panel";

const RULER_MARKS = ["00:00", "00:10", "00:20", "00:30", "00:40", "00:50"] as const;
const TRACKS = ["V2", "V1", "A1", "A2"] as const;

export function TimelinePanel() {
  return (
    <Panel className="timeline-panel" title="TIMELINE" aside="SÉQUENCE 01">
      <div className="timeline-ruler">
        {RULER_MARKS.map((mark) => (
          <span key={mark}>{mark}</span>
        ))}
      </div>
      {TRACKS.map((track) => (
        <div className="track" key={track}>
          <div className="track-label">{track}</div>
          <div className="track-lane">
            <div className="track-placeholder">
              {track.startsWith("V") ? "Déposez un clip vidéo ici" : "Déposez un clip audio ici"}
            </div>
          </div>
        </div>
      ))}
    </Panel>
  );
}
