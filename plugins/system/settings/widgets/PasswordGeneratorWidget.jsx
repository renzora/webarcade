import { createSignal, onMount } from 'solid-js';
import { IconKey } from '@tabler/icons-solidjs';

export default function PasswordGeneratorWidget() {
  const [password, setPassword] = createSignal('');
  const [length, setLength] = createSignal(16);
  const [includeUppercase, setIncludeUppercase] = createSignal(true);
  const [includeLowercase, setIncludeLowercase] = createSignal(true);
  const [includeNumbers, setIncludeNumbers] = createSignal(true);
  const [includeSymbols, setIncludeSymbols] = createSignal(true);
  const [strength, setStrength] = createSignal('');
  const [copied, setCopied] = createSignal(false);

  const lowercase = 'abcdefghijklmnopqrstuvwxyz';
  const uppercase = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ';
  const numbers = '0123456789';
  const symbols = '!@#$%^&*()_+-=[]{}|;:,.<>?';

  const generatePassword = () => {
    let charset = '';
    let generatedPassword = '';

    if (includeLowercase()) charset += lowercase;
    if (includeUppercase()) charset += uppercase;
    if (includeNumbers()) charset += numbers;
    if (includeSymbols()) charset += symbols;

    if (charset === '') {
      setPassword('Select at least one option');
      setStrength('');
      return;
    }

    // Generate password with crypto.getRandomValues for better randomness
    const array = new Uint32Array(length());
    crypto.getRandomValues(array);

    for (let i = 0; i < length(); i++) {
      generatedPassword += charset[array[i] % charset.length];
    }

    setPassword(generatedPassword);
    calculateStrength(generatedPassword);
    setCopied(false);
  };

  const calculateStrength = (pwd) => {
    let score = 0;
    const len = pwd.length;

    // Length score
    if (len >= 8) score += 1;
    if (len >= 12) score += 1;
    if (len >= 16) score += 1;

    // Character variety score
    if (/[a-z]/.test(pwd)) score += 1;
    if (/[A-Z]/.test(pwd)) score += 1;
    if (/[0-9]/.test(pwd)) score += 1;
    if (/[^a-zA-Z0-9]/.test(pwd)) score += 1;

    if (score <= 2) setStrength('Weak');
    else if (score <= 4) setStrength('Medium');
    else if (score <= 6) setStrength('Strong');
    else setStrength('Very Strong');
  };

  const copyToClipboard = () => {
    if (password()) {
      navigator.clipboard.writeText(password());
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const getStrengthColor = () => {
    switch (strength()) {
      case 'Weak': return 'text-error';
      case 'Medium': return 'text-warning';
      case 'Strong': return 'text-info';
      case 'Very Strong': return 'text-success';
      default: return '';
    }
  };

  const getStrengthBadge = () => {
    switch (strength()) {
      case 'Weak': return 'badge-error';
      case 'Medium': return 'badge-warning';
      case 'Strong': return 'badge-info';
      case 'Very Strong': return 'badge-success';
      default: return '';
    }
  };

  onMount(() => {
    generatePassword();
  });

  return (
    <div class="card bg-gradient-to-br from-yellow-500/20 to-yellow-500/5 bg-base-100 shadow-lg h-full flex flex-col p-3">
      {/* Header */}
      <div class="flex items-center gap-1.5 mb-2">
        <IconKey size={16} class="text-yellow-500 opacity-80" />
        <span class="text-xs font-medium opacity-70">Password Generator</span>
      </div>

      <div class="flex-1 flex flex-col">


        {/* Password Display */}
        <div class="form-control mb-3">
          <div class="flex justify-between items-center mb-1">
            <label class="label py-1">
              <span class="label-text text-xs">Generated Password</span>
            </label>
            {strength() && (
              <span class={`badge badge-sm ${getStrengthBadge()}`}>
                {strength()}
              </span>
            )}
          </div>
          <div class="flex gap-2">
            <input
              type="text"
              class="input input-bordered input-sm flex-1 font-mono text-xs"
              value={password()}
              readOnly
            />
            <button
              class="btn btn-sm btn-square btn-primary"
              onClick={copyToClipboard}
              title="Copy to clipboard"
            >
              {copied() ? '✓' : '📋'}
            </button>
          </div>
        </div>

        {/* Length Slider */}
        <div class="form-control mb-3">
          <div class="flex justify-between items-center mb-1">
            <label class="label py-1">
              <span class="label-text text-xs">Length</span>
            </label>
            <span class="badge badge-sm badge-neutral">{length()}</span>
          </div>
          <input
            type="range"
            min="4"
            max="64"
            value={length()}
            class="range range-primary range-sm"
            onInput={(e) => setLength(parseInt(e.target.value))}
          />
          <div class="w-full flex justify-between text-xs px-2 opacity-50">
            <span>4</span>
            <span>16</span>
            <span>32</span>
            <span>64</span>
          </div>
        </div>

        {/* Options */}
        <div class="form-control mb-3">
          <label class="label py-1">
            <span class="label-text text-xs">Include</span>
          </label>
          <div class="space-y-2">
            <label class="label cursor-pointer py-1">
              <span class="label-text text-xs">Uppercase (A-Z)</span>
              <input
                type="checkbox"
                class="checkbox checkbox-sm checkbox-primary"
                checked={includeUppercase()}
                onChange={(e) => setIncludeUppercase(e.target.checked)}
              />
            </label>
            <label class="label cursor-pointer py-1">
              <span class="label-text text-xs">Lowercase (a-z)</span>
              <input
                type="checkbox"
                class="checkbox checkbox-sm checkbox-primary"
                checked={includeLowercase()}
                onChange={(e) => setIncludeLowercase(e.target.checked)}
              />
            </label>
            <label class="label cursor-pointer py-1">
              <span class="label-text text-xs">Numbers (0-9)</span>
              <input
                type="checkbox"
                class="checkbox checkbox-sm checkbox-primary"
                checked={includeNumbers()}
                onChange={(e) => setIncludeNumbers(e.target.checked)}
              />
            </label>
            <label class="label cursor-pointer py-1">
              <span class="label-text text-xs">Symbols (!@#$...)</span>
              <input
                type="checkbox"
                class="checkbox checkbox-sm checkbox-primary"
                checked={includeSymbols()}
                onChange={(e) => setIncludeSymbols(e.target.checked)}
              />
            </label>
          </div>
        </div>

        {/* Generate Button */}
        <button
          class="btn btn-sm btn-primary mt-auto"
          onClick={generatePassword}
        >
          🔄 Generate New Password
        </button>

        {/* Security Notice */}
        <div class="text-xs opacity-50 mt-2 text-center">
          Generated using crypto.getRandomValues()
        </div>
      </div>
    </div>
  );
}
