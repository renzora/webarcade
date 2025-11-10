import { createSignal, createEffect, onCleanup, Show } from 'solid-js';
import { IconCurrencyDollar } from '@tabler/icons-solidjs';

export default function CurrencyConverterWidget() {
  const [rates, setRates] = createSignal({});
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal(null);
  const [fromCurrency, setFromCurrency] = createSignal('USD');
  const [toCurrency, setToCurrency] = createSignal('EUR');
  const [amount, setAmount] = createSignal('100');
  const [result, setResult] = createSignal('0');
  const [lastUpdate, setLastUpdate] = createSignal('');

  // Popular currencies to show first
  const popularCurrencies = ['USD', 'EUR', 'GBP', 'JPY', 'CAD', 'AUD', 'CHF', 'CNY', 'INR', 'MXN'];

  const fetchRates = async () => {
    try {
      setLoading(true);
      setError(null);

      // Using exchangerate-api.com (free, no API key needed for basic usage)
      const response = await fetch('https://api.exchangerate-api.com/v4/latest/USD');

      if (!response.ok) {
        throw new Error('Failed to fetch exchange rates');
      }

      const data = await response.json();
      setRates(data.rates);
      setLastUpdate(new Date(data.time_last_updated * 1000).toLocaleTimeString());
      setLoading(false);
      convert(data.rates);
    } catch (err) {
      setError(err.message);
      setLoading(false);
    }
  };

  const convert = (currentRates = rates()) => {
    const value = parseFloat(amount());
    if (isNaN(value) || !currentRates[fromCurrency()] || !currentRates[toCurrency()]) {
      setResult('0');
      return;
    }

    // Convert from -> USD -> to
    const inUSD = value / currentRates[fromCurrency()];
    const converted = inUSD * currentRates[toCurrency()];
    setResult(converted.toFixed(2));
  };

  createEffect(() => {
    fetchRates();

    // Update rates every 5 minutes
    const interval = setInterval(fetchRates, 300000);
    onCleanup(() => clearInterval(interval));
  });

  const handleAmountChange = (e) => {
    setAmount(e.target.value);
    convert();
  };

  const handleFromChange = (e) => {
    setFromCurrency(e.target.value);
    convert();
  };

  const handleToChange = (e) => {
    setToCurrency(e.target.value);
    convert();
  };

  const swapCurrencies = () => {
    const temp = fromCurrency();
    setFromCurrency(toCurrency());
    setToCurrency(temp);
    convert();
  };

  const allCurrencies = () => {
    const available = Object.keys(rates());
    // Sort: popular first, then alphabetically
    return available.sort((a, b) => {
      const aPopular = popularCurrencies.includes(a);
      const bPopular = popularCurrencies.includes(b);
      if (aPopular && !bPopular) return -1;
      if (!aPopular && bPopular) return 1;
      return a.localeCompare(b);
    });
  };

  return (
    <div class="card bg-gradient-to-br from-green-500/20 to-green-500/5 bg-base-100 shadow-lg h-full flex flex-col p-3">
      {/* Header */}
      <div class="flex items-center gap-1.5 mb-2">
        <IconCurrencyDollar size={16} class="text-green-500 opacity-80" />
        <span class="text-xs font-medium opacity-70">Currency Converter</span>
      </div>

      <div class="flex-1 flex flex-col">


        <Show when={loading() && Object.keys(rates()).length === 0}>
          <div class="flex items-center justify-center flex-1">
            <span class="loading loading-spinner loading-sm"></span>
          </div>
        </Show>

        <Show when={error()}>
          <div class="alert alert-error alert-sm">
            <span class="text-xs">{error()}</span>
          </div>
        </Show>

        <Show when={!loading() || Object.keys(rates()).length > 0}>
          {/* Amount Input */}
          <div class="form-control mb-3">
            <label class="label py-1">
              <span class="label-text text-xs">Amount</span>
            </label>
            <input
              type="number"
              class="input input-bordered input-sm w-full"
              value={amount()}
              onInput={handleAmountChange}
              placeholder="Enter amount"
            />
          </div>

          {/* From Currency */}
          <div class="form-control mb-3">
            <label class="label py-1">
              <span class="label-text text-xs">From</span>
            </label>
            <select
              class="select select-bordered select-sm w-full"
              value={fromCurrency()}
              onChange={handleFromChange}
            >
              {allCurrencies().map(currency => (
                <option value={currency}>
                  {currency} {popularCurrencies.includes(currency) ? '⭐' : ''}
                </option>
              ))}
            </select>
          </div>

          {/* Swap Button */}
          <button
            class="btn btn-sm btn-ghost btn-circle self-center mb-2"
            onClick={swapCurrencies}
          >
            ⇅
          </button>

          {/* To Currency */}
          <div class="form-control mb-3">
            <label class="label py-1">
              <span class="label-text text-xs">To</span>
            </label>
            <select
              class="select select-bordered select-sm w-full"
              value={toCurrency()}
              onChange={handleToChange}
            >
              {allCurrencies().map(currency => (
                <option value={currency}>
                  {currency} {popularCurrencies.includes(currency) ? '⭐' : ''}
                </option>
              ))}
            </select>
          </div>

          {/* Result */}
          <div class="alert alert-success mt-auto">
            <div class="flex flex-col w-full">
              <div class="text-lg font-bold font-mono">
                {amount()} {fromCurrency()} =
              </div>
              <div class="text-2xl font-bold font-mono text-success-content">
                {result()} {toCurrency()}
              </div>
              <Show when={lastUpdate()}>
                <div class="text-xs opacity-70 mt-1">
                  Updated: {lastUpdate()}
                </div>
              </Show>
            </div>
          </div>

          {/* Refresh Button */}
          <button
            class="btn btn-sm btn-outline btn-primary mt-2"
            onClick={fetchRates}
            disabled={loading()}
          >
            <Show when={loading()}>
              <span class="loading loading-spinner loading-xs"></span>
            </Show>
            Refresh Rates
          </button>
        </Show>
      </div>
    </div>
  );
}
