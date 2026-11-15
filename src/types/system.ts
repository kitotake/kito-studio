export type RuntimeInfo = {
  appName: string;
  appVersion: string;
  os: string;
  architecture: string;
};

export type MediaToolsStatus = {
  ffmpegFound: boolean;
  ffprobeFound: boolean;
};
