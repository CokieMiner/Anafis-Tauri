import { Box, Typography } from '@mui/material';
import type React from 'react';
import { memo, useMemo } from 'react';
import type { DataSequence } from '@/core/types/dataLibrary';
import Plot from '@/shared/components/PlotlyChart';
import {
  ANAFIS_CHART_CONFIG,
  ANAFIS_DARK_AXIS,
  ANAFIS_DARK_LAYOUT,
  CHART_COLORS,
} from '@/shared/components/plotlyTheme';

interface SequenceChartProps {
  sequence: DataSequence | null;
}

const SequenceChart: React.FC<SequenceChartProps> = memo(({ sequence }) => {
  const { data: traces, layout } = useMemo((): {
    data: Plotly.Data[];
    layout: Partial<Plotly.Layout>;
  } => {
    if (!sequence || sequence.data.length === 0) {
      return { data: [], layout: ANAFIS_DARK_LAYOUT };
    }

    const xData = Array.from({ length: sequence.data.length }, (_, i) => i);
    const yData = sequence.data;

    let uncertainties = sequence.uncertainties;
    if (!uncertainties || !Array.isArray(uncertainties)) {
      uncertainties = [];
    }
    if (uncertainties.length > 0 && uncertainties.length !== yData.length) {
      console.warn(`[SequenceChart] Uncertainties length mismatch, truncating`);
      uncertainties = uncertainties.slice(0, yData.length);
    }

    const trace: Plotly.Data = {
      x: xData,
      y: yData,
      mode: 'lines+markers',
      type: 'scatter',
      name: sequence.name,
      marker: { color: CHART_COLORS.line, size: 4 },
      line: { color: CHART_COLORS.line, width: 2 },
    };

    if (uncertainties.length > 0) {
      trace.error_y = {
        type: 'data',
        array: uncertainties,
        visible: true,
        color: CHART_COLORS.error,
        thickness: 1.5,
        width: 4,
      };
    }

    return {
      data: [trace],
      layout: {
        ...ANAFIS_DARK_LAYOUT,
        xaxis: {
          ...ANAFIS_DARK_AXIS,
          title: { text: 'Index', font: { color: '#aaa', size: 11 } },
        },
        yaxis: {
          ...ANAFIS_DARK_AXIS,
          title: {
            text: `${sequence.name} (${sequence.unit})`,
            font: { color: '#aaa', size: 11 },
          },
        },
        hovermode: 'closest',
      },
    };
  }, [sequence]);

  return (
    <Box sx={{ bgcolor: 'background.paper', p: 2, borderRadius: 1 }}>
      <Typography variant="h6" gutterBottom>
        Chart Preview
      </Typography>
      <Box
        sx={{
          width: '100%',
          height: 300,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
        }}
      >
        {!sequence ? (
          <Typography variant="h6" color="text.secondary">
            Select a sequence to view chart
          </Typography>
        ) : (
          <Plot
            data={traces}
            layout={layout}
            config={ANAFIS_CHART_CONFIG}
            useResizeHandler
            style={{ width: '100%', height: '100%' }}
          />
        )}
      </Box>
    </Box>
  );
});

SequenceChart.displayName = 'SequenceChart';

export default SequenceChart;
