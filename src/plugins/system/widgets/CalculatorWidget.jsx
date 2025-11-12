import { createSignal } from 'solid-js';
import { IconCalculator, IconBackspace } from '@tabler/icons-solidjs';

export default function CalculatorWidget() {
  const [display, setDisplay] = createSignal('0');
  const [previousValue, setPreviousValue] = createSignal(null);
  const [operation, setOperation] = createSignal(null);
  const [newNumber, setNewNumber] = createSignal(true);

  const handleNumber = (num) => {
    if (newNumber()) {
      setDisplay(num);
      setNewNumber(false);
    } else {
      setDisplay(display() === '0' ? num : display() + num);
    }
  };

  const handleDecimal = () => {
    if (newNumber()) {
      setDisplay('0.');
      setNewNumber(false);
    } else if (!display().includes('.')) {
      setDisplay(display() + '.');
    }
  };

  const handleOperation = (op) => {
    const current = parseFloat(display());

    if (previousValue() !== null && operation() && !newNumber()) {
      calculate();
    }

    setPreviousValue(current);
    setOperation(op);
    setNewNumber(true);
  };

  const calculate = () => {
    if (previousValue() === null || operation() === null) return;

    const prev = previousValue();
    const current = parseFloat(display());
    let result;

    switch (operation()) {
      case '+':
        result = prev + current;
        break;
      case '-':
        result = prev - current;
        break;
      case '×':
        result = prev * current;
        break;
      case '÷':
        result = current !== 0 ? prev / current : 'Error';
        break;
      default:
        return;
    }

    setDisplay(result === 'Error' ? result : String(result));
    setPreviousValue(null);
    setOperation(null);
    setNewNumber(true);
  };

  const clear = () => {
    setDisplay('0');
    setPreviousValue(null);
    setOperation(null);
    setNewNumber(true);
  };

  const backspace = () => {
    if (newNumber() || display().length === 1) {
      setDisplay('0');
      setNewNumber(true);
    } else {
      setDisplay(display().slice(0, -1));
    }
  };

  const Button = (props) => (
    <button
      class={`btn btn-sm ${props.class || 'btn-ghost'} text-xs h-10`}
      onClick={props.onClick}
    >
      {props.children}
    </button>
  );

  return (
    <div class="card bg-gradient-to-br from-primary/20 to-primary/5 bg-base-100 shadow-lg h-full flex flex-col p-3">
      {/* Header */}
      <div class="flex items-center gap-1.5 mb-2">
        <IconCalculator size={16} class="text-primary opacity-80" />
        <span class="text-xs font-medium opacity-70">Calculator</span>
      </div>

      {/* Display */}
      <div class="bg-base-200/70 rounded-lg p-2 mb-2">
        <div class="text-right text-xs opacity-50 h-3">
          {previousValue() !== null && operation() ? `${previousValue()} ${operation()}` : ''}
        </div>
        <div class="text-right text-lg font-bold break-all" title={display()}>
          {display().length > 15 ? display().slice(0, 12) + '...' : display()}
        </div>
      </div>

      {/* Buttons */}
      <div class="grid grid-cols-4 gap-1">
        {/* Row 1 */}
        <Button onClick={clear} class="btn-error text-error-content col-span-2">C</Button>
        <Button onClick={backspace} class="btn-ghost">
          <IconBackspace size={16} />
        </Button>
        <Button onClick={() => handleOperation('÷')} class="btn-primary">÷</Button>

        {/* Row 2 */}
        <Button onClick={() => handleNumber('7')}>7</Button>
        <Button onClick={() => handleNumber('8')}>8</Button>
        <Button onClick={() => handleNumber('9')}>9</Button>
        <Button onClick={() => handleOperation('×')} class="btn-primary">×</Button>

        {/* Row 3 */}
        <Button onClick={() => handleNumber('4')}>4</Button>
        <Button onClick={() => handleNumber('5')}>5</Button>
        <Button onClick={() => handleNumber('6')}>6</Button>
        <Button onClick={() => handleOperation('-')} class="btn-primary">-</Button>

        {/* Row 4 */}
        <Button onClick={() => handleNumber('1')}>1</Button>
        <Button onClick={() => handleNumber('2')}>2</Button>
        <Button onClick={() => handleNumber('3')}>3</Button>
        <Button onClick={() => handleOperation('+')} class="btn-primary">+</Button>

        {/* Row 5 */}
        <Button onClick={() => handleNumber('0')} class="col-span-2">0</Button>
        <Button onClick={handleDecimal}>.</Button>
        <Button onClick={calculate} class="btn-success text-success-content">=</Button>
      </div>
    </div>
  );
}
