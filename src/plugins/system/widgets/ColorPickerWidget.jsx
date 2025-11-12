import { createSignal, For } from 'solid-js';
import { IconPalette, IconCopy, IconCheck } from '@tabler/icons-solidjs';

export default function ColorPickerWidget() {
  const [selectedColor, setSelectedColor] = createSignal('#3b82f6');
  const [copied, setCopied] = createSignal(false);
  const [recentColors, setRecentColors] = createSignal([]);

  const addToRecent = (color) => {
    const recent = recentColors();
    if (recent.includes(color)) return;

    const updated = [color, ...recent.slice(0, 7)]; // Keep last 8 colors
    setRecentColors(updated);
    localStorage.setItem('color_picker_recent', JSON.stringify(updated));
  };

  const handleColorChange = (e) => {
    const color = e.target.value;
    setSelectedColor(color);
  };

  const handleColorSelect = (color) => {
    setSelectedColor(color);
    addToRecent(color);
  };

  const copyToClipboard = async (text) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  };

  const hexToRgb = (hex) => {
    const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
    return result
      ? {
          r: parseInt(result[1], 16),
          g: parseInt(result[2], 16),
          b: parseInt(result[3], 16)
        }
      : null;
  };

  const hexToHsl = (hex) => {
    const rgb = hexToRgb(hex);
    if (!rgb) return null;

    const r = rgb.r / 255;
    const g = rgb.g / 255;
    const b = rgb.b / 255;

    const max = Math.max(r, g, b);
    const min = Math.min(r, g, b);
    let h, s, l = (max + min) / 2;

    if (max === min) {
      h = s = 0;
    } else {
      const d = max - min;
      s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
      switch (max) {
        case r: h = ((g - b) / d + (g < b ? 6 : 0)) / 6; break;
        case g: h = ((b - r) / d + 2) / 6; break;
        case b: h = ((r - g) / d + 4) / 6; break;
      }
    }

    return {
      h: Math.round(h * 360),
      s: Math.round(s * 100),
      l: Math.round(l * 100)
    };
  };

  const presetColors = [
    '#ef4444', '#f97316', '#f59e0b', '#eab308',
    '#84cc16', '#22c55e', '#10b981', '#14b8a6',
    '#06b6d4', '#0ea5e9', '#3b82f6', '#6366f1',
    '#8b5cf6', '#a855f7', '#d946ef', '#ec4899',
    '#f43f5e', '#64748b', '#475569', '#1e293b'
  ];

  const ColorFormat = (props) => {
    const rgb = hexToRgb(selectedColor());
    const hsl = hexToHsl(selectedColor());

    let value = '';
    switch (props.format) {
      case 'hex':
        value = selectedColor().toUpperCase();
        break;
      case 'rgb':
        value = rgb ? `rgb(${rgb.r}, ${rgb.g}, ${rgb.b})` : '';
        break;
      case 'hsl':
        value = hsl ? `hsl(${hsl.h}, ${hsl.s}%, ${hsl.l}%)` : '';
        break;
    }

    return (
      <div class="bg-base-200/50 rounded p-1.5 flex items-center justify-between gap-2">
        <div class="text-xs font-mono flex-1 truncate" title={value}>
          {value}
        </div>
        <button
          class="btn btn-xs btn-ghost p-1 h-auto min-h-0"
          onClick={() => copyToClipboard(value)}
          title="Copy"
        >
          {copied() ? <IconCheck size={12} class="text-success" /> : <IconCopy size={12} />}
        </button>
      </div>
    );
  };

  return (
    <div class="card bg-gradient-to-br from-purple-500/20 to-purple-500/5 bg-base-100 shadow-lg h-full flex flex-col p-3">
      {/* Header */}
      <div class="flex items-center gap-1.5 mb-2">
        <IconPalette size={16} class="text-purple-500 opacity-80" />
        <span class="text-xs font-medium opacity-70">Color Picker</span>
      </div>

      {/* Color Preview */}
      <div class="flex gap-2 mb-3">
        <div
          class="w-20 h-20 rounded-lg border-2 border-base-content/20 shadow-inner cursor-pointer"
          style={{ 'background-color': selectedColor() }}
          onClick={() => document.getElementById('color-input').click()}
        />
        <div class="flex-1 flex flex-col gap-1">
          <input
            id="color-input"
            type="color"
            value={selectedColor()}
            onInput={handleColorChange}
            onChange={(e) => addToRecent(e.target.value)}
            class="w-full h-8 rounded cursor-pointer"
            style="border: 2px solid var(--fallback-bc,oklch(var(--bc)/0.2));"
          />
          <input
            type="text"
            value={selectedColor()}
            onInput={(e) => handleColorChange(e)}
            class="input input-xs input-bordered bg-base-200/50 font-mono text-xs"
            placeholder="#000000"
          />
        </div>
      </div>

      {/* Color Formats */}
      <div class="space-y-1 mb-3">
        <ColorFormat format="hex" />
        <ColorFormat format="rgb" />
        <ColorFormat format="hsl" />
      </div>

      {/* Preset Colors */}
      <div class="mb-2">
        <div class="text-xs opacity-50 mb-1.5">Presets</div>
        <div class="grid grid-cols-10 gap-1">
          <For each={presetColors}>
            {(color) => (
              <button
                class="w-full aspect-square rounded border border-base-content/20 hover:scale-110 transition-transform"
                style={{ 'background-color': color }}
                onClick={() => handleColorSelect(color)}
                title={color}
              />
            )}
          </For>
        </div>
      </div>

      {/* Recent Colors */}
      {recentColors().length > 0 && (
        <div>
          <div class="text-xs opacity-50 mb-1.5">Recent</div>
          <div class="grid grid-cols-8 gap-1">
            <For each={recentColors()}>
              {(color) => (
                <button
                  class="w-full aspect-square rounded border border-base-content/20 hover:scale-110 transition-transform"
                  style={{ 'background-color': color }}
                  onClick={() => handleColorSelect(color)}
                  title={color}
                />
              )}
            </For>
          </div>
        </div>
      )}
    </div>
  );
}
