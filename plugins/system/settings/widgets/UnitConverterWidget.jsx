import { createSignal } from 'solid-js';
import { IconRuler2 } from '@tabler/icons-solidjs';

export default function UnitConverterWidget() {
  const [category, setCategory] = createSignal('length');
  const [fromUnit, setFromUnit] = createSignal('meters');
  const [toUnit, setToUnit] = createSignal('feet');
  const [inputValue, setInputValue] = createSignal('1');
  const [result, setResult] = createSignal('3.28084');

  const conversions = {
    length: {
      meters: 1,
      kilometers: 0.001,
      centimeters: 100,
      millimeters: 1000,
      miles: 0.000621371,
      yards: 1.09361,
      feet: 3.28084,
      inches: 39.3701
    },
    weight: {
      kilograms: 1,
      grams: 1000,
      milligrams: 1000000,
      pounds: 2.20462,
      ounces: 35.274
    },
    temperature: {
      celsius: { toCelsius: (v) => v, fromCelsius: (v) => v },
      fahrenheit: { toCelsius: (v) => (v - 32) * 5/9, fromCelsius: (v) => (v * 9/5) + 32 },
      kelvin: { toCelsius: (v) => v - 273.15, fromCelsius: (v) => v + 273.15 }
    },
    volume: {
      liters: 1,
      milliliters: 1000,
      gallons: 0.264172,
      quarts: 1.05669,
      pints: 2.11338,
      cups: 4.22675,
      fluid_ounces: 33.814
    },
    area: {
      square_meters: 1,
      square_kilometers: 0.000001,
      square_feet: 10.7639,
      square_yards: 1.19599,
      acres: 0.000247105,
      hectares: 0.0001
    }
  };

  const convert = () => {
    const value = parseFloat(inputValue());
    if (isNaN(value)) {
      setResult('Invalid input');
      return;
    }

    const currentCategory = conversions[category()];

    if (category() === 'temperature') {
      // Special handling for temperature
      const celsius = currentCategory[fromUnit()].toCelsius(value);
      const converted = currentCategory[toUnit()].fromCelsius(celsius);
      setResult(converted.toFixed(4));
    } else {
      // Standard ratio-based conversion
      const baseValue = value / currentCategory[fromUnit()];
      const converted = baseValue * currentCategory[toUnit()];
      setResult(converted.toFixed(4));
    }
  };

  const handleCategoryChange = (newCategory) => {
    setCategory(newCategory);
    const units = Object.keys(conversions[newCategory]);
    setFromUnit(units[0]);
    setToUnit(units[1] || units[0]);
    convert();
  };

  const handleInput = (e) => {
    setInputValue(e.target.value);
    convert();
  };

  const handleFromUnit = (e) => {
    setFromUnit(e.target.value);
    convert();
  };

  const handleToUnit = (e) => {
    setToUnit(e.target.value);
    convert();
  };

  return (
    <div class="card bg-gradient-to-br from-purple-500/20 to-purple-500/5 bg-base-100 shadow-lg h-full flex flex-col p-3">
      {/* Header */}
      <div class="flex items-center gap-1.5 mb-2">
        <IconRuler2 size={16} class="text-purple-500 opacity-80" />
        <span class="text-xs font-medium opacity-70">Unit Converter</span>
      </div>

      <div class="flex-1 flex flex-col">


        {/* Category Selection */}
        <div class="form-control mb-3">
          <label class="label py-1">
            <span class="label-text text-xs">Category</span>
          </label>
          <select
            class="select select-bordered select-sm w-full"
            value={category()}
            onChange={(e) => handleCategoryChange(e.target.value)}
          >
            <option value="length">Length</option>
            <option value="weight">Weight</option>
            <option value="temperature">Temperature</option>
            <option value="volume">Volume</option>
            <option value="area">Area</option>
          </select>
        </div>

        {/* From Unit */}
        <div class="form-control mb-3">
          <label class="label py-1">
            <span class="label-text text-xs">From</span>
          </label>
          <div class="flex gap-2">
            <input
              type="number"
              class="input input-bordered input-sm flex-1"
              value={inputValue()}
              onInput={handleInput}
              placeholder="Enter value"
            />
            <select
              class="select select-bordered select-sm w-32"
              value={fromUnit()}
              onChange={handleFromUnit}
            >
              {Object.keys(conversions[category()]).map(unit => (
                <option value={unit}>{unit.replace('_', ' ')}</option>
              ))}
            </select>
          </div>
        </div>

        {/* To Unit */}
        <div class="form-control mb-3">
          <label class="label py-1">
            <span class="label-text text-xs">To</span>
          </label>
          <div class="flex gap-2">
            <input
              type="text"
              class="input input-bordered input-sm flex-1 font-mono"
              value={result()}
              readOnly
            />
            <select
              class="select select-bordered select-sm w-32"
              value={toUnit()}
              onChange={handleToUnit}
            >
              {Object.keys(conversions[category()]).map(unit => (
                <option value={unit}>{unit.replace('_', ' ')}</option>
              ))}
            </select>
          </div>
        </div>

        {/* Quick swap button */}
        <button
          class="btn btn-sm btn-outline btn-purple-500 mt-auto"
          onClick={() => {
            const temp = fromUnit();
            setFromUnit(toUnit());
            setToUnit(temp);
            convert();
          }}
        >
          ⇄ Swap Units
        </button>
      </div>
    </div>
  );
}
