// Shared Plotly theme — consistent styling across all charts in AnaFis
// Import and spread these into your Plotly layout/trace configs.

/** Dark theme layout — transparent paper, dark grid */
export const ANAFIS_DARK_LAYOUT: Partial<Plotly.Layout> = {
  paper_bgcolor: 'transparent',
  plot_bgcolor: 'rgba(14, 14, 18, 0.6)',
  font: { color: '#aaa', family: 'Inter, sans-serif', size: 11 },
  margin: { l: 55, r: 16, t: 28, b: 44 },
  showlegend: false,
  hovermode: 'closest',
};

/** Light theme layout — for exports */
const ANAFIS_LIGHT_LAYOUT: Partial<Plotly.Layout> = {
  paper_bgcolor: '#ffffff',
  plot_bgcolor: '#ffffff',
  font: { color: '#222', family: 'Inter, sans-serif', size: 11 },
  margin: { l: 55, r: 16, t: 28, b: 44 },
  showlegend: false,
  hovermode: 'closest',
};

/** Standard dark axis styling */
export const ANAFIS_DARK_AXIS: Partial<Plotly.LayoutAxis> = {
  gridcolor: 'rgba(255,255,255,0.06)',
  zerolinecolor: 'rgba(255,255,255,0.12)',
  linecolor: '#444',
  tickfont: { color: '#888', size: 10, family: 'Inter, sans-serif' },
};

/** Standard light axis styling (exports) */
const ANAFIS_LIGHT_AXIS: Partial<Plotly.LayoutAxis> = {
  gridcolor: 'rgba(0,0,0,0.08)',
  zerolinecolor: 'rgba(0,0,0,0.15)',
  linecolor: 'rgba(0,0,0,0.3)',
  tickfont: { color: '#444', size: 10, family: 'Inter, sans-serif' },
};

/** Get layout + axes by theme */
export function getThemeLayout(theme: 'dark' | 'light') {
  return {
    layout: theme === 'dark' ? ANAFIS_DARK_LAYOUT : ANAFIS_LIGHT_LAYOUT,
    axis: theme === 'dark' ? ANAFIS_DARK_AXIS : ANAFIS_LIGHT_AXIS,
  };
}

/** Standard Plotly config for embedded charts (no mode bar, responsive) */
export const ANAFIS_CHART_CONFIG: Partial<Plotly.Config> = {
  responsive: true,
  displayModeBar: false,
};

/** Color palette */
export const CHART_COLORS = {
  primary: '#64b5f6', // data scatter
  fit: '#ffb300', // fit curves
  error: '#f44336', // error bars / outliers
  line: '#90caf9', // line series
  secondary: '#4caf50', // trend overlays
  residual: '#ef5350', // residuals
} as const;
