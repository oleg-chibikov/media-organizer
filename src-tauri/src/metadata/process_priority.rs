use std::process::Command;

/// Runs ExifTool below the app and the desktop, so a long read doesn't stall the cursor.
#[cfg(unix)]
pub(super) fn lower_priority(command: &mut Command) {
    use std::os::unix::process::CommandExt;

    const BACKGROUND_NICE: libc::c_int = 10;
    // SAFETY: setpriority is async-signal-safe and changes only the child about to exec.
    unsafe {
        command.pre_exec(|| {
            libc::setpriority(libc::PRIO_PROCESS as _, 0, BACKGROUND_NICE);
            Ok(())
        });
    }
}

#[cfg(windows)]
pub(super) fn lower_priority(command: &mut Command) {
    use std::os::windows::process::CommandExt;

    const BELOW_NORMAL_PRIORITY_CLASS: u32 = 0x0000_4000;
    command.creation_flags(BELOW_NORMAL_PRIORITY_CLASS);
}
