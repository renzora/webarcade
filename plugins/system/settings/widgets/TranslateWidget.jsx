import { createSignal, Show } from 'solid-js';
import { IconLanguage } from '@tabler/icons-solidjs';

export default function TranslateWidget() {
  const [fromLang, setFromLang] = createSignal('auto');
  const [toLang, setToLang] = createSignal('es');
  const [inputText, setInputText] = createSignal('');
  const [translatedText, setTranslatedText] = createSignal('');
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal(null);

  // Popular languages
  const languages = [
    { code: 'auto', name: 'Auto Detect' },
    { code: 'en', name: 'English' },
    { code: 'es', name: 'Spanish' },
    { code: 'fr', name: 'French' },
    { code: 'de', name: 'German' },
    { code: 'it', name: 'Italian' },
    { code: 'pt', name: 'Portuguese' },
    { code: 'ru', name: 'Russian' },
    { code: 'ja', name: 'Japanese' },
    { code: 'ko', name: 'Korean' },
    { code: 'zh', name: 'Chinese (Simplified)' },
    { code: 'zh-TW', name: 'Chinese (Traditional)' },
    { code: 'ar', name: 'Arabic' },
    { code: 'hi', name: 'Hindi' },
    { code: 'nl', name: 'Dutch' },
    { code: 'pl', name: 'Polish' },
    { code: 'tr', name: 'Turkish' },
    { code: 'vi', name: 'Vietnamese' },
    { code: 'th', name: 'Thai' },
    { code: 'sv', name: 'Swedish' }
  ];

  const translate = async () => {
    if (!inputText().trim()) {
      setTranslatedText('');
      return;
    }

    setLoading(true);
    setError(null);

    try {
      // Using MyMemory Translation API (free, no API key required)
      // Note: Limited to 1000 words/day per IP
      const encodedText = encodeURIComponent(inputText());
      const langPair = fromLang() === 'auto'
        ? toLang()
        : `${fromLang()}|${toLang()}`;

      const url = `https://api.mymemory.translated.net/get?q=${encodedText}&langpair=${langPair}`;

      const response = await fetch(url);

      if (!response.ok) {
        throw new Error('Translation failed');
      }

      const data = await response.json();

      if (data.responseStatus === 200 || data.responseData) {
        setTranslatedText(data.responseData.translatedText);

        // Show match quality if available
        if (data.responseData.match < 0.5) {
          setError('Translation quality may be low');
        }
      } else {
        throw new Error(data.responseDetails || 'Translation failed');
      }
    } catch (err) {
      setError(err.message);
      setTranslatedText('');
    } finally {
      setLoading(false);
    }
  };

  const handleInputChange = (e) => {
    setInputText(e.target.value);
  };

  const swapLanguages = () => {
    if (fromLang() === 'auto') return;

    const temp = fromLang();
    setFromLang(toLang());
    setToLang(temp);

    // Swap text too
    const tempText = inputText();
    setInputText(translatedText());
    setTranslatedText(tempText);
  };

  const copyToClipboard = () => {
    if (translatedText()) {
      navigator.clipboard.writeText(translatedText());
    }
  };

  return (
    <div class="card bg-gradient-to-br from-blue-500/20 to-blue-500/5 bg-base-100 shadow-lg h-full flex flex-col p-3">
      {/* Header */}
      <div class="flex items-center gap-1.5 mb-2">
        <IconLanguage size={16} class="text-blue-500 opacity-80" />
        <span class="text-xs font-medium opacity-70">Translate</span>
      </div>

      <div class="flex-1 flex flex-col">


        {/* Language Selection */}
        <div class="flex gap-2 mb-3 items-center">
          <select
            class="select select-bordered select-sm flex-1"
            value={fromLang()}
            onChange={(e) => setFromLang(e.target.value)}
          >
            {languages.map(lang => (
              <option value={lang.code}>{lang.name}</option>
            ))}
          </select>

          <button
            class="btn btn-sm btn-ghost btn-circle"
            onClick={swapLanguages}
            disabled={fromLang() === 'auto'}
            title="Swap languages"
          >
            ⇄
          </button>

          <select
            class="select select-bordered select-sm flex-1"
            value={toLang()}
            onChange={(e) => setToLang(e.target.value)}
          >
            {languages.filter(lang => lang.code !== 'auto').map(lang => (
              <option value={lang.code}>{lang.name}</option>
            ))}
          </select>
        </div>

        {/* Input Text */}
        <div class="form-control mb-3">
          <label class="label py-1">
            <span class="label-text text-xs">Text to translate</span>
          </label>
          <textarea
            class="textarea textarea-bordered textarea-sm h-20 resize-none"
            placeholder="Enter text..."
            value={inputText()}
            onInput={handleInputChange}
          ></textarea>
        </div>

        {/* Translate Button */}
        <button
          class="btn btn-sm btn-primary mb-3"
          onClick={translate}
          disabled={loading() || !inputText().trim()}
        >
          <Show when={loading()}>
            <span class="loading loading-spinner loading-xs"></span>
          </Show>
          Translate
        </button>

        <Show when={error()}>
          <div class="alert alert-warning alert-sm mb-3">
            <span class="text-xs">{error()}</span>
          </div>
        </Show>

        {/* Output Text */}
        <Show when={translatedText()}>
          <div class="form-control flex-1">
            <div class="flex justify-between items-center mb-1">
              <label class="label py-1">
                <span class="label-text text-xs">Translation</span>
              </label>
              <button
                class="btn btn-xs btn-ghost"
                onClick={copyToClipboard}
                title="Copy to clipboard"
              >
                📋
              </button>
            </div>
            <div class="textarea textarea-bordered textarea-sm h-20 overflow-auto bg-base-300">
              {translatedText()}
            </div>
          </div>
        </Show>

        {/* API Info */}
        <div class="text-xs opacity-50 mt-auto pt-2">
          Powered by MyMemory API
        </div>
      </div>
    </div>
  );
}
