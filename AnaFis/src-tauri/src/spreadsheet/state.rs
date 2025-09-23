use std::sync::Mutex;
use super::cell::UnifiedCell;

pub struct SpreadsheetState {
    pub data: Mutex<Vec<Vec<UnifiedCell>>>,
}

impl SpreadsheetState {
    pub fn new() -> Self {
        // Initialize with a 100x100 grid of empty cells
        let data = vec![vec![UnifiedCell::Empty; 100]; 100];
        Self {
            data: Mutex::new(data),
        }
    }

    // CRUD operations will be implemented here
}
