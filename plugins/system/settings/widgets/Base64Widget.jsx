import { createSignal } from 'solid-js';
import { IconBinaryTree } from '@tabler/icons-solidjs';

export default function Base64Widget() {
  const [mode, setMode] = createSignal('encode');
  const [input, setInput] = createSignal('');
  const [output, setOutput] = createSignal('');
  const [error, setError] = createSignal(null);

  const encode = (text) => {
    try {
      setError(null);
      // Handle Unicode properly
      const encoded = btoa(unescape(encodeURIComponent(text)));
      setOutput(encoded);
    } catch (err) {
      setError('Encoding failed: ' + err.message);
      setOutput('');
    }
  };

  const decode = (text) => {
    try {
      setError(null);
      // Handle Unicode properly
      const decoded = decodeURIComponent(escape(atob(text)));
      setOutput(decoded);
    } catch (err) {
      setError('Decoding failed: Invalid Base64 string');
      setOutput('');
    }
  };

  const handleInputChange = (e) => {
    const value = e.target.value;
    setInput(value);

    if (!value.trim()) {
      setOutput('');
      setError(null);
      return;
    }

    if (mode() === 'encode') {
      encode(value);
    } else {
      decode(value);
    }
  };

  const handleModeChange = (newMode) => {
    setMode(newMode);
    setError(null);

    if (!input().trim()) {
      setOutput('');
      return;
    }

    if (newMode === 'encode') {
      encode(input());
    } else {
      decode(input());
    }
  };

  const swapMode = () => {
    // Swap mode and texts
    const newMode = mode() === 'encode' ? 'decode' : 'encode';
    setMode(newMode);
    setInput(output());
    setOutput(input());
    setError(null);
  };

  const copyToClipboard = () => {
    if (output()) {
      navigator.clipboard.writeText(output());
    }
  };

  const clearAll = () => {
    setInput('');
    setOutput('');
    setError(null);
  };

  return (
    <div class="card bg-gradient-to-br from-orange-500/20 to-orange-500/5 bg-base-100 shadow-lg h-full flex flex-col p-3">
      {/* Header */}
      <div class="flex items-center gap-1.5 mb-2">
        <IconBinaryTree size={16} class="text-orange-500 opacity-80" />
        <span class="text-xs font-medium opacity-70">Base64</span>
      </div>

      <div class="flex-1 flex flex-col">


        {/* Mode Selection */}
        <div class="flex gap-2 mb-3">
          <button
            class={`btn btn-sm flex-1 ${mode() === 'encode' ? 'btn-primary' : 'btn-outline'}`}
            onClick={() => handleModeChange('encode')}
          >
            Encode
          </button>
          <button
            class={`btn btn-sm flex-1 ${mode() === 'decode' ? 'btn-primary' : 'btn-outline'}`}
            onClick={() => handleModeChange('decode')}
          >
            Decode
          </button>
        </div>

        {/* Input */}
        <div class="form-control mb-3 flex-1">
          <div class="flex justify-between items-center mb-1">
            <label class="label py-1">
              <span class="label-text text-xs">
                {mode() === 'encode' ? 'Plain Text' : 'Base64 String'}
              </span>
            </label>
            <span class="text-xs opacity-50">
              {input().length} chars
            </span>
          </div>
          <textarea
            class="textarea textarea-bordered textarea-sm flex-1 resize-none font-mono text-xs"
            placeholder={mode() === 'encode' ? 'Enter text to encode...' : 'Enter Base64 to decode...'}
            value={input()}
            onInput={handleInputChange}
          ></textarea>
        </div>

        {/* Swap Button */}
        <button
          class="btn btn-sm btn-ghost btn-circle self-center mb-2"
          onClick={swapMode}
          disabled={!output()}
          title="Swap input/output"
        >
          ⇅
        </button>

        {/* Output */}
        <div class="form-control mb-3 flex-1">
          <div class="flex justify-between items-center mb-1">
            <label class="label py-1">
              <span class="label-text text-xs">
                {mode() === 'encode' ? 'Base64 String' : 'Plain Text'}
              </span>
            </label>
            <div class="flex gap-1">
              <span class="text-xs opacity-50">
                {output().length} chars
              </span>
              {output() && (
                <button
                  class="btn btn-xs btn-ghost"
                  onClick={copyToClipboard}
                  title="Copy to clipboard"
                >
                  📋
                </button>
              )}
            </div>
          </div>
          <textarea
            class="textarea textarea-bordered textarea-sm flex-1 resize-none font-mono text-xs bg-base-300"
            value={output()}
            readOnly
            placeholder={error() || 'Output will appear here...'}
          ></textarea>
        </div>

        {error() && (
          <div class="alert alert-error alert-sm mb-3">
            <span class="text-xs">{error()}</span>
          </div>
        )}

        {/* Action Buttons */}
        <div class="flex gap-2">
          <button
            class="btn btn-sm btn-outline flex-1"
            onClick={clearAll}
            disabled={!input() && !output()}
          >
            Clear
          </button>
          {output() && (
            <button
              class="btn btn-sm btn-success flex-1"
              onClick={copyToClipboard}
            >
              Copy Output
            </button>
          )}
        </div>

        {/* Info */}
        <div class="text-xs opacity-50 mt-2">
          Supports Unicode characters (UTF-8)
        </div>
      </div>
    </div>
  );
}
