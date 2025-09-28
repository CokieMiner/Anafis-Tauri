import React, { useState } from 'react';
import { Box, Button, Typography, TextField } from '@mui/material';
import { invoke } from '@tauri-apps/api/core';
import UncertaintyCell from './UncertaintyCell';

const UncertaintyTest: React.FC = () => {
  const [testCellRef] = useState('A1');
  const [testValue, setTestValue] = useState('5.2 ± 0.1');
  const [isEditing, setIsEditing] = useState(false);
  const [result, setResult] = useState<string>('');

  const handleTestDetection = async () => {
    try {
      const hasUncertainty = await invoke<boolean>('detect_uncertainty_mode', {
        input: testValue
      });
      setResult(`Detection result: ${hasUncertainty ? 'Has uncertainty' : 'No uncertainty'}`);
    } catch (error) {
      setResult(`Error: ${error}`);
    }
  };

  const handleTestToggle = async () => {
    try {
      await invoke('toggle_uncertainty_cell_mode', {
        cellRef: testCellRef,
        enable: true
      });
      setResult('Successfully enabled uncertainty mode');
    } catch (error) {
      setResult(`Error: ${error}`);
    }
  };

  const handleTestSetValue = async () => {
    try {
      await invoke('set_uncertainty_cell_value', {
        cellRef: testCellRef,
        value: 5.2,
        uncertainty: 0.1,
        uncertaintyType: 'absolute'
      });
      setResult('Successfully set uncertainty value');
    } catch (error) {
      setResult(`Error: ${error}`);
    }
  };

  const handleTestGetComponents = async () => {
    try {
      const components = await invoke('get_uncertainty_cell_components', {
        cellRef: testCellRef
      });
      setResult(`Components: ${JSON.stringify(components, null, 2)}`);
    } catch (error) {
      setResult(`Error: ${error}`);
    }
  };

  return (
    <Box sx={{ p: 2, maxWidth: 600 }}>
      <Typography variant="h6" gutterBottom>
        Uncertainty Cell Test
      </Typography>
      
      <Box sx={{ mb: 2 }}>
        <TextField
          label="Test Value"
          value={testValue}
          onChange={(e) => setTestValue(e.target.value)}
          fullWidth
          margin="normal"
        />
      </Box>

      <Box sx={{ display: 'flex', gap: 1, mb: 2, flexWrap: 'wrap' }}>
        <Button variant="outlined" onClick={handleTestDetection}>
          Test Detection
        </Button>
        <Button variant="outlined" onClick={handleTestToggle}>
          Enable Uncertainty Mode
        </Button>
        <Button variant="outlined" onClick={handleTestSetValue}>
          Set Uncertainty Value
        </Button>
        <Button variant="outlined" onClick={handleTestGetComponents}>
          Get Components
        </Button>
      </Box>

      <Box sx={{ mb: 2, p: 2, border: '1px solid #ccc', borderRadius: 1 }}>
        <Typography variant="subtitle2" gutterBottom>
          Uncertainty Cell Component:
        </Typography>
        <Box sx={{ width: 200, height: 40, border: '1px solid #ddd' }}>
          <UncertaintyCell
            cellRef={testCellRef}
            initialValue={testValue}
            isEditing={isEditing}
            onValueChange={(value) => {
              setTestValue(value);
              setResult(`Value changed to: ${value}`);
            }}
            onEditingChange={setIsEditing}
            onFocusAreaChange={(area) => {
              setResult(`Focus area changed to: ${area}`);
            }}
          />
        </Box>
        <Button 
          variant="text" 
          size="small" 
          onClick={() => setIsEditing(!isEditing)}
          sx={{ mt: 1 }}
        >
          {isEditing ? 'Stop Editing' : 'Start Editing'}
        </Button>
      </Box>

      <Box sx={{ p: 2, bgcolor: 'grey.100', borderRadius: 1 }}>
        <Typography variant="subtitle2" gutterBottom>
          Result:
        </Typography>
        <Typography variant="body2" component="pre" sx={{ whiteSpace: 'pre-wrap' }}>
          {result || 'No result yet'}
        </Typography>
      </Box>
    </Box>
  );
};

export default UncertaintyTest;