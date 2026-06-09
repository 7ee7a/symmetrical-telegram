use std::path::{Path, PathBuf};
use umya_spreadsheet::{reader, writer};

pub fn process_files(pdf_paths: &[PathBuf], excel_path: &Path) -> Result<usize, String> {
    let mut spreadsheet = reader::xlsx::read(excel_path)
        .map_err(|e| format!("Failed to read Excel file: {}", e))?;
    
    let sheet = spreadsheet.get_active_sheet_mut();
    
    let mut column_map = std::collections::HashMap::new();
    let mut header_row_idx = 1;
    
    let expected_cols = vec![
        "SL.No", "Arr Flt No.", "Dep FltNo.", "Regn No.", "Origin", "Dest", 
        "Arr Date", "Arr Time", "Arr StdTyp", "Dep Date", "Dep Time", "Dep StdTyp", 
        "MTOW", "Landing", "Normal HRS", "Double HRS", "Remote HRS", "Parking"
    ];

    // Scan up to the first 50 rows to find the headers
    let mut found_headers = false;
    for row in 1..=50 {
        let mut temp_map = std::collections::HashMap::new();
        for col in 1..=100 {
            if let Some(cell) = sheet.get_cell((col, row)) {
                let val = cell.get_value().trim().to_string();
                if !val.is_empty() {
                    temp_map.insert(val.to_lowercase(), col);
                }
            }
        }
        
        // Check if this row contains at least one of our expected headers (case-insensitive)
        if expected_cols.iter().any(|&expected| temp_map.contains_key(&expected.to_lowercase())) {
            // Rebuild column_map with the formal expected name -> col
            for &expected in &expected_cols {
                if let Some(&col) = temp_map.get(&expected.to_lowercase()) {
                    column_map.insert(expected.to_string(), col);
                }
            }
            header_row_idx = row;
            found_headers = true;
            break;
        }
    }
    
    if !found_headers || column_map.is_empty() {
        return Err("Could not find any of the expected headers (like 'SL.No') in the first 50 rows of the Excel file.".into());
    }

    let mut total_rows_added = 0;
    
    // Find next empty row starting after the header row
    let mut next_row = header_row_idx + 1;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_read_pdf() {
        let path = PathBuf::from("./input.PDF");
        if path.exists() {
            let text = pdf_extract::extract_text(&path).unwrap();
            println!("PDF TEXT:\n{}", text);
        } else {
            println!("No input.PDF found");
        }
    }
}
