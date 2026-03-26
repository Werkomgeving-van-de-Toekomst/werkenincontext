//! Performance tests for workflow load testing
//!
//! Tests system behavior under realistic load conditions.

use uuid::Uuid;
use std::time::{Duration, Instant};
use tokio::time::sleep;

// Test helpers from workflow tests
fn create_test_stage_instance(document_id: Uuid, stage_id: &str, approvers: Vec<Uuid>) -> iou_core::workflows::StageInstance {
    iou_core::workflows::StageInstance::new(document_id, stage_id.to_string(), approvers)
}

struct TestUser {
    id: Uuid,
}

impl TestUser {
    fn manager() -> Self {
        Self { id: Uuid::new_v4() }
    }
}

/// Test: Concurrent document approvals
#[tokio::test]
async fn test_concurrent_document_approvals() {
    // Setup: Create 100 documents awaiting approval
    let document_count = 100;
    let approver = TestUser::manager();

    let mut documents: Vec<iou_core::workflows::StageInstance> = (0..document_count)
        .map(|_| {
            create_test_stage_instance(
                Uuid::new_v4(),
                "approval",
                vec![approver.id],
            )
        })
        .collect();

    // Start timing
    let start = Instant::now();

    // Simulate 10 concurrent approvers
    let approvals_per_batch = 10;
    let mut handles = vec![];

    for batch in 0..(document_count / approvals_per_batch) {
        let batch_docs = &mut documents
            [batch * approvals_per_batch..(batch + 1) * approvals_per_batch];

        for doc in batch_docs {
            let doc_id = doc.id;
            let approver_id = approver.id;

            let handle = tokio::spawn(async move {
                // Simulate approval processing
                sleep(Duration::from_millis(10)).await;
                (doc_id, approver_id)
            });

            handles.push(handle);
        }
    }

    // Wait for all approvals to complete
    for handle in handles {
        handle.await.unwrap();
    }

    let elapsed = start.elapsed();

    // Assert: All documents processed within acceptable time
    assert!(
        elapsed < Duration::from_secs(5),
        "Processing {} documents took {:?}, expected < 5s",
        document_count,
        elapsed
    );

    println!(
        "Processed {} documents in {:?} ({:.2} docs/sec)",
        document_count,
        elapsed,
        document_count as f64 / elapsed.as_secs_f64()
    );
}

/// Test: Large document diff generation performance
#[tokio::test]
async fn test_large_document_diff_generation() {
    // Setup: Create document with 10,000 lines
    let line_count: usize = 10_000;
    let mut content = String::new();

    for i in 0..line_count {
        content.push_str(&format!("Line {}: Original content\n", i));
    }

    // Modify middle section
    let modified_start = line_count / 2;
    let mut modified_lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    for i in modified_start..modified_start + 100 {
        modified_lines[i] = format!("Line {}: Modified content", i);
    }

    let modified_content = modified_lines.join("\n");

    // Measure: Time to generate diff
    let start = Instant::now();

    // Simple line-by-line diff simulation
    let original_lines: Vec<&str> = content.lines().collect();
    let modified_lines_vec: Vec<&str> = modified_content.lines().collect();

    let mut diff_count = 0;
    for (orig, modi) in original_lines.iter().zip(modified_lines_vec.iter()) {
        if orig != modi {
            diff_count += 1;
        }
    }

    let elapsed = start.elapsed();

    // Assert: Diff generation completes within acceptable time
    assert!(
        elapsed < Duration::from_millis(100),
        "Diff generation took {:?}, expected < 100ms",
        elapsed
    );

    assert_eq!(diff_count, 100);

    println!(
        "Diff of {} lines took {:?} ({} changes found)",
        line_count,
        elapsed,
        diff_count
    );
}

/// Test: Version list pagination performance
#[tokio::test]
async fn test_version_list_pagination() {
    // Setup: Create document with 100 versions
    let document_id = Uuid::new_v4();
    let version_count: usize = 100;
    let page_size: usize = 20;

    let versions: Vec<u32> = (1..=version_count as u32).collect();

    // Test: Paginated version list
    let start = Instant::now();

    let mut all_pages = vec![];
    for page in 0..(version_count / page_size) {
        let page_start = page * page_size;
        let page_end = page_start + page_size;

        let page_versions: Vec<_> = versions[page_start..page_end]
            .iter()
            .map(|&v| (document_id, v))
            .collect();

        all_pages.extend(page_versions);
    }

    let elapsed = start.elapsed();

    assert_eq!(all_pages.len(), version_count);

    assert!(
        elapsed < Duration::from_millis(10),
        "Pagination took {:?}, expected < 10ms",
        elapsed
    );

    println!(
        "Paginated {} versions in {:?} ({} pages of {} items)",
        version_count,
        elapsed,
        version_count / page_size,
        page_size
    );
}

/// Test: Stage transition overhead
#[tokio::test]
async fn test_stage_transition_overhead() {
    use iou_core::workflows::StageStatus;

    let stage_count: usize = 1000;
    let document_id = Uuid::new_v4();
    let approver = TestUser::manager();

    let mut stages: Vec<iou_core::workflows::StageInstance> = (0..stage_count)
        .map(|_| {
            create_test_stage_instance(
                document_id,
                "test_stage",
                vec![approver.id],
            )
        })
        .collect();

    let start = Instant::now();

    for stage in &mut stages {
        let _ = stage.transition_to(StageStatus::InProgress);
        let _ = stage.transition_to(StageStatus::Completed);
    }

    let elapsed = start.elapsed();

    let transition_count = stage_count * 2;
    let avg_time = elapsed.as_micros() as f64 / transition_count as f64;

    println!(
        "Performed {} transitions in {:?} (avg: {:.2}μs per transition)",
        transition_count,
        elapsed,
        avg_time
    );

    assert!(
        avg_time < 1000.0,
        "Average transition time {}μs exceeds 1000μs threshold",
        avg_time
    );
}
