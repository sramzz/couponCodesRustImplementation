use rust_xlsxwriter::*;
use std::path::Path;


/// Errors that can occur during export.
#[derive(Debug)]
pub enum ExportError {
    /// File system error (permissions, disk full, etc).
    IoError(String),
    /// Error writing Excel content.
    XlsxError(String),
}

/// Exports coupons to one or more Excel files.
///
/// - Each file contains at most `max_per_file` coupons.
/// - File naming: `{date}_{base_name}_{batch_number}.xlsx`, where `date` is today's date.
/// - No header row is included; the file only contains coupon codes starting from the first row.
///
/// Returns a list of file paths that were created.
///
/// # Arguments
/// - `coupons` — the full list of generated coupons
/// - `output_dir` — the folder where files will be saved
/// - `base_name` — the base file name (without extension)
/// - `max_per_file` — the maximum number of coupons per Excel file
pub fn export_to_excel(
    coupons: &[String],
    output_dir: &Path,
    base_name: &str,
    max_per_file: usize,
) -> Result<Vec<String>, ExportError> {
    // ── Split into chunks ───────────────────────────────────
    // If the list is empty, we still produce one file (with just a header).
    // Otherwise, chunk into groups of max_per_file.
    let chunks: Vec<&[String]> = if coupons.is_empty() {
        vec![&[]]
    } else {
        let chunk_size = if max_per_file == 0 { 10_000 } else { max_per_file };
        coupons.chunks(chunk_size).collect()
    };

    let mut created_files: Vec<String> = Vec::new();
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    for (index, chunk) in chunks.iter().enumerate() {
        // ── Build file name ─────────────────────────────────
        let batch_number = index + 1;
        let file_name = format!("{}_{}_{}.xlsx", today, base_name, batch_number);

        let full_path = output_dir.join(&file_name);

        // ── Create workbook and worksheet ───────────────────
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();

        // Write each coupon on its own row (row 0, 1, 2, ...)
        for (row, coupon) in chunk.iter().enumerate() {
            worksheet
                .write_string(row as u32, 0, coupon)
                .map_err(|e| ExportError::XlsxError(e.to_string()))?;
        }

        // ── Save to disk ────────────────────────────────────
        workbook
            .save(&full_path)
            .map_err(|e| ExportError::IoError(e.to_string()))?;

        created_files.push(full_path.to_string_lossy().to_string());
    }

    Ok(created_files)
}
