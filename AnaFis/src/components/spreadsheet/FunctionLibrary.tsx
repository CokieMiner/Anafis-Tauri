import React, { useState } from 'react';
import {
  Box,
  Typography,
  IconButton,
  Paper,
  List,
  ListItemText,
  Chip,
  TextField,
  InputAdornment,
  Button,
  Divider,
  ListItemButton
} from '@mui/material';
import { CloseIcon, SearchIcon } from '../../icons';

interface FunctionLibraryProps {
  onClose: () => void;
  onFunctionSelect: (func: string) => void;
}

interface FunctionCategory {
  name: string;
  icon: string;
  functions: FunctionInfo[];
}

interface FunctionInfo {
  name: string;
  description: string;
  syntax: string;
  example: string;
}

const functionCategories: FunctionCategory[] = [
  {
    name: 'Statistical',
    icon: '📊',
    functions: [
      {
        name: 'SUM',
        description: 'Sum of values in a range',
        syntax: 'SUM(range)',
        example: 'SUM(A1:A10)',
      },
      {
        name: 'AVERAGE',
        description: 'Average of values in a range',
        syntax: 'AVERAGE(range)',
        example: 'AVERAGE(B1:B20)',
      },
      {
        name: 'COUNT',
        description: 'Count of numeric values',
        syntax: 'COUNT(range)',
        example: 'COUNT(F1:F20)',
      },
      {
        name: 'MIN',
        description: 'Minimum value in a range',
        syntax: 'MIN(range)',
        example: 'MIN(D1:D10)',
      },
      {
        name: 'MAX',
        description: 'Maximum value in a range',
        syntax: 'MAX(range)',
        example: 'MAX(E1:E10)',
      },
      {
        name: 'STDEV',
        description: 'Standard deviation of values',
        syntax: 'STDEV(range)',
        example: 'STDEV(C1:C15)',
      },
      {
        name: 'NORMDIST',
        description: 'Normal distribution PDF/CDF',
        syntax: 'NORMDIST(x, mean, std_dev, cumulative)',
        example: 'NORMDIST(A1, 0, 1, TRUE)',
      },
    ],
  },
  {
    name: 'Mathematical',
    icon: '🧮',
    functions: [
      {
        name: 'SIN',
        description: 'Sine function',
        syntax: 'SIN(angle)',
        example: 'SIN(A1)',
      },
      {
        name: 'COS',
        description: 'Cosine function',
        syntax: 'COS(angle)',
        example: 'COS(B1)',
      },
      {
        name: 'TAN',
        description: 'Tangent function',
        syntax: 'TAN(angle)',
        example: 'TAN(C1)',
      },
      {
        name: 'ASIN',
        description: 'Arcsine function',
        syntax: 'ASIN(value)',
        example: 'ASIN(D1)',
      },
      {
        name: 'ACOS',
        description: 'Arccosine function',
        syntax: 'ACOS(value)',
        example: 'ACOS(E1)',
      },
      {
        name: 'ATAN',
        description: 'Arctangent function',
        syntax: 'ATAN(value)',
        example: 'ATAN(F1)',
      },
      {
        name: 'ATAN2',
        description: 'Two-argument arctangent',
        syntax: 'ATAN2(y, x)',
        example: 'ATAN2(A1, B1)',
      },
      {
        name: 'SINH',
        description: 'Hyperbolic sine',
        syntax: 'SINH(value)',
        example: 'SINH(A1)',
      },
      {
        name: 'COSH',
        description: 'Hyperbolic cosine',
        syntax: 'COSH(value)',
        example: 'COSH(B1)',
      },
      {
        name: 'TANH',
        description: 'Hyperbolic tangent',
        syntax: 'TANH(value)',
        example: 'TANH(C1)',
      },
      {
        name: 'LN',
        description: 'Natural logarithm',
        syntax: 'LN(value)',
        example: 'LN(D1)',
      },
      {
        name: 'LOG10',
        description: 'Base-10 logarithm',
        syntax: 'LOG10(value)',
        example: 'LOG10(E1)',
      },
      {
        name: 'LOG',
        description: 'Logarithm with custom base',
        syntax: 'LOG(value, base)',
        example: 'LOG(F1, 2)',
      },
      {
        name: 'EXP',
        description: 'Exponential function',
        syntax: 'EXP(value)',
        example: 'EXP(F1)',
      },
      {
        name: 'SQRT',
        description: 'Square root',
        syntax: 'SQRT(value)',
        example: 'SQRT(G1)',
      },
      {
        name: 'POW',
        description: 'Power function',
        syntax: 'POW(base, exponent)',
        example: 'POW(H1, 2)',
      },
      {
        name: 'ABS',
        description: 'Absolute value',
        syntax: 'ABS(value)',
        example: 'ABS(I1)',
      },
      {
        name: 'ROUND',
        description: 'Round to nearest integer',
        syntax: 'ROUND(value)',
        example: 'ROUND(J1)',
      },
      {
        name: 'FLOOR',
        description: 'Round down to nearest integer',
        syntax: 'FLOOR(value)',
        example: 'FLOOR(K1)',
      },
      {
        name: 'CEIL',
        description: 'Round up to nearest integer',
        syntax: 'CEIL(value)',
        example: 'CEIL(L1)',
      },
    ],
  },
  {
    name: 'Advanced',
    icon: '🔬',
    functions: [
      {
        name: 'GAMMA',
        description: 'Gamma function',
        syntax: 'GAMMA(value)',
        example: 'GAMMA(A1)',
      },
      {
        name: 'DIGAMMA',
        description: 'Digamma function',
        syntax: 'DIGAMMA(value)',
        example: 'DIGAMMA(B1)',
      },
      {
        name: 'LNGAMMA',
        description: 'Natural log of gamma function',
        syntax: 'LNGAMMA(value)',
        example: 'LNGAMMA(C1)',
      },
      {
        name: 'BETA',
        description: 'Beta function',
        syntax: 'BETA(a, b)',
        example: 'BETA(A1, B1)',
      },
      {
        name: 'ERF',
        description: 'Error function',
        syntax: 'ERF(value)',
        example: 'ERF(A1)',
      },
      {
        name: 'ERFI',
        description: 'Imaginary error function',
        syntax: 'ERFI(value)',
        example: 'ERFI(B1)',
      },
      {
        name: 'ZETA',
        description: 'Riemann zeta function approximation',
        syntax: 'ZETA(value)',
        example: 'ZETA(C1)',
      },
    ],
  },
  {
    name: 'Smooth Functions',
    icon: '📈',
    functions: [
      {
        name: 'SMOOTHABS',
        description: 'Smooth absolute value',
        syntax: 'SMOOTHABS(value)',
        example: 'SMOOTHABS(A1)',
      },
      {
        name: 'SMOOTHSTEP',
        description: 'Smooth step function',
        syntax: 'SMOOTHSTEP(value)',
        example: 'SMOOTHSTEP(B1)',
      },
      {
        name: 'SMOOTHFLOOR',
        description: 'Smooth floor function',
        syntax: 'SMOOTHFLOOR(value)',
        example: 'SMOOTHFLOOR(C1)',
      },
      {
        name: 'SMOOTHCEIL',
        description: 'Smooth ceiling function',
        syntax: 'SMOOTHCEIL(value)',
        example: 'SMOOTHCEIL(D1)',
      },
      {
        name: 'SMOOTHROUND',
        description: 'Smooth rounding function',
        syntax: 'SMOOTHROUND(value, decimals)',
        example: 'SMOOTHROUND(E1, 2)',
      },
      {
        name: 'SMOOTHTRUNC',
        description: 'Smooth truncation function',
        syntax: 'SMOOTHTRUNC(value)',
        example: 'SMOOTHTRUNC(F1)',
      },
      {
        name: 'SMOOTHFRACT',
        description: 'Smooth fractional part',
        syntax: 'SMOOTHFRACT(value)',
        example: 'SMOOTHFRACT(G1)',
      },
    ],
  },
];

export const FunctionLibrary: React.FC<FunctionLibraryProps> = ({
  onClose,
  onFunctionSelect,
}) => {
  const [selectedCategory, setSelectedCategory] = useState<string>('Mathematical');
  const [selectedFunction, setSelectedFunction] = useState<FunctionInfo | null>(null);
  const [searchTerm, setSearchTerm] = useState<string>('');

  const currentCategory = functionCategories.find(cat => cat.name === selectedCategory);

  // Filter functions based on search term
  const filteredFunctions = currentCategory?.functions.filter(func =>
    func.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    func.description.toLowerCase().includes(searchTerm.toLowerCase())
  ) || [];

  return (
    <Paper
      elevation={0}
      sx={{
        width: 360,
        height: '100%',
        display: 'flex',
        flexDirection: 'column',
        bgcolor: 'rgba(10, 10, 10, 0.95)',
        border: '1px solid rgba(255, 255, 255, 0.08)',
        borderRadius: '12px',
        overflow: 'hidden',
      }}
    >
      {/* Header */}
      <Box
        sx={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          p: 2,
          bgcolor: 'rgba(255, 255, 255, 0.02)',
          borderBottom: '1px solid rgba(255, 255, 255, 0.08)',
        }}
      >
        <Typography variant="h6" sx={{ fontWeight: 600, color: 'rgba(255, 255, 255, 0.9)' }}>
          Function Library
        </Typography>
        <IconButton
          onClick={onClose}
          size="small"
          sx={{
            color: 'rgba(255, 255, 255, 0.7)',
            borderRadius: '6px',
            '&:hover': {
              bgcolor: 'rgba(255, 255, 255, 0.1)',
              color: 'rgba(255, 255, 255, 0.9)',
            },
          }}
        >
          <CloseIcon />
        </IconButton>
      </Box>

      {/* Search Field */}
      <Box sx={{ p: 2, borderBottom: '1px solid rgba(255, 255, 255, 0.08)' }}>
        <TextField
          fullWidth
          size="small"
          placeholder="Search functions..."
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
          sx={{
            '& .MuiOutlinedInput-root': {
              bgcolor: 'rgba(255, 255, 255, 0.02)',
              borderRadius: '8px',
              '& fieldset': {
                borderColor: 'rgba(255, 255, 255, 0.08)',
              },
              '&:hover fieldset': {
                borderColor: 'rgba(255, 255, 255, 0.2)',
              },
              '&.Mui-focused fieldset': {
                borderColor: '#2196f3',
              },
            },
            '& .MuiInputBase-input': {
              color: 'rgba(255, 255, 255, 0.9)',
            },
            '& .MuiInputBase-input::placeholder': {
              color: 'rgba(255, 255, 255, 0.5)',
            },
          }}
          InputProps={{
            startAdornment: (
              <InputAdornment position="start">
                <SearchIcon sx={{ color: 'rgba(255, 255, 255, 0.5)', fontSize: 20 }} />
              </InputAdornment>
            ),
          }}
        />
      </Box>

      {/* Content */}
      <Box sx={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
        {/* Categories Sidebar */}
        <Box
          sx={{
            width: 110,
            borderRight: '1px solid rgba(255, 255, 255, 0.08)',
            bgcolor: 'rgba(255, 255, 255, 0.01)',
            height: '100%',
            overflow: 'auto',
          }}
        >
          <List dense sx={{ p: 0.5, height: '100%' }}>
            {functionCategories.map((category) => (
              <ListItemButton
                key={category.name}
                selected={selectedCategory === category.name}
                onClick={() => {
                  setSelectedCategory(category.name);
                  setSelectedFunction(null);
                }}
                sx={{
                  px: 1,
                  py: 0.5,
                  mb: 0.5,
                  borderRadius: '6px',
                  border: '1px solid rgba(255, 255, 255, 0.08)',
                  bgcolor: selectedCategory === category.name
                    ? 'rgba(33, 150, 243, 0.15)'
                    : 'rgba(255, 255, 255, 0.02)',
                  color: selectedCategory === category.name
                    ? '#2196f3'
                    : 'rgba(255, 255, 255, 0.7)',
                  transition: 'all 0.2s ease-in-out',
                  '&:hover': {
                    bgcolor: selectedCategory === category.name
                      ? 'rgba(33, 150, 243, 0.2)'
                      : 'rgba(255, 255, 255, 0.08)',
                    borderColor: '#2196f3',
                    color: '#ffffff',
                    transform: 'translateY(-1px)',
                    boxShadow: '0 2px 8px rgba(33, 150, 243, 0.3)',
                  },
                }}
              >
                <ListItemText
                  primary={
                    <Box sx={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 0.5 }}>
                      <Typography component="span" sx={{ fontSize: 18 }}>
                        {category.icon}
                      </Typography>
                      <Typography
                        variant="body2"
                        sx={{
                          fontSize: 10,
                          fontWeight: 500,
                          textAlign: 'center',
                          lineHeight: 1.2,
                        }}
                      >
                        {category.name}
                      </Typography>
                    </Box>
                  }
                />
              </ListItemButton>
            ))}
          </List>
        </Box>

        {/* Functions List */}
        <Box sx={{ flex: 1, p: 1.5, overflow: 'hidden', display: 'flex', flexDirection: 'column', height: '100%' }}>
          <Typography variant="subtitle2" sx={{ mb: 1, fontWeight: 600, color: 'rgba(255, 255, 255, 0.9)' }}>
            <Box component="span" sx={{ mr: 1 }}>{currentCategory?.icon}</Box>
            {selectedCategory}
          </Typography>

          <Box sx={{ flex: 1, overflow: 'auto', mb: selectedFunction ? 1.5 : 0, height: '100%' }}>
            <List dense sx={{ height: '100%', overflow: 'auto' }}>
                {filteredFunctions.map((func) => (
                  <ListItemButton
                    key={func.name}
                    selected={selectedFunction?.name === func.name}
                    onClick={() => setSelectedFunction(func)}
                    sx={{
                      mb: 0.5,
                      borderRadius: '6px',
                      border: '1px solid rgba(255, 255, 255, 0.08)',
                      bgcolor: selectedFunction?.name === func.name
                        ? 'rgba(33, 150, 243, 0.15)'
                        : 'rgba(255, 255, 255, 0.02)',
                      transition: 'all 0.2s ease-in-out',
                      '&:hover': {
                        bgcolor: 'rgba(33, 150, 243, 0.1)',
                        borderColor: '#2196f3',
                        transform: 'translateY(-1px)',
                        boxShadow: '0 2px 8px rgba(33, 150, 243, 0.2)',
                      },
                    }}
                  >
                    <ListItemText
                      primary={
                        <Typography variant="body2" sx={{
                          fontWeight: 600,
                          fontSize: 13,
                          color: selectedFunction?.name === func.name
                            ? '#2196f3'
                            : 'rgba(255, 255, 255, 0.9)'
                        }}>
                          {func.name}
                        </Typography>
                      }
                      secondary={
                        <Typography
                          variant="caption"
                          sx={{
                            color: 'rgba(255, 255, 255, 0.6)',
                            overflow: 'hidden',
                            textOverflow: 'ellipsis',
                            whiteSpace: 'nowrap',
                            fontSize: 11,
                          }}
                        >
                          {func.description}
                        </Typography>
                      }
                    />
                  </ListItemButton>
                ))}
              </List>
            </Box>

          {/* Function Details */}
          {selectedFunction && (
            <>
              <Divider sx={{ my: 1.5, borderColor: 'rgba(255, 255, 255, 0.1)' }} />
              <Box>
                <Typography variant="subtitle2" sx={{ mb: 1.5, fontWeight: 600, color: '#2196f3' }}>
                  {selectedFunction.name}
                </Typography>

                <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
                  <Box>
                    <Typography variant="body2" sx={{ fontWeight: 600, mb: 0.5, color: 'rgba(255, 255, 255, 0.8)', fontSize: 12 }}>
                      Syntax:
                    </Typography>
                    <Chip
                      label={selectedFunction.syntax}
                      variant="outlined"
                      size="small"
                      sx={{
                        fontFamily: 'monospace',
                        fontSize: 10,
                        bgcolor: 'rgba(255, 255, 255, 0.05)',
                        borderColor: 'rgba(255, 255, 255, 0.2)',
                        color: 'rgba(255, 255, 255, 0.9)',
                      }}
                    />
                  </Box>

                  <Box>
                    <Typography variant="body2" sx={{ fontWeight: 600, mb: 0.5, color: 'rgba(255, 255, 255, 0.8)', fontSize: 12 }}>
                      Example:
                    </Typography>
                    <Chip
                      label={selectedFunction.example}
                      variant="outlined"
                      size="small"
                      sx={{
                        fontFamily: 'monospace',
                        fontSize: 10,
                        bgcolor: 'rgba(255, 255, 255, 0.05)',
                        borderColor: 'rgba(255, 255, 255, 0.2)',
                        color: 'rgba(255, 255, 255, 0.9)',
                      }}
                    />
                  </Box>

                  <Box>
                    <Typography variant="body2" sx={{ fontWeight: 600, mb: 0.5, color: 'rgba(255, 255, 255, 0.8)', fontSize: 12 }}>
                      Description:
                    </Typography>
                    <Typography variant="body2" sx={{ color: 'rgba(255, 255, 255, 0.7)', fontSize: 11 }}>
                      {selectedFunction.description}
                    </Typography>
                  </Box>
                </Box>

                <Button
                  variant="outlined"
                  size="small"
                  onClick={() => onFunctionSelect(selectedFunction.name)}
                  sx={{
                    mt: 1.5,
                    fontWeight: 600,
                    fontSize: 12,
                    borderRadius: '6px',
                    border: '1px solid rgba(255, 255, 255, 0.08)',
                    background: 'rgba(255, 255, 255, 0.02)',
                    color: 'rgba(255, 255, 255, 0.7)',
                    transition: 'all 0.2s ease-in-out',
                    '&:hover': {
                      background: 'rgba(33, 150, 243, 0.1)',
                      borderColor: '#2196f3',
                      color: '#ffffff',
                      transform: 'translateY(-1px)',
                      boxShadow: '0 2px 8px rgba(33, 150, 243, 0.3)',
                    },
                  }}
                  fullWidth
                >
                  Insert Function
                </Button>
              </Box>
            </>
          )}
        </Box>
      </Box>
    </Paper>
  );
};
