use crate::{
    models::{FileKind, MoveResultStatus, MoveStrategy, PlanStatus},
    mover::prepare_move_operations,
    test_support::media::{file_record, ready_plan},
};

#[test]
fn creates_operation_for_ready_media_plan() {
    let files = vec![file_record("IMG_1001", "jpg", FileKind::Photo)];
    let plans = vec![ready_plan(
        "IMG_1001",
        r"C:\todo\Processed\2026\05\23\2026-05-23 10-00-00 (IMG_1001).jpg",
    )];

    let prepared = prepare_move_operations(&files, &plans);

    assert_eq!(prepared.operations.len(), 1);
    assert_eq!(prepared.skipped.len(), 0);
    assert_eq!(prepared.operations[0].file_id, "IMG_1001");
}

#[test]
fn skips_when_plan_not_ready() {
    let files = vec![file_record("IMG_1001", "jpg", FileKind::Photo)];
    let mut plan = ready_plan("IMG_1001", r"C:\todo\Processed\2026\05\23\output.jpg");
    plan.plan_status = PlanStatus::BlockedNoDate;

    let prepared = prepare_move_operations(&files, &[plan]);

    assert_eq!(prepared.operations.len(), 0);
    assert_eq!(prepared.skipped.len(), 1);
    assert!(matches!(
        prepared.skipped[0].result_status,
        MoveResultStatus::Skipped
    ));
    assert!(matches!(
        prepared.skipped[0].move_strategy,
        MoveStrategy::NotAttempted
    ));
}

#[test]
fn skips_unsupported_file_kind() {
    let files = vec![file_record("note", "jpg", FileKind::Unsupported)];
    let plans = vec![ready_plan("note", r"C:\todo\Processed\2026\05\23\note.jpg")];

    let prepared = prepare_move_operations(&files, &plans);

    assert_eq!(prepared.operations.len(), 0);
    assert_eq!(prepared.skipped.len(), 1);
    assert_eq!(prepared.skipped[0].file_id, "note");
}
