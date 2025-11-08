// import { useState } from "react";
// import reactLogo from "./assets/react.svg";
import useAnimationFrame from "use-animation-frame";
import { invoke } from "@tauri-apps/api/core";
import {
  // warn,
  // debug,
  // trace,
  info,
  error,
  // attachConsole,
  // attachLogger,
} from "@tauri-apps/plugin-log";
import "./App.css";
import { useCallback, useEffect, useRef } from "react";
import useAudio from "./Audio";

const DEFAULT_CONTROLLER = {
  a: false,
  b: false,
  x: false,
  y: false,
  l: false,
  r: false,
  select: false,
  start: false,
  up: false,
  down: false,
  left: false,
  right: false,
};

const KEY_MAP = {
  w: "up",
  a: "left",
  s: "down",
  d: "right",
  b: "a",
  " ": "b",
  n: "y",
  m: "x",
  q: "l",
  e: "r",
  r: "start",
  f: "select",
};

function App() {
  const controllers = useRef([DEFAULT_CONTROLLER, DEFAULT_CONTROLLER]);
  const { start, clear, pushSamples } = useAudio({
    numChannels: 1,
    sampleRate: 32_000,
    bufferSize: 10_000_000,
  });

  const onKeyDown = useCallback((e: KeyboardEvent) => {
    const button = KEY_MAP[e.key as keyof typeof KEY_MAP];
    if (button) {
      controllers.current[0] = {
        ...controllers.current[0],
        [button]: true,
      };
    }
  }, []);

  const onKeyUp = useCallback((e: KeyboardEvent) => {
    const button = KEY_MAP[e.key as keyof typeof KEY_MAP];
    if (button) {
      controllers.current[0] = {
        ...controllers.current[0],
        [button]: false,
      };
    }
  }, []);

  useEffect(() => {
    window.addEventListener("keydown", onKeyDown);
    window.addEventListener("keyup", onKeyUp);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
      window.removeEventListener("keyup", onKeyUp);
    };
  }, []);
  async function on_file_load(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.item(0);
    if (!file) {
      return;
    }
    info(`Loading ROM file: ${file.name}`);
    const arrayBuffer = await file.arrayBuffer();
    await invoke("load_rom", { romData: arrayBuffer });
    start();
  }

  async function run_frame() {
    // info("Running frame update");
    const pixelData = await invoke("update_emulator", {
      userInput: {
        controllers: controllers.current,
        reset: false,
      },
    });
    const ctx = (
      document.getElementById("canvas") as HTMLCanvasElement
    ).getContext("2d");
    if (ctx) {
      const imageData = new ImageData(
        new Uint8ClampedArray(pixelData as ArrayBuffer),
        256,
        240
      );
      ctx.putImageData(imageData, 0, 0);
    }
    const data = new Uint8ClampedArray(
      (await invoke("get_audio_samples")) as ArrayBuffer
    );
    const float_data = new Float32Array(
      data.buffer,
      data.byteOffset,
      data.byteLength / Float32Array.BYTES_PER_ELEMENT
    );
    if (float_data.some((s) => s > 1.0 || s < -1.0)) {
      error("Invalid audio sample received, skipping frame");
      return;
    }
    const final_arr = float_data.map((s) => 10.0 * s);
    // const float_data = new Float32Array(data).map(
    //   (s) => (s / 255.0) * 2.0 - 1.0
    // );
    pushSamples([final_arr], final_arr.length);
  }

  useAnimationFrame(run_frame);

  return (
    <main className="container">
      <h1>Welcome to Tauri + React</h1>
      <canvas id="canvas" width="256" height="240" className="canvas" />
      <input onChange={on_file_load} type="file" />
    </main>
  );
}

export default App;
