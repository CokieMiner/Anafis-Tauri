# Data Validation Sidebar ✓

**Status**: Planned  
**Priority**: Medium  
**Complexity**: Medium  
**Dependencies**: Rust backend (regex crate, chrono)

---

## Purpose

Define validation rules for data entry and quality control in spreadsheet cells, ensuring data integrity in real-time. All validation logic executed in Rust for performance and consistency.

**Architecture**: Rust handles ALL validation logic, rule evaluation, range checking. TypeScript only for UI rendering and calling validation commands.

---

## Features

### Validation Types
- **Numeric Range**: Min/max value constraints
- **Type Checking**: Number, text, date, boolean
- **Pattern Matching**: Regex patterns for text validation
- **Custom Formula**: User-defined validation formulas
- **List Validation**: Dropdown of allowed values
- **Length Validation**: Min/max string length

### Real-time Validation
- Check as user types/edits cells
- Immediate visual feedback
- Non-blocking (doesn't prevent input)

### Visual Feedback
- Red background for invalid cells
- Yellow background for warnings
- Green checkmark for valid cells
- Error icon with tooltip

### Bulk Validation
- Validate existing data in ranges
- Generate validation report
- Export invalid cells list

---

## UI Layout

```
┌─────────────────────────────────────┐
│ Data Validation                 [X] │
├─────────────────────────────────────┤
```
┌─────────────────────────────────────┐
│ Data Validation                 [X] │
├─────────────────────────────────────┤
│ Sheet: [Sheet1 ▼]                  │
│                                     │
│ ━━━ Active Rules ━━━               │
│                                     │
│ ┌─────────────────────────────┐   │
│ │ Rule 1: A1:A100             │   │
│ │ Type: Numeric Range         │   │
│ │ 0 ≤ value ≤ 100             │   │
│ │ Status: 5 errors  [Edit][x]│   │
│ └─────────────────────────────┘   │
│                                     │
│ ┌─────────────────────────────┐   │
│ │ Rule 2: B1:B100             │   │
│ │ Type: Type Check (Number)   │   │
│ │ Status: Valid     [Edit][x]│   │
│ └─────────────────────────────┘   │
│                                     │
│ [+ Add New Rule]                   │
│                                     │
│ ━━━ Create/Edit Rule ━━━           │
│                                     │
│ Range: [A1:A100] [Select from 📋]  │
│                                     │
│ Rule Type: [Numeric Range ▼]       │
│  • Numeric Range                   │
│  • Type Checking                   │
│  • Pattern Match (Regex)           │
│  • Custom Formula                  │
│  • List (Dropdown)                 │
│  • Text Length                     │
│                                     │
│ ┌───────────────────────────────┐ │
│ │ Rule Parameters               │ │
│ │                               │ │
│ │ Min Value: [0_____________]   │ │
│ │ Max Value: [100___________]   │ │
│ │                               │ │
│ │ [✓] Allow blank cells         │ │
│ │ [✓] Show error message        │ │
│ │ [✓] Show input message        │ │
│ │ [✓] Highlight invalid cells   │ │
│ └───────────────────────────────┘ │
│                                     │
│ Messages:                           │
│  Input Message (before entry):     │
│  ┌─────────────────────────────┐  │
│  │ Enter a value between 0     │  │
│  │ and 100                     │  │
│  └─────────────────────────────┘  │
│                                     │
│  Error Message (invalid entry):    │
│  ┌─────────────────────────────┐  │
│  │ Value must be between       │  │
│  │ 0 and 100                   │  │
│  └─────────────────────────────┘  │
│                                     │
│ [Validate Existing Data]           │
│ [Save Rule] [Cancel]               │
│                                     │
│ ━━━ Validation Status ━━━          │
│                                     │
│ ┌─────────────────────────────┐   │
│ │ Valid:   95 cells (95%)     │   │
│ │ Invalid: 5 cells (5%)       │   │
│ │ Blank:   0 cells (0%)       │   │
│ └─────────────────────────────┘   │
│                                     │
│ Invalid Cells (click to jump):      │
│  → A15: 127.5 (> max: 100)         │
│  → A47: -5.3  (< min: 0)           │
│  → A89: "text" (not numeric)       │
│  [Show All ▼]                      │
│                                     │
│ [Apply Rule] [Remove Rule]         │
│ [Clear Highlighting]               │
│ [Export Report]                    │
└─────────────────────────────────────┘
```

---

## Data Flow Pattern

**Type**: Monitor → Validate → Highlight (Pattern C)

1. User defines validation rule for a range
2. Rule stored in validation rules store
3. Monitor Univer cell change events
4. On cell change in monitored range, validate new value
5. If invalid, apply visual styling using Univer formatting
6. Display error message in sidebar/tooltip
7. No data modification - only visual feedback
8. User can fix invalid values

---

## Technical Implementation

### TypeScript Interfaces

```typescript
interface DataValidationDialogProps {
  open: boolean;
  onClose: () => void;
  univerRef: UniverSpreadsheetRef;
  onSelectionChange: (cellRef: string) => void;
}

type ValidationType = 
  | 'numeric_range' 
  | 'type_check' 
  | 'pattern' 
  | 'custom_formula'
  | 'list'
  | 'text_length';

interface ValidationRule {
  id: string;
  range: string;
  type: ValidationType;
  
  // Parameters based on type
  parameters: {
    // Numeric range
    min?: number;
    max?: number;
    
    // Type checking
    allowedTypes?: ('number' | 'text' | 'date' | 'boolean')[];
    
    // Pattern matching
    pattern?: string; // Regex pattern
    patternFlags?: string; // Regex flags (i, g, m)
    
    // Custom formula
    formula?: string; // e.g., "=A1>0 AND A1<100"
    
    // List validation
    list?: string[]; // Allowed values
    listRange?: string; // Or reference to range containing list
    
    // Text length
    minLength?: number;
    maxLength?: number;
  };
  
  // Options
  allowBlank: boolean;
  showErrorMessage: boolean;
  showInputMessage: boolean;
  
  // Messages
  inputMessage?: {
    title: string;
    text: string;
  };
  errorMessage?: {
    title: string;
    text: string;
  };
  
  // Styling
  errorStyle?: {
    backgroundColor: string;
    textColor: string;
    borderColor?: string;
  };
}

interface ValidationError {
  cellRef: string;
  value: any;
  rule: ValidationRule;
  reason: string;
  timestamp: Date;
}

interface ValidationReport {
  totalCells: number;
  validCells: number;
  invalidCells: number;
  blankCells: number;
  errors: ValidationError[];
}

// Validation rules store
class ValidationRulesStore {
  private rules: Map<string, ValidationRule[]>; // cellRef -> rules
  
  addRule(range: string, rule: ValidationRule): void;
  removeRule(ruleId: string): void;
  getRulesForCell(cellRef: string): ValidationRule[];
  getAllRules(): ValidationRule[];
  clear(): void;
}
```

### Validation Functions

```typescript
---

## Technical Implementation

---

## Rust Backend Implementation

### Validation Engine

```rust
// AnaFis/src-tauri/src/validation/mod.rs

use serde::{Deserialize, Serialize};
use regex::Regex;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ValidationRule {
    NumericRange {
        min: Option<f64>,
        max: Option<f64>,
        allow_blank: bool,
    },
    TypeCheck {
        allowed_types: Vec<String>, // "number", "text", "date", "boolean"
        allow_blank: bool,
    },
    Pattern {
        pattern: String,
        flags: Option<String>,
        allow_blank: bool,
    },
    List {
        allowed_values: Vec<String>,
        allow_blank: bool,
    },
    TextLength {
        min_length: Option<usize>,
        max_length: Option<usize>,
        allow_blank: bool,
    },
    CustomFormula {
        formula: String,
        allow_blank: bool,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationError {
    pub cell_ref: String,
    pub value: String,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationReport {
    pub total_cells: usize,
    pub valid_cells: usize,
    pub invalid_cells: usize,
    pub blank_cells: usize,
    pub errors: Vec<ValidationError>,
}

// Validate a single value against a rule
#[tauri::command]
pub fn validate_value(value: String, rule: ValidationRule) -> Result<ValidationResult, String> {
    // Handle blank values
    let is_blank = value.trim().is_empty();
    
    let allow_blank = match &rule {
        ValidationRule::NumericRange { allow_blank, .. } => *allow_blank,
        ValidationRule::TypeCheck { allow_blank, .. } => *allow_blank,
        ValidationRule::Pattern { allow_blank, .. } => *allow_blank,
        ValidationRule::List { allow_blank, .. } => *allow_blank,
        ValidationRule::TextLength { allow_blank, .. } => *allow_blank,
        ValidationRule::CustomFormula { allow_blank, .. } => *allow_blank,
    };
    
    if is_blank {
        if allow_blank {
            return Ok(ValidationResult {
                valid: true,
                reason: None,
            });
        } else {
            return Ok(ValidationResult {
                valid: false,
                reason: Some("Blank value not allowed".to_string()),
            });
        }
    }
    
    // Validate based on rule type
    match rule {
        ValidationRule::NumericRange { min, max, .. } => {
            match value.parse::<f64>() {
                Ok(num) => {
                    if let Some(min_val) = min {
                        if num < min_val {
                            return Ok(ValidationResult {
                                valid: false,
                                reason: Some(format!("Value {} is less than minimum {}", num, min_val)),
                            });
                        }
                    }
                    if let Some(max_val) = max {
                        if num > max_val {
                            return Ok(ValidationResult {
                                valid: false,
                                reason: Some(format!("Value {} exceeds maximum {}", num, max_val)),
                            });
                        }
                    }
                    Ok(ValidationResult {
                        valid: true,
                        reason: None,
                    })
                }
                Err(_) => Ok(ValidationResult {
                    valid: false,
                    reason: Some(format!("'{}' is not a valid number", value)),
                }),
            }
        }
        
        ValidationRule::TypeCheck { allowed_types, .. } => {
            let value_type = determine_type(&value);
            
            if allowed_types.contains(&value_type) {
                Ok(ValidationResult {
                    valid: true,
                    reason: None,
                })
            } else {
                Ok(ValidationResult {
                    valid: false,
                    reason: Some(format!(
                        "Invalid type: expected {}, got {}",
                        allowed_types.join(" or "),
                        value_type
                    )),
                })
            }
        }
        
        ValidationRule::Pattern { pattern, flags, .. } => {
            let regex = if let Some(f) = flags {
                // Handle flags like "i" for case-insensitive
                if f.contains('i') {
                    Regex::new(&format!("(?i){}", pattern))
                } else {
                    Regex::new(&pattern)
                }
            } else {
                Regex::new(&pattern)
            };
            
            match regex {
                Ok(re) => {
                    if re.is_match(&value) {
                        Ok(ValidationResult {
                            valid: true,
                            reason: None,
                        })
                    } else {
                        Ok(ValidationResult {
                            valid: false,
                            reason: Some("Value does not match required pattern".to_string()),
                        })
                    }
                }
                Err(e) => Err(format!("Invalid regex pattern: {}", e)),
            }
        }
        
        ValidationRule::List { allowed_values, .. } => {
            if allowed_values.contains(&value) {
                Ok(ValidationResult {
                    valid: true,
                    reason: None,
                })
            } else {
                Ok(ValidationResult {
                    valid: false,
                    reason: Some(format!(
                        "'{}' is not in the allowed list",
                        value
                    )),
                })
            }
        }
        
        ValidationRule::TextLength { min_length, max_length, .. } => {
            let len = value.len();
            
            if let Some(min) = min_length {
                if len < min {
                    return Ok(ValidationResult {
                        valid: false,
                        reason: Some(format!("Text too short: {} characters (minimum {})", len, min)),
                    });
                }
            }
            
            if let Some(max) = max_length {
                if len > max {
                    return Ok(ValidationResult {
                        valid: false,
                        reason: Some(format!("Text too long: {} characters (maximum {})", len, max)),
                    });
                }
            }
            
            Ok(ValidationResult {
                valid: true,
                reason: None,
            })
        }
        
        ValidationRule::CustomFormula { formula, .. } => {
            // TODO: Integrate with formula evaluation engine
            // For now, placeholder
            Ok(ValidationResult {
                valid: true,
                reason: Some("Custom formula validation not yet implemented".to_string()),
            })
        }
    }
}

// Validate a range of values
#[tauri::command]
pub fn validate_range(
    values: Vec<String>,
    cell_refs: Vec<String>,
    rule: ValidationRule,
) -> Result<ValidationReport, String> {
    if values.len() != cell_refs.len() {
        return Err("Values and cell references must have same length".to_string());
    }
    
    let mut valid_count = 0;
    let mut invalid_count = 0;
    let mut blank_count = 0;
    let mut errors = Vec::new();
    
    for (value, cell_ref) in values.iter().zip(cell_refs.iter()) {
        let result = validate_value(value.clone(), rule.clone())?;
        
        if value.trim().is_empty() {
            blank_count += 1;
        }
        
        if result.valid {
            valid_count += 1;
        } else {
            invalid_count += 1;
            errors.push(ValidationError {
                cell_ref: cell_ref.clone(),
                value: value.clone(),
                reason: result.reason.unwrap_or_else(|| "Unknown error".to_string()),
            });
        }
    }
    
    Ok(ValidationReport {
        total_cells: values.len(),
        valid_cells: valid_count,
        invalid_cells: invalid_count,
        blank_cells: blank_count,
        errors,
    })
}

// Helper function to determine value type
fn determine_type(value: &str) -> String {
    // Try to parse as number
    if value.parse::<f64>().is_ok() {
        return "number".to_string();
    }
    
    // Try to parse as boolean
    let lower = value.to_lowercase();
    if lower == "true" || lower == "false" {
        return "boolean".to_string();
    }
    
    // Try to parse as date (ISO format)
    if chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok() ||
       chrono::DateTime::parse_from_rfc3339(value).is_ok() {
        return "date".to_string();
    }
    
    // Default to text
    "text".to_string()
}
```

---

## TypeScript Implementation (UI Only)

### Validation Functions (Simplified - Call Rust)

```typescript
// REMOVED: All validation logic moved to Rust
// TypeScript only calls Rust commands:

import { invoke } from '@tauri-apps/api/tauri';

// Validate single value (calls Rust)
async function validateValue(
  value: any,
  rule: ValidationRule
): Promise<{ valid: boolean; reason?: string }> {
  return await invoke('validate_value', {
    value: String(value),
    rule
  });
}

// Validate range (calls Rust)
async function validateRange(
  values: any[],
  cellRefs: string[],
  rule: ValidationRule
): Promise<ValidationReport> {
  return await invoke('validate_range', {
    values: values.map(v => String(v)),
    cellRefs,
    rule
  });
}

// Validate all cells in a range
async function validateRange(
  univerRef: UniverSpreadsheetRef,
  range: string,
  rule: ValidationRule
): Promise<ValidationReport> {
  const data = await univerRef.current?.getRange(range);
  if (!data) {
    throw new Error('Could not read data from range');
  }
  
  const errors: ValidationError[] = [];
  let validCells = 0;
  let invalidCells = 0;
  let blankCells = 0;
  
  data.flat().forEach((value, index) => {
    const cellRef = `${range}[${index}]`; // Simplified, need actual cell ref
    
    if (value === null || value === undefined || value === '') {
      blankCells++;
      if (!rule.allowBlank) {
        errors.push({
          cellRef,
          value,
          rule,
          reason: 'Blank cell not allowed',
          timestamp: new Date()
        });
        invalidCells++;
      } else {
        validCells++;
      }
    } else {
      const result = validateValue(value, rule);
      if (result.valid) {
        validCells++;
      } else {
        invalidCells++;
        errors.push({
          cellRef,
          value,
          rule,
          reason: result.reason || 'Invalid value',
          timestamp: new Date()
        });
      }
    }
  });
  
  return {
    totalCells: data.flat().length,
    validCells,
    invalidCells,
    blankCells,
    errors
  };
}

// Apply visual feedback
async function highlightInvalidCells(
  univerRef: UniverSpreadsheetRef,
  errors: ValidationError[],
  rule: ValidationRule
): Promise<void> {
  const style = rule.errorStyle || {
    backgroundColor: '#ffebee',
    textColor: '#c62828',
    borderColor: '#f44336'
  };
  
  for (const error of errors) {
    await univerRef.current?.setCellStyle(error.cellRef, {
      backgroundColor: style.backgroundColor,
      color: style.textColor,
      border: `2px solid ${style.borderColor}`
    });
  }
}
```

---

## Dependencies

**Rust Dependencies**:
- regex 1.x (pattern matching validation)
- chrono 0.4 (date type detection)
- serde, serde_json
- tauri 1.x

**TypeScript Dependencies**:
- @tauri-apps/api (Tauri invoke)
- React, Material-UI (UI components)

**Note**: All validation logic in Rust, TypeScript only for UI

---

## File Location

- **Component**: `AnaFis/src/components/spreadsheet/DataValidationSidebar.tsx`
- **Rust Backend**: `AnaFis/src-tauri/src/validation/mod.rs`
- **Types**: `AnaFis/src/types/validation.ts` (TypeScript interfaces matching Rust enums)

---

## Success Criteria

- ✓ All validation types work correctly (numeric range, type check, pattern, list, length)
- ✓ Real-time validation via Rust commands
- ✓ Visual feedback applied correctly (red highlighting)
- ✓ Bulk validation generates accurate reports via Rust
- ✓ **All validation logic in Rust, TypeScript for UI only**
- ✓ Performance: Validate 10,000 cells in < 50ms (Rust performance)
- ✓ Sidebar allows managing multiple rules simultaneously
- ✓ Can edit/delete existing rules from sidebar
- ✓ Regex patterns validated and compiled in Rust
- ✓ Type detection accurate (number, text, date, boolean)

---

## Architecture Notes

**Rust-First Design**:
- All validation rules evaluated in Rust for consistency and performance
- Regex compilation and matching in Rust (faster than JavaScript)
- Type detection in Rust (more accurate)
- Benefits: 10x faster validation, consistent behavior, type-safe rule evaluation

**Validation Strategy**:
- Single-value validation for real-time feedback
- Bulk validation for existing data ranges
- All rules defined as Rust enums for type safety
- Custom formulas: Future integration with formula engine

---

**Next Steps**: Implement after Outlier Detection Sidebar
- Sidebar pattern allows continuous monitoring of validation status
- Better UX than dialog for managing multiple rules
- Rust validation engine reusable across app
