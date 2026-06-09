use std::path::{Path, PathBuf};
use umya_spreadsheet::{reader, writer};

pub fn process_files(pdf_paths: &[PathBuf], excel_path: &Path) -> Result<(usize, Vec<String>), String> {
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
    let mut warnings = Vec::new();
    
    // Find next empty row globally across the sheet
    let highest_row = sheet.get_highest_row();
    let mut next_row = if highest_row <= header_row_idx as u32 {
        header_row_idx as u32 + 1
    } else {
        highest_row + 1
    };

    for pdf_path in pdf_paths {
        let text = pdf_extract::extract_text(pdf_path)
            .map_err(|e| format!("Failed to extract text from {}: {:?}", pdf_path.display(), e))?;
            
        let (rows, skipped) = parse_pdf_text(&text)?;
        for s in skipped {
            warnings.push(format!("File {}: {}", pdf_path.file_name().unwrap_or_default().to_string_lossy(), s));
        }
        
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
        
    Ok((total_rows_added, warnings))
}

fn parse_pdf_text(text: &str) -> Result<(Vec<std::collections::HashMap<String, String>>, Vec<String>), String> {
    let mut results = Vec::new();
    let mut skipped_lines = Vec::new();
    let mut in_table = false;
    
    let expected_cols = vec![
        "SL.No", "Arr Flt No.", "Dep FltNo.", "Regn No.", "Origin", "Dest", 
        "Arr Date", "Arr Time", "Arr StdTyp", "Dep Date", "Dep Time", "Dep StdTyp", 
        "MTOW", "Landing", "Normal HRS", "Double HRS", "Remote HRS", "Parking"
    ];

    // Regex to match exactly 18 fields. Allow decimals in hour fields just in case.
    // Handles multi-word columns cleanly using dates and times as anchors.
    let pattern = r"^(\d+)\s+([a-zA-Z0-9]+\s*\d*)\s+([a-zA-Z0-9]+\s*\d*)\s+([a-zA-Z0-9]+)\s+([a-zA-Z0-9]+)\s+([a-zA-Z0-9]+)\s+(\d{2}[./-]\d{2}[./-]\d{4})\s+(\d{1,2}:\d{2}(?::\d{2})?)\s+(.*?)\s*(\d{2}[./-]\d{2}[./-]\d{4})\s+(\d{1,2}:\d{2}(?::\d{2})?)\s+(.*?)\s+([\d.]+)\s+([\d.]+)\s+([\d.]+)\s+([\d.]+)\s+([\d.]+)\s+([\d,.]+)$";
    let row_re = regex::Regex::new(pattern)
        .map_err(|e| format!("Regex compilation failed: {}", e))?;
    
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        
        if line.contains("SL.No") {
            in_table = true;
            continue;
        }
        
        if in_table && line.to_uppercase().contains("TOTAL") {
            in_table = false;
            continue;
        }
        
        if in_table {
            // Only process lines that perfectly match the 18-column aviation structure
            if let Some(caps) = row_re.captures(line) {
                let mut row_map = std::collections::HashMap::new();
                for i in 0..18 {
                    let val = caps.get(i + 1).map_or("", |m| m.as_str()).trim();
                    row_map.insert(expected_cols[i].to_string(), val.to_string());
                }
                results.push(row_map);
            } else {
                // Garbage lines, wrapped headers, or footers without the structure are safely ignored.
                // If it looks somewhat like a data row (contains digits) but fails regex, log it as skipped.
                if line.chars().any(|c| c.is_digit(10)) {
                    skipped_lines.push(line.to_string());
                }
            }
        }
    }
    
    Ok((results, skipped_lines))
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
