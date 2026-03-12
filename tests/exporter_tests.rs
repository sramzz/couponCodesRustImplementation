use coupon_generator::exporter::export_to_excel;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::io::{Read, Seek};
use std::path::Path;
use tempfile::tempdir;
use zip::ZipArchive;

// Helper: creates a vector of dummy coupon strings.
fn dummy_coupons(count: usize) -> Vec<String> {
    (0..count).map(|i| format!("san{:07}", i)).collect()
}

fn read_sheet_cells(workbook_path: &Path) -> HashMap<String, String> {
    let file = std::fs::File::open(workbook_path).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();
    let shared_strings = read_shared_strings(&mut archive);
    let sheet_xml = read_zip_entry(&mut archive, "xl/worksheets/sheet1.xml");

    parse_sheet_cells(&sheet_xml, &shared_strings)
}

fn read_shared_strings<R: Read + Seek>(archive: &mut ZipArchive<R>) -> Vec<String> {
    match archive.by_name("xl/sharedStrings.xml") {
        Ok(mut file) => {
            let mut xml = String::new();
            file.read_to_string(&mut xml).unwrap();
            parse_shared_strings(&xml)
        }
        Err(_) => Vec::new(),
    }
}

fn read_zip_entry<R: Read + Seek>(archive: &mut ZipArchive<R>, name: &str) -> String {
    let mut file = archive.by_name(name).unwrap();
    let mut xml = String::new();
    file.read_to_string(&mut xml).unwrap();
    xml
}

fn parse_shared_strings(xml: &str) -> Vec<String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut strings = Vec::new();
    let mut inside_text = false;
    let mut current = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref event)) if event.name().as_ref() == b"si" => {
                current.clear();
            }
            Ok(Event::Start(ref event)) if event.name().as_ref() == b"t" => {
                inside_text = true;
            }
            Ok(Event::Text(event)) if inside_text => {
                current.push_str(std::str::from_utf8(event.as_ref()).unwrap());
            }
            Ok(Event::End(ref event)) if event.name().as_ref() == b"t" => {
                inside_text = false;
            }
            Ok(Event::End(ref event)) if event.name().as_ref() == b"si" => {
                strings.push(current.clone());
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(error) => panic!("failed to parse shared strings xml: {error}"),
        }
    }

    strings
}

fn parse_sheet_cells(xml: &str, shared_strings: &[String]) -> HashMap<String, String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut cells = HashMap::new();
    let mut current_cell_ref: Option<String> = None;
    let mut current_cell_type: Option<String> = None;
    let mut current_value = String::new();
    let mut inside_value = false;
    let mut inside_inline_text = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref event)) if event.name().as_ref() == b"c" => {
                current_cell_ref = None;
                current_cell_type = None;
                current_value.clear();

                for attribute in event.attributes() {
                    let attribute = attribute.unwrap();
                    match attribute.key.as_ref() {
                        b"r" => {
                            current_cell_ref = Some(
                                std::str::from_utf8(attribute.value.as_ref())
                                    .unwrap()
                                    .to_string(),
                            );
                        }
                        b"t" => {
                            current_cell_type = Some(
                                std::str::from_utf8(attribute.value.as_ref())
                                    .unwrap()
                                    .to_string(),
                            );
                        }
                        _ => {}
                    }
                }
            }
            Ok(Event::Start(ref event)) if event.name().as_ref() == b"v" => {
                inside_value = true;
            }
            Ok(Event::Start(ref event)) if event.name().as_ref() == b"t" => {
                if current_cell_type.as_deref() == Some("inlineStr") {
                    inside_inline_text = true;
                }
            }
            Ok(Event::Text(event)) if inside_value || inside_inline_text => {
                current_value.push_str(std::str::from_utf8(event.as_ref()).unwrap());
            }
            Ok(Event::End(ref event)) if event.name().as_ref() == b"v" => {
                inside_value = false;
            }
            Ok(Event::End(ref event)) if event.name().as_ref() == b"t" => {
                inside_inline_text = false;
            }
            Ok(Event::End(ref event)) if event.name().as_ref() == b"c" => {
                if let Some(cell_ref) = current_cell_ref.take() {
                    let resolved_value = if current_cell_type.as_deref() == Some("s") {
                        let index = current_value.parse::<usize>().unwrap();
                        shared_strings[index].clone()
                    } else {
                        current_value.clone()
                    };
                    cells.insert(cell_ref, resolved_value);
                }
                current_cell_type = None;
                current_value.clear();
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(error) => panic!("failed to parse worksheet xml: {error}"),
        }
    }

    cells
}

// ───────────────────────────────────────────────
// FILE COUNT TESTS
// ───────────────────────────────────────────────

#[test]
fn small_batch_produces_one_file() {
    let coupons = dummy_coupons(100);
    let dir = tempdir().unwrap();
    let files = export_to_excel(&coupons, dir.path(), "coupons", 10_000).unwrap();
    assert_eq!(files.len(), 1);
}

#[test]
fn exactly_10000_coupons_produces_one_file() {
    let coupons = dummy_coupons(10_000);
    let dir = tempdir().unwrap();
    let files = export_to_excel(&coupons, dir.path(), "coupons", 10_000).unwrap();
    assert_eq!(files.len(), 1);
}

#[test]
fn just_over_10000_produces_two_files() {
    let coupons = dummy_coupons(10_001);
    let dir = tempdir().unwrap();
    let files = export_to_excel(&coupons, dir.path(), "coupons", 10_000).unwrap();
    assert_eq!(files.len(), 2);
}

#[test]
fn large_batch_splits_correctly() {
    let coupons = dummy_coupons(25_000);
    let dir = tempdir().unwrap();
    let files = export_to_excel(&coupons, dir.path(), "coupons", 10_000).unwrap();
    // 25,000 / 10,000 = 2 full files + 1 partial = 3 files
    assert_eq!(files.len(), 3);
}

#[test]
fn empty_list_produces_one_file() {
    let coupons: Vec<String> = vec![];
    let dir = tempdir().unwrap();
    let files = export_to_excel(&coupons, dir.path(), "coupons", 10_000).unwrap();
    assert_eq!(files.len(), 1);
}

#[test]
fn exported_file_writes_codes_header_and_starts_data_on_second_row() {
    let coupons = dummy_coupons(3);
    let dir = tempdir().unwrap();
    let files = export_to_excel(&coupons, dir.path(), "coupons", 10_000).unwrap();

    let cells = read_sheet_cells(Path::new(&files[0]));

    assert_eq!(cells.get("A1"), Some(&"codes".to_string()));
    assert_eq!(cells.get("A2"), Some(&coupons[0]));
    assert_eq!(cells.get("A3"), Some(&coupons[1]));
}

#[test]
fn empty_export_contains_only_the_codes_header() {
    let coupons: Vec<String> = vec![];
    let dir = tempdir().unwrap();
    let files = export_to_excel(&coupons, dir.path(), "coupons", 10_000).unwrap();

    let cells = read_sheet_cells(Path::new(&files[0]));

    assert_eq!(cells.get("A1"), Some(&"codes".to_string()));
    assert!(!cells.contains_key("A2"));
    assert_eq!(cells.len(), 1);
}

#[test]
fn split_exports_repeat_the_header_and_keep_data_starting_on_second_row() {
    let coupons = dummy_coupons(10_001);
    let dir = tempdir().unwrap();
    let files = export_to_excel(&coupons, dir.path(), "coupons", 10_000).unwrap();

    let first_cells = read_sheet_cells(Path::new(&files[0]));
    let second_cells = read_sheet_cells(Path::new(&files[1]));

    assert_eq!(first_cells.get("A1"), Some(&"codes".to_string()));
    assert_eq!(first_cells.get("A2"), Some(&coupons[0]));
    assert_eq!(second_cells.get("A1"), Some(&"codes".to_string()));
    assert_eq!(second_cells.get("A2"), Some(&coupons[10_000]));
}

#[test]
fn max_per_file_still_counts_coupon_rows_only() {
    let coupons = dummy_coupons(10_000);
    let dir = tempdir().unwrap();
    let files = export_to_excel(&coupons, dir.path(), "coupons", 10_000).unwrap();

    let cells = read_sheet_cells(Path::new(&files[0]));

    assert_eq!(files.len(), 1);
    assert_eq!(cells.get("A1"), Some(&"codes".to_string()));
    assert_eq!(cells.get("A10001"), Some(&coupons[9_999]));
}

// ───────────────────────────────────────────────
// FILE EXISTENCE TESTS
// ───────────────────────────────────────────────

#[test]
fn all_returned_file_paths_exist_on_disk() {
    let coupons = dummy_coupons(25_000);
    let dir = tempdir().unwrap();
    let files = export_to_excel(&coupons, dir.path(), "coupons", 10_000).unwrap();
    for file_path in &files {
        assert!(
            Path::new(file_path).exists(),
            "File does not exist: {}",
            file_path
        );
    }
}

// ───────────────────────────────────────────────
// FILE NAMING TESTS
// ───────────────────────────────────────────────

#[test]
fn files_use_date_base_name_and_batch_number() {
    let coupons = dummy_coupons(25_000);
    let dir = tempdir().unwrap();
    let files = export_to_excel(&coupons, dir.path(), "export", 10_000).unwrap();

    let names: Vec<String> = files
        .iter()
        .map(|f| {
            Path::new(f)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string()
        })
        .collect();

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    assert_eq!(names[0], format!("{}_export_1.xlsx", today));
    assert_eq!(names[1], format!("{}_export_2.xlsx", today));
    assert_eq!(names[2], format!("{}_export_3.xlsx", today));
}

// ───────────────────────────────────────────────
// FILE SIZE SANITY CHECK
// ───────────────────────────────────────────────

#[test]
fn generated_files_are_not_empty() {
    let coupons = dummy_coupons(100);
    let dir = tempdir().unwrap();
    let files = export_to_excel(&coupons, dir.path(), "coupons", 10_000).unwrap();
    for file_path in &files {
        let metadata = std::fs::metadata(file_path).unwrap();
        assert!(
            metadata.len() > 0,
            "File is empty: {}",
            file_path
        );
    }
}
