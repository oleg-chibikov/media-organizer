use crate::{
    models::{ConflictStatus, DateKind, FileKind, PlanStatus},
    mover::build_move_confirmation_summary,
    test_support::media::{file_record, ready_metadata, ready_plan},
};

#[test]
fn builds_confirmation_summary_for_ready_and_blocked_items() {
    let files = vec![
        file_record("photo_1", "jpg", FileKind::Photo),
        file_record("video_1", "jpg", FileKind::Video),
    ];
    let mut blocked = ready_plan("video_1", r"C:\todo\Processed\2026\05\23\video_1.jpg");
    blocked.plan_status = PlanStatus::BlockedNoDate;
    let plans = vec![
        ready_plan("photo_1", r"C:\todo\Processed\2026\05\23\photo_1.jpg"),
        blocked,
    ];
    let metadata = vec![
        ready_metadata("photo_1", DateKind::Fallback),
        ready_metadata("video_1", DateKind::TimezoneAware),
    ];

    let summary = build_move_confirmation_summary(&files, &metadata, &plans);

    assert_eq!(summary.total_plans, 2);
    assert_eq!(summary.ready_operations, 1);
    assert_eq!(summary.skipped_operations, 1);
    assert_eq!(summary.blocked_no_date, 1);
    assert_eq!(summary.photos_ready, 1);
    assert_eq!(summary.videos_ready, 0);
    assert_eq!(summary.fallback_warnings, 1);
}

#[test]
fn counts_suffix_applied_conflicts() {
    let files = vec![file_record("photo_1", "jpg", FileKind::Photo)];
    let mut plan = ready_plan("photo_1", r"C:\todo\Processed\2026\05\23\photo_1 - 2.jpg");
    plan.conflict_status = ConflictStatus::SuffixApplied;
    let metadata = vec![ready_metadata("photo_1", DateKind::TimezoneAware)];

    let summary = build_move_confirmation_summary(&files, &metadata, &[plan]);

    assert_eq!(summary.suffix_applied, 1);
    assert_eq!(summary.ready_operations, 1);
    assert_eq!(summary.skipped_operations, 0);
}
