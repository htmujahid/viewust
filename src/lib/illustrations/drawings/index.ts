import type { Component } from "svelte";

import type { Kind } from "../kinds";

import Computer from "./Computer.svelte";
import Monitor from "./Monitor.svelte";
import Keyboard from "./Keyboard.svelte";
import Mouse from "./Mouse.svelte";
import Webcam from "./Webcam.svelte";
import Audio from "./Audio.svelte";
import Microphone from "./Microphone.svelte";
import Gamepad from "./Gamepad.svelte";
import Storage from "./Storage.svelte";
import Printer from "./Printer.svelte";
import Hub from "./Hub.svelte";
import Wireless from "./Wireless.svelte";
import Phone from "./Phone.svelte";
import SecurityKey from "./SecurityKey.svelte";
import Board from "./Board.svelte";
import Cpu from "./Cpu.svelte";
import Ram from "./Ram.svelte";
import Gpu from "./Gpu.svelte";
import Nvme from "./Nvme.svelte";
import Ssd from "./Ssd.svelte";
import Hdd from "./Hdd.svelte";
import Psu from "./Psu.svelte";
import Nic from "./Nic.svelte";
import SoundCard from "./SoundCard.svelte";
import Router from "./Router.svelte";
import Internet from "./Internet.svelte";
import Generic from "./Generic.svelte";

export const drawings: Record<Kind, Component> = {
  computer: Computer,
  monitor: Monitor,
  keyboard: Keyboard,
  mouse: Mouse,
  webcam: Webcam,
  audio: Audio,
  microphone: Microphone,
  gamepad: Gamepad,
  storage: Storage,
  printer: Printer,
  hub: Hub,
  wireless: Wireless,
  phone: Phone,
  securitykey: SecurityKey,
  board: Board,
  cpu: Cpu,
  ram: Ram,
  gpu: Gpu,
  nvme: Nvme,
  ssd: Ssd,
  hdd: Hdd,
  psu: Psu,
  nic: Nic,
  soundcard: SoundCard,
  router: Router,
  internet: Internet,
  generic: Generic,
};
