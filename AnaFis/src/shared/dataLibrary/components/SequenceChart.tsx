import React, { useEffect, useRef, memo } from 'react';
import { Box, Typography } from '@mui/material';
import * as echarts from 'echarts';
import type { DataSequence } from '@/core/types/dataLibrary';

interface SequenceChartProps {
  sequence: DataSequence | null;
}

const SequenceChart: React.FC<SequenceChartProps> = memo(({ sequence }) => {
  const chartRef = useRef<HTMLDivElement>(null);
  const chartInstanceRef = useRef<echarts.ECharts | null>(null);

  // Initialize chart instance once
  useEffect(() => {
    if (!chartRef.current || chartInstanceRef.current) { return; }

    // Create chart instance
    chartInstanceRef.current = echarts.init(chartRef.current, null, {
      renderer: 'canvas',
      devicePixelRatio: 2,
    });

    // Set up responsive resize handling
    let resizeTimeout: NodeJS.Timeout | null = null;
    const debouncedResize = () => {
      if (resizeTimeout) {
        clearTimeout(resizeTimeout);
      }
      resizeTimeout = setTimeout(() => {
        if (chartInstanceRef.current) {
          chartInstanceRef.current.resize();
        }
      }, 100); // 100ms debounce
    };

    // Use ResizeObserver for container size changes (preferred over window resize)
    let resizeObserver: ResizeObserver | null = null;
    if (typeof ResizeObserver !== 'undefined') {
      resizeObserver = new ResizeObserver(debouncedResize);
      resizeObserver.observe(chartRef.current);
    } else {
      // Fallback to window resize event if ResizeObserver not available
      window.addEventListener('resize', debouncedResize);
    }

    return () => {
      // Cleanup resize handling
      if (resizeTimeout) {
        clearTimeout(resizeTimeout);
      }
      if (resizeObserver) {
        resizeObserver.disconnect();
      } else {
        window.removeEventListener('resize', debouncedResize);
      }

      // Cleanup chart instance
      if (chartInstanceRef.current) {
        chartInstanceRef.current.dispose();
        chartInstanceRef.current = null;
      }
    };
  }, []); // Empty dependency array - only run once on mount

  // Update chart when sequence changes
  useEffect(() => {
    if (!chartInstanceRef.current) { return; }

    // Always clear chart first
    chartInstanceRef.current.clear();

    // Exit early if no sequence is provided
    if (!sequence) {
      return;
    }

    // Set new data if sequence exists
    const currentSequence = sequence;

    // Create local copy of uncertainties to avoid modifying props
    let uncertainties = currentSequence.uncertainties;
    if (!uncertainties || !Array.isArray(uncertainties)) {
      console.warn('[SequenceChart] Sequence uncertainties is missing or not an array, using empty array:', currentSequence);
      uncertainties = [];
    }

    // Validate data length alignment
    const dataLength = currentSequence.data.length;
    const uncertaintiesLength = uncertainties.length;

    if (dataLength === 0) {
      console.warn('[SequenceChart] Sequence data is empty');
      return;
    }

    if (uncertaintiesLength > 0 && uncertaintiesLength !== dataLength) {
      console.warn(`[SequenceChart] Uncertainties length (${uncertaintiesLength}) doesn't match data length (${dataLength}), truncating uncertainties`);
      uncertainties = uncertainties.slice(0, dataLength);
    }

    const xData = Array.from({ length: dataLength }, (_, i) => i);
    const yData = currentSequence.data;
    const errorData = uncertainties;
    const series: echarts.SeriesOption[] = [
      {
        name: currentSequence.name,
        type: 'line',
        data: xData.map((x, i) => [x, yData[i]]),
        showSymbol: true,
        symbolSize: 4,
        itemStyle: { color: '#90caf9' },
        lineStyle: { color: '#90caf9', width: 2 },
      },
    ];

      // Add error bars if uncertainties exist
      if (errorData.length > 0) {
        series.push({
          name: 'Uncertainties',
          type: 'custom',
          renderItem: (_params: echarts.CustomSeriesRenderItemParams, api: echarts.CustomSeriesRenderItemAPI) => {
            const point = api.coord([api.value(0), api.value(1)]);
            const errorValue = api.value(2) as number;
            const yTop = api.coord([api.value(0), (api.value(1) as number) + errorValue]);
            const yBottom = api.coord([api.value(0), (api.value(1) as number) - errorValue]);

          // Ensure all coordinates are valid numbers
          if (typeof point[0] !== 'number' || typeof point[1] !== 'number' ||
              typeof yTop[1] !== 'number' || typeof yBottom[1] !== 'number') {
            return null;
          }

          return {
            type: 'group',
            children: [
              {
                type: 'line',
                shape: {
                  x1: point[0],
                  y1: yTop[1],
                  x2: point[0],
                  y2: yBottom[1]
                },
                style: { stroke: '#f44336', lineWidth: 1.5 },
              },
              {
                type: 'line',
                shape: {
                  x1: point[0] - 4,
                  y1: yTop[1],
                  x2: point[0] + 4,
                  y2: yTop[1]
                },
                style: { stroke: '#f44336', lineWidth: 1.5 },
              },
              {
                type: 'line',
                shape: {
                  x1: point[0] - 4,
                  y1: yBottom[1],
                  x2: point[0] + 4,
                  y2: yBottom[1]
                },
                style: { stroke: '#f44336', lineWidth: 1.5 },
              },
            ],
          };
        },
        data: xData.map((x, i) => [x, yData[i], errorData[i]]),
        z: 1,
        silent: true,
      });
    }

    const option: echarts.EChartsOption = {
      backgroundColor: 'transparent',
      grid: {
        left: 60,
        right: 20,
        top: 20,
        bottom: 40,
        containLabel: false,
      },
      xAxis: {
        type: 'value',
        name: 'Index',
        nameLocation: 'middle',
        nameGap: 30,
        nameTextStyle: { color: '#ffffff', fontSize: 11 },
        axisLine: { lineStyle: { color: 'rgba(255,255,255,0.3)' } },
        axisLabel: { color: '#ffffff', fontSize: 10 },
        splitLine: { lineStyle: { color: 'rgba(255,255,255,0.1)' } },
      },
      yAxis: {
        type: 'value',
        name: `${currentSequence.name} (${currentSequence.unit})`,
        nameLocation: 'middle',
        nameGap: 45,
        nameTextStyle: { color: '#ffffff', fontSize: 11 },
        axisLine: { lineStyle: { color: 'rgba(255,255,255,0.3)' } },
        axisLabel: { color: '#ffffff', fontSize: 10 },
        splitLine: { lineStyle: { color: 'rgba(255,255,255,0.1)' } },
      },
      series,
      tooltip: {
        trigger: 'axis',
        axisPointer: { type: 'cross' },
        backgroundColor: 'rgba(0,0,0,0.8)',
        borderColor: '#90caf9',
        textStyle: { color: '#ffffff' },
      },
    };

    chartInstanceRef.current.setOption(option, true);
  }, [sequence]);

  return (
    <Box sx={{ bgcolor: 'background.paper', p: 2, borderRadius: 1 }}>
      <Typography variant="h6" gutterBottom>Chart Preview</Typography>
      <Box
        ref={chartRef}
        sx={{
          width: '100%',
          height: 300,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
        }}
      >
        {!sequence && (
          <Typography variant="h6" color="text.secondary">
            Select a sequence to view chart
          </Typography>
        )}
      </Box>
    </Box>
  );
});

SequenceChart.displayName = 'SequenceChart';

export default SequenceChart;