use std::path::{Path, PathBuf};
use umya_spreadsheet::{reader, writer};

pub fn process_files(pdf_paths: &[PathBuf], excel_path: &Path) -> Result<usize, String> {
    let mut spreadsheet = reader::xlsx::read(excel_path)
        .map_err(|e| format!("Failed to read Excel file: {}", e))?;
    
    let sheet = spreadsheet.get_active_sheet_mut();
    
    let mut column_map = std::collections::HashMap::new();
    
    // Find the header row in Excel (assume row 1)
    for col in 1..=100 {
        if let Some(cell) = sheet.get_cell((col, 1)) {
            let val = cell.get_value().trim().to_string();
            if !val.is_empty() {
                column_map.insert(val, col);
            }
        }
    }
    
    if column_map.is_empty() {
        return Err("Could not find any headers in the first row of the Excel file.".into());
    }

    let mut total_rows_added = 0;
    
    // Find next empty row
    let mut next_row = 2;
    while sheet.get_cell((1, next_row)).is_some() && !sheet.get_value((1, next_row)).is_empty() {
        next_row += 1;
    }

    for pdf_path in pdf_paths {
        let text = pdf_extract::extract_text(pdf_path)
            .map_err(|e| format!("Failed to extract text from {}: {:?}", pdf_path.display(), e))?;
            
        let rows = parse_pdf_text(&text)?;
        
        for row_data in rows {
            for (header, value) in row_data {
                if let Some(&col_idx) = column_map.get(&header) {
                    sheet.get_cell_mut((col_idx, next_row)).set_value(value);
                }
            }
            next_row += 1;
            total_rows_added += 1;
        }
    }
    
    writer::xlsx::write(&spreadsheet, excel_path)
        .map_err(|e| format!("Failed to save Excel file: {}", e))?;
        
    Ok(total_rows_added)
}

fn parse_pdf_text(text: &str) -> Result<Vec<std::collections::HashMap<String, String>>, String> {
    let mut results = Vec::new();
    let mut in_table = false;
    
    let expected_cols = vec![
        "SL.No", "Arr Flt No.", "Dep FltNo.", "Regn No.", "Origin", "Dest", 
        "Arr Date", "Arr Time", "Arr StdTyp", "Dep Date", "Dep Time", "Dep StdTyp", 
        "MTOW", "Landing", "Normal HRS", "Double HRS", "Remote HRS", "Parking"
    ];
    
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        
        if line.contains("SL.No") {
            in_table = true;
            continue;
        }
        
        if in_table && line.to_uppercase().contains("TOTAL") {
            in_table = false;
            break;
        }
        
        if in_table {
            // Some columns might have spaces in data, but we use a simple split for now
            let parts: Vec<&str> = line.split_whitespace().collect();
            
            if parts.len() < 5 {
                continue;
            }
            
            let mut row_map = std::collections::HashMap::new();
            
            let mut col_idx = 0;
            for part in parts {
                if col_idx < expected_cols.len() {
                    row_map.insert(expected_cols[col_idx].to_string(), part.to_string());
                    col_idx += 1;
                }
            }
            
            if !row_map.is_empty() {
                results.push(row_map);
            }
        }
    }
    
    Ok(results)
}
