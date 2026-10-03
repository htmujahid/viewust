import type { Kind } from "./kinds";

const ART: Record<
  Kind,
  { box: [number, number, number, number]; cm?: number; px?: number; boost?: number }
> = {
  computer: { box: [48, 2, 64, 97], cm: 45, boost: 1.7 },
  monitor: { box: [18, 6, 124, 86], cm: 55 },
  keyboard: { box: [8, 22, 144, 58], cm: 44 },
  mouse: { box: [46, 6, 68, 88], cm: 11.5 },
  webcam: { box: [44, 4, 72, 88], cm: 9.5 },
  audio: { box: [22, 8, 116, 76], cm: 8.4 },
  microphone: { box: [48, 4, 64, 88], cm: 9.5 },
  gamepad: { box: [14, 30, 132, 58], cm: 16 },
  storage: { box: [12, 26, 138, 48], cm: 6.5 },
  printer: { box: [16, 6, 128, 88], cm: 40 },
  hub: { box: [12, 8, 136, 70], cm: 12 },
  wireless: { box: [12, 28, 142, 44], cm: 4.5 },
  phone: { box: [52, 2, 56, 96], cm: 15 },
  securitykey: { box: [18, 28, 124, 44], cm: 6 },
  generic: { box: [28, 28, 122, 44], cm: 8 },
  board: { box: [0, 0, 160, 160], px: 330 },
  cpu: { box: [0, 0, 100, 100], px: 100 },
  ram: { box: [0, 0, 160, 44], px: 150 },
  gpu: { box: [0, 0, 200, 92], px: 250 },
  nvme: { box: [0, 0, 140, 44], px: 130 },
  ssd: { box: [0, 0, 120, 90], px: 115 },
  hdd: { box: [0, 0, 140, 100], px: 130 },
  psu: { box: [0, 0, 150, 110], px: 150 },
  nic: { box: [0, 0, 140, 90], px: 115 },
  soundcard: { box: [0, 0, 140, 90], px: 115 },
  router: { box: [0, 0, 150, 100], px: 130 },
  internet: { box: [18, 6, 132, 82], px: 150 },
};

const REF_CM = 45;
const REF_PX = 200;
const GAMMA = 0.85;
const MIN_PX = 40;

export function artSize(kind: Kind, cm?: number): { width: number; height: number } {
  const { box, cm: defaultCm = 10, px: fixed, boost = 1 } = ART[kind];
  const real = Math.min(Math.max(cm ?? defaultCm, 2), 120);
  const longest = Math.max(box[2], box[3]);
  const px = fixed ?? Math.max(MIN_PX, REF_PX * (real / REF_CM) ** GAMMA) * boost;
  const scale = px / longest;
  return { width: Math.round(box[2] * scale), height: Math.round(box[3] * scale) };
}

export function artBox(kind: Kind): [number, number, number, number] {
  return ART[kind].box;
}
