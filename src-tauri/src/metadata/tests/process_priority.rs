use std::process::Command;

use crate::metadata::process_priority::lower_priority;

#[test]
fn starts_the_child_with_a_lower_priority() {
    let mut command = Command::new("sh");
    command.args(["-c", "ps -o nice= -p $$"]);
    lower_priority(&mut command);

    let output = command.output().expect("run ps");

    let nice: i32 = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .expect("nice value");
    assert!(nice > 0, "nice was {nice}");
}
