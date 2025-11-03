import { useState } from "react";
import reactLogo from "./assets/react.svg";
import useAnimationFrame from "use-animation-frame";
import { invoke } from "@tauri-apps/api/core";
import {
  warn,
  debug,
  trace,
  info,
  error,
  attachConsole,
  attachLogger,
} from "@tauri-apps/plugin-log";
import "./App.css";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function run_frame() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
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
    </main>
  );
}

export default App;
