import { createTextColumn, Column } from '@wasback/react-datasheet-grid';

// Define the structure for a value with uncertainty
export type Uncertainty = {
  value: number | null;
  uncertainty: number | null;
};

// Function to parse a string into an Uncertainty object
const parseUncertainty = (text: string): Uncertainty | null => {
  if (!text) {
    return null;
  }
  const parts = text.split('±');
  if (parts.length === 2) {
    const value = parseFloat(parts[0]);
    const uncertainty = parseFloat(parts[1]);
    if (!isNaN(value) && !isNaN(uncertainty)) {
      return { value, uncertainty };
    }
  }
  const value = parseFloat(text);
  if (!isNaN(value)) {
    return { value, uncertainty: null };
  }
  return null;
};

// Function to format an Uncertainty object into a string
const formatUncertainty = (data: Uncertainty | null): string => {
  if (!data) {
    return '';
  }
  if (data.uncertainty !== null) {
    return `${data.value} ± ${data.uncertainty}`;
  }
  if (data.value !== null) {
    return `${data.value}`;
  }
  return '';
};

// Create a new column type for uncertainty values
export const uncertaintyColumn = createTextColumn<Uncertainty | null>({
  parseUserInput: parseUncertainty,
  formatBlurredInput: formatUncertainty,
});
