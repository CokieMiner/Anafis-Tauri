// Shared Plotly instance — creates Plot component from the minimal dist bundle

// @ts-expect-error — plotly.js-dist-min has no types; the full types come from @types/react-plotly.js
import Plotly from 'plotly.js-dist-min';
import _createPlotlyComponent from 'react-plotly.js/factory';

// Vite 8 (ESM) wraps CJS default exports in { default: fn }; unwrap if needed
const _mod = _createPlotlyComponent as unknown as Record<string, unknown>;
const createPlotlyComponent: typeof _createPlotlyComponent =
  (_mod.default as typeof _createPlotlyComponent) ?? _createPlotlyComponent;

const Plot = createPlotlyComponent(Plotly);

export default Plot;
export { Plotly };
