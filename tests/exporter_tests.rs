use coupon_generator::exporter::export_to_excel;
use std::path::Path;
use tempfile::tempdir;

// Helper: creates a vector of dummy coupon strings.
fn dummy_coupons(count: usize) -> Vec<String> {
    (0..count).map(|i| format!("san{:07}", i)).collect()
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
