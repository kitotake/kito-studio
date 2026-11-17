import { formatFrameRateValue } from "./format";
import { CUSTOM_PROFILE, type SettingsFormExtras, type SettingsFormValue } from "./settingsForm";
import { FRAME_RATES, SAMPLE_RATES, VIDEO_PROFILES } from "./projectProfiles";
import { validateProjectName } from "./validateProjectName";

type Props = {
  value: SettingsFormValue;
  extras: SettingsFormExtras;
  onChange: (next: SettingsFormValue) => void;
  autoFocusName?: boolean;
};

/** Champs communs à la création et aux paramètres d'un projet. */
export function ProjectSettingsFields({ value, extras, onChange, autoFocusName = false }: Props) {
  const nameError = validateProjectName(value.name);
  const set = <K extends keyof SettingsFormValue>(key: K, next: SettingsFormValue[K]) =>
    onChange({ ...value, [key]: next });

  const frameRates = FRAME_RATES.map((n) => ({ key: `${n}/1`, label: formatFrameRateValue(n, 1) }));
  if (extras.customFrameRate) {
    const { num, den } = extras.customFrameRate;
    frameRates.push({ key: `${num}/${den}`, label: formatFrameRateValue(num, den) });
  }
  const sampleRates = [...SAMPLE_RATES, ...(extras.customSampleRate ? [extras.customSampleRate] : [])];

  return (
    <>
      <label>
        Nom du projet
        <input
          value={value.name}
          onChange={(event) => set("name", event.target.value)}
          autoFocus={autoFocusName}
          aria-invalid={nameError !== null}
          aria-describedby="project-name-error"
        />
        <span id="project-name-error" className="field-error" role={nameError ? "alert" : undefined}>
          {nameError}
        </span>
      </label>

      <label>
        Résolution
        <select value={value.profile} onChange={(event) => set("profile", event.target.value)}>
          {VIDEO_PROFILES.map((profile) => (
            <option value={profile.id} key={profile.id}>
              {profile.label} — {profile.width} × {profile.height}
            </option>
          ))}
          {extras.customResolution ? (
            <option value={CUSTOM_PROFILE}>
              Personnalisée — {extras.customResolution.width} × {extras.customResolution.height}
            </option>
          ) : null}
        </select>
      </label>

      <label>
        Fréquence d'images
        <select value={value.frameRate} onChange={(event) => set("frameRate", event.target.value)}>
          {frameRates.map((rate) => (
            <option value={rate.key} key={rate.key}>
              {rate.label}
            </option>
          ))}
        </select>
      </label>

      <label>
        Fréquence d'échantillonnage audio
        <select value={value.sampleRate} onChange={(event) => set("sampleRate", Number(event.target.value))}>
          {sampleRates.map((rate) => (
            <option value={rate} key={rate}>
              {new Intl.NumberFormat("fr-FR").format(rate / 1000)} kHz
            </option>
          ))}
        </select>
      </label>
    </>
  );
}
