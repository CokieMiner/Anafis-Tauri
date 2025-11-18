import React, { useEffect, useRef } from 'react';
import * as echarts from 'echarts';

interface HistogramChartProps {
  data: number[];
  title?: string;
  width?: number;
  height?: number;
  theme?: 'dark' | 'light';
  showOutliers?: boolean;
  outliers?: number[];
}

const HistogramChart: React.FC<HistogramChartProps> = ({
  data,
  title = 'Distribution',
  width = 388,
  height = 250,
  theme = 'dark',
  showOutliers = true,
  outliers = []
}) => {
  const chartRef = useRef<HTMLDivElement>(null);
  const chartInstanceRef = useRef<echarts.ECharts | null>(null);

  // Helper function to calculate histogram using Sturges' rule
  function calculateHistogram(data: number[]) {
    if (data.length === 0) {
      return { bins: [], binEdges: [], binWidth: 0 };
    }

    const min = Math.min(...data);
    const max = Math.max(...data);
    const range = max - min;

    // Sturges' rule for number of bins
    const numBins = Math.ceil(1 + 3.322 * Math.log10(data.length));
    const binWidth = range / numBins;

    const bins = new Array(numBins).fill(0) as number[];
    const binEdges: number[] = [];

    // Create bin edges
    for (let i = 0; i <= numBins; i++) {
      binEdges.push(min + i * binWidth);
    }

    // Count values in each bin
    data.forEach(value => {
      const binIndex = Math.min(Math.floor((value - min) / binWidth), numBins - 1);
      if (binIndex >= 0 && binIndex < numBins && bins[binIndex] !== undefined) {
        bins[binIndex]++;
      }
    });

    return { bins, binEdges, binWidth };
  }

  // Helper function to check if a bin contains outliers
  function isBinOutlier(binStart: number, binEnd: number, outliers: number[]): boolean {
    return outliers.some(outlier => outlier >= binStart && outlier < binEnd);
  }

  useEffect(() => {
    if (!chartRef.current || data.length === 0) {
      return;
    }

    // Initialize chart if not already done
    chartInstanceRef.current ??= echarts.init(chartRef.current, null, {
      renderer: 'canvas',
      devicePixelRatio: 2
    });

    // Resize chart to fit container
    chartInstanceRef.current.resize({
      width,
      height
    });

    // Calculate histogram data
    const { bins, binEdges, binWidth } = calculateHistogram(data);
    
    // Calculate data range for axis scaling
    const dataMin = Math.min(...data);
    const dataMax = Math.max(...data);
    const maxFrequency = Math.max(...bins);
    
    // Add some padding to the ranges
    const xPadding = (dataMax - dataMin) * 0.05; // 5% padding
    const yPadding = maxFrequency * 0.1; // 10% padding for frequency

    // Generate chart options
    const option: echarts.EChartsOption = {
      backgroundColor: theme === 'light' ? '#ffffff' : '#0a0a0a',
      title: {
        text: title,
        left: 'center',
        textStyle: {
          color: theme === 'light' ? '#000000' : '#ffffff',
          fontSize: 14
        }
      },
      grid: {
        left: '15%',
        right: '8%',
        top: '20%',
        bottom: '20%',
        containLabel: true,
      },
      xAxis: {
        type: 'value',
        name: 'Value',
        nameLocation: 'middle',
        nameGap: 25,
        nameTextStyle: {
          color: theme === 'light' ? '#000000' : '#ffffff',
          fontSize: 12
        },
        min: dataMin - xPadding,
        max: dataMax + xPadding,
        axisLine: {
          lineStyle: {
            color: theme === 'light' ? 'rgba(0,0,0,0.3)' : 'rgba(255,255,255,0.3)'
          }
        },
        axisLabel: {
          color: theme === 'light' ? '#000000' : '#ffffff',
          formatter: (value: number) => {
            if (Math.abs(value) >= 1e6 || (Math.abs(value) < 0.001 && value !== 0)) {
              return value.toExponential(2);
            } else if (Math.abs(value) >= 1000) {
              return value.toFixed(0);
            } else if (Math.abs(value) >= 1) {
              return value.toFixed(2);
            } else {
              return value.toFixed(4);
            }
          }
        },
        splitLine: {
          lineStyle: {
            color: theme === 'light' ? 'rgba(0,0,0,0.1)' : 'rgba(255,255,255,0.1)'
          }
        },
      },
      yAxis: {
        type: 'value',
        name: 'Frequency',
        nameLocation: 'middle',
        nameGap: 35,
        nameTextStyle: {
          color: theme === 'light' ? '#000000' : '#ffffff',
          fontSize: 12
        },
        min: 0,
        max: maxFrequency + yPadding,
        axisLine: {
          lineStyle: {
            color: theme === 'light' ? 'rgba(0,0,0,0.3)' : 'rgba(255,255,255,0.3)'
          }
        },
        axisLabel: {
          color: theme === 'light' ? '#000000' : '#ffffff'
        },
        splitLine: {
          lineStyle: {
            color: theme === 'light' ? 'rgba(0,0,0,0.1)' : 'rgba(255,255,255,0.1)'
          }
        },
      },
      series: [
        {
          name: 'Histogram',
          type: 'bar',
          data: bins.map((count, index) => ({
            value: [binEdges[index], count],
            itemStyle: {
              color: showOutliers && index < binEdges.length - 1 && isBinOutlier(binEdges[index]!, binEdges[index + 1]!, outliers)
                ? '#f44336' // Red for outlier bins
                : '#2196f3' // Blue for normal bins
            }
          })),
          barWidth: '90%',
          itemStyle: {
            borderColor: theme === 'light' ? 'rgba(0,0,0,0.2)' : 'rgba(255,255,255,0.2)',
            borderWidth: 1
          }
        }
      ],
      tooltip: {
        trigger: 'axis',
        axisPointer: { type: 'shadow' },
        backgroundColor: theme === 'light' ? 'rgba(255,255,255,0.95)' : 'rgba(0,0,0,0.8)',
        borderColor: '#2196f3',
        textStyle: {
          color: theme === 'light' ? '#000000' : '#ffffff'
        },
        formatter: (params: unknown) => {
          const param = params as Array<{ value: [number, number] }>;
          const data = param[0];
          if (!data) {
            return '';
          }
          const binStart = data.value[0];
          const binEnd = binStart + binWidth;
          const count = data.value[1];
          return `Range: ${binStart.toFixed(3)} - ${binEnd.toFixed(3)}<br/>Count: ${count}`;
        }
      }
    };

    chartInstanceRef.current.setOption(option, true);

    // Cleanup on unmount
    return () => {
      if (chartInstanceRef.current) {
        chartInstanceRef.current.dispose();
        chartInstanceRef.current = null;
      }
    };
  }, [data, title, theme, showOutliers, outliers, width, height]);

  return (
    <div
      ref={chartRef}
      style={{
        width: `${width}px`,
        height: `${height}px`,
        overflow: 'hidden',
        position: 'relative',
      }}
    />
  );
};

export default HistogramChart;