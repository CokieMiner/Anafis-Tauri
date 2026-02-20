// Shared Plotly instance — creates Plot component from the minimal dist bundle

// @ts-expect-error — plotly.js-dist-min has no types; the full types come from @types/react-plotly.js
import Plotly from 'plotly.js-dist-min';
import createPlotlyComponent from 'react-plotly.js/factory';

const Plot = createPlotlyComponent(Plotly);

export default Plot;
export { Plotly };
