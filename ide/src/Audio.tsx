import React, { useRef, useEffect } from "react";
import { info, error } from "@tauri-apps/plugin-log";
import AudioNodeSrc from "./AudioNode.js?raw";

const AUDIO_SRC_BLOB = new Blob([AudioNodeSrc], {
  type: "application/javascript",
});
const AUDIO_NODE_URL = URL.createObjectURL(AUDIO_SRC_BLOB);

const useAudio = ({
  numChannels,
  sampleRate,
  bufferSize = sampleRate,
}: {
  numChannels: number;
  sampleRate: number;
  bufferSize?: number;
}) => {
  const audioContext = useRef<AudioContext | null>(null);
  const audioNode = useRef<AudioWorkletNode | null>(null);
  // Start the audio
  const start = () => {
    info("Starting audio");
    info(AudioNodeSrc);
    info("Setting up audio context");
    // Set up audio context
    const ctx = new (window.AudioContext || window.webkitAudioContext)({
      sampleRate,
    });
    ctx.audioWorklet
      .addModule(AUDIO_NODE_URL)
      .then(() => {
        info("Creating audio node");
        const node = new AudioWorkletNode(ctx, "ring-buffer-queue-source", {
          processorOptions: {
            capacity: bufferSize,
          },
        });
        node.connect(ctx.destination);
        audioNode.current = node;
      })
      .catch((e) => {
        error("Failed to load audio worklet module: " + e);
      });

    audioContext.current = ctx;
  };
  const clear = () => {};
  useEffect(() => {
    // Create source node
    return () => {
      //   audioContext.current?.close();
    };
  }, []);

  const pushSamples = (channelData: Float32Array[], length: number) => {
    info("Audio node is " + audioNode.current);
    audioNode.current?.port.postMessage(channelData[0], [
      channelData[0].buffer,
    ]);
  };
  return {
    start,
    pushSamples,
    clear,
  };
};

export default useAudio;
