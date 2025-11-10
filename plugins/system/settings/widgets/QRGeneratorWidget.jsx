import { createSignal, createEffect, onMount } from 'solid-js';
import { IconQrcode, IconDownload, IconCopy, IconCheck } from '@tabler/icons-solidjs';
import QRCode from 'qrcode';

export default function QRGeneratorWidget() {
  const [text, setText] = createSignal('https://webarcade.dev');
  const [bgColor, setBgColor] = createSignal('#ffffff');
  const [fgColor, setFgColor] = createSignal('#000000');
  const [level, setLevel] = createSignal('M');
  const [copied, setCopied] = createSignal(false);
  let canvasRef;
  let containerRef;

  // Load saved text from localStorage
  onMount(() => {
    const savedText = localStorage.getItem('qr_generator_text');
    if (savedText) {
      setText(savedText);
    }
  });

  // Generate QR code whenever any parameter changes
  createEffect(() => {
    if (canvasRef && containerRef) {
      const containerWidth = containerRef.offsetWidth - 24; // Subtract padding
      QRCode.toCanvas(canvasRef, text(), {
        width: containerWidth,
        margin: 2,
        color: {
          dark: fgColor(),
          light: bgColor()
        },
        errorCorrectionLevel: level()
      });
    }
  });

  const handleTextChange = (e) => {
    const value = e.target.value;
    setText(value);
    localStorage.setItem('qr_generator_text', value);
  };

  const downloadQR = () => {
    if (!canvasRef) return;

    const link = document.createElement('a');
    link.download = 'qrcode.png';
    link.href = canvasRef.toDataURL();
    link.click();
  };

  const copyToClipboard = async () => {
    try {
      await navigator.clipboard.writeText(text());
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  };

  return (
    <div class="card bg-gradient-to-br from-info/20 to-info/5 bg-base-100 shadow-lg h-full flex flex-col p-3">
      {/* Header */}
      <div class="flex items-center gap-1.5 mb-2">
        <IconQrcode size={16} class="text-info opacity-80" />
        <span class="text-xs font-medium opacity-70">QR Generator</span>
      </div>

      {/* QR Code Display */}
      <div ref={containerRef} class="flex justify-center items-center mb-3 bg-base-200/50 rounded-lg p-3">
        <canvas
          ref={canvasRef}
          class="rounded bg-white w-full"
        />
      </div>

      {/* Input */}
      <div class="mb-2">
        <label class="text-xs opacity-50 mb-1 block">Text / URL</label>
        <div class="flex gap-1">
          <input
            type="text"
            value={text()}
            onInput={handleTextChange}
            class="input input-sm input-bordered bg-base-200/50 flex-1 text-xs"
            placeholder="Enter text or URL"
          />
          <button
            class="btn btn-sm btn-ghost p-1 min-h-0 h-auto"
            onClick={copyToClipboard}
            title="Copy text"
          >
            {copied() ? <IconCheck size={16} class="text-success" /> : <IconCopy size={16} />}
          </button>
        </div>
      </div>

      {/* Error Correction Level */}
      <div class="mb-2">
        <label class="text-xs opacity-50 mb-1 block">Error Correction</label>
        <div class="flex gap-1">
          {['L', 'M', 'Q', 'H'].map((l) => (
            <button
              class={`btn btn-xs flex-1 ${level() === l ? 'btn-info' : 'btn-ghost'}`}
              onClick={() => setLevel(l)}
            >
              {l}
            </button>
          ))}
        </div>
      </div>

      {/* Colors */}
      <div class="grid grid-cols-2 gap-2 mb-2">
        <div>
          <label class="text-xs opacity-50 mb-1 block">Foreground</label>
          <input
            type="color"
            value={fgColor()}
            onInput={(e) => setFgColor(e.target.value)}
            class="w-full h-8 rounded cursor-pointer"
            style="border: 2px solid var(--fallback-bc,oklch(var(--bc)/0.2));"
          />
        </div>
        <div>
          <label class="text-xs opacity-50 mb-1 block">Background</label>
          <input
            type="color"
            value={bgColor()}
            onInput={(e) => setBgColor(e.target.value)}
            class="w-full h-8 rounded cursor-pointer"
            style="border: 2px solid var(--fallback-bc,oklch(var(--bc)/0.2));"
          />
        </div>
      </div>

      {/* Download Button */}
      <button
        class="btn btn-sm btn-info gap-1 mt-auto"
        onClick={downloadQR}
      >
        <IconDownload size={16} />
        Download PNG
      </button>
    </div>
  );
}
