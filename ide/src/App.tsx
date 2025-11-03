// import { useState } from "react";
// import reactLogo from "./assets/react.svg";
import useAnimationFrame from "use-animation-frame";
import { invoke } from "@tauri-apps/api/core";
import {
  // warn,
  // debug,
  // trace,
  info,
  // error,
  // attachConsole,
  // attachLogger,
} from "@tauri-apps/plugin-log";
import "./App.css";

function App() {
  // const [greetMsg, setGreetMsg] = useState("");
  // const [name, setName] = useState("");

  async function on_file_load(e: React.ChangeEvent<HTMLInputElement>) {
    info("File load triggered");
    const file = e.target.files?.item(0);
    if (!file) {
      return;
    }
    info(`Loading ROM file: ${file.name}`);
    const arrayBuffer = await file.arrayBuffer();
    await invoke("load_rom", { romData: arrayBuffer });
  }

  async function run_frame() {
    info("Running frame update");
    const pixelData = await invoke("update_emulator", { durationMillis: 16 });
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
