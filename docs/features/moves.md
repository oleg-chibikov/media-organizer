# Moves

Moves take the confirmed files to their planned paths. A run can start while the scan still reads dates. It picks up each file once its plan is final, and ends after the last check.

## What the person sees

- **Move Files...** in the status bar turns on once the scan finds a file. In Rename only mode the dialog says Rename instead of Move.
- **The dialog** asks `Move N files?` and shows the destination. While files are still being checked it asks about all files and says how many are ready now.
- **Warnings** in the dialog count files with no capture date, files that get a number, and files that stay where they are.
- **Earlier days** the dialog lists library folders of the same places that move too, under a checkbox. See [planning.md](./planning.md#earlier-days-of-a-place).
- **Progress** the status bar counts moved files and shows the backend's step, such as `Checking source and destination`.
- **Pause and Resume** replace Move Files during the run. Paused shows "Paused. The file moving now will finish."
- **Done** reads like "Done. Moved 120 files. Copied 3 files and kept the originals. 2 files didn't move. See Errors. You can undo this run in History." Only the parts that apply appear. **Open Destination** shows next to it.

## User flow

```mermaid
flowchart TD
  ready["Sees Ready rows"] --> press["Presses Move Files..."]
  press --> dialog{"Reads the dialog"}
  dialog -->|cancel| ready
  dialog -->|"unchecks earlier days"| replan["Plans update without them"] --> dialog
  dialog -->|Move| run["Rows turn Moving, then Moved"]
  run --> pause{"Presses Pause?"}
  pause -->|yes| paused["Current file finishes, the rest wait"]
  paused -->|Resume| run
  pause -->|no| done["Reads the Done summary"]
  done --> errors["Opens Errors for files that didn't move"]
  done --> open["Presses Open Destination"]
  done --> history["Undoes the run in History"]
```

## What happens inside

```mermaid
sequenceDiagram
  actor Person
  participant UI as features/moves
  participant Cmd as commands/move_execution.rs
  participant Session as scan_session/moves
  participant Run as move_run/
  participant Mover as mover/ and safe_fs/
  participant Hist as history/
  Person->>UI: Move Files...
  UI->>Cmd: get_move_confirmation_summary(scan_id, file_ids)
  Cmd-->>UI: counts, destination, library folders
  Person->>UI: Move
  UI->>Cmd: start_moves(scan_id)
  Cmd->>Session: checks the picked folder, starts a thread
  Session->>Hist: opens a run log
  loop each batch of ready files
    Session->>Run: batch
    Run-->>UI: move:progress {phase, completed, total}
    Run->>Mover: checks and moves one file
    Run->>Hist: one log line per file
    Run-->>UI: move:file {result}
  end
  Session->>Hist: closes the run, updates the index
  Session-->>UI: move:finished {moved, copied, skipped, failed}
  Session->>Session: removes empty source folders
  Session-->>UI: move:done {scan_id, removed_selected_folder, error}
```

1. **Start** `start_moves` fails unless the scan's folder was picked in the native dialog this session. A second start for the same scan fails too.
2. **Batches** the run thread moves every ready file, then waits for more. Files whose dates or places are still pending join later batches.
3. **End** the run ends once the scan, its reads and its final places are done and no ready file is left.
4. **Cleanup** after the run, the session removes empty folders under the picked folder, deepest first. The destination root is skipped.

## One file

```mermaid
flowchart TD
  op["Planned move"] --> safety{"Source a plain file inside the picked folder?<br/>Destination inside the root?"}
  safety -->|no| skipped["Skipped, source stays"]
  safety -->|yes| exists{"Destination free right now?"}
  exists -->|no| skipped
  exists -->|yes| rename{"No-replace rename works?"}
  rename -->|yes| moved["Moved"]
  rename -->|"other disk"| copy{"Verified copy works?"}
  rename -->|"other error"| failed["Failed, source stays"]
  copy -->|yes| copied["Copied, original kept"]
  copy -->|no| failed
  moved & copied & skipped & failed --> log{"Log line written?"}
  log -->|no| stop["Run stops, later files are Skipped"]
```

| Result    | Strategy           | What happened                                         |
| --------- | ------------------ | ----------------------------------------------------- |
| `moved`   | `rename`           | The file has its new path                             |
| `copied`  | `copy_keep_source` | A checked copy sits at the new path, the source stays |
| `skipped` | `not_attempted`    | A check failed before the move, the source stays      |
| `failed`  | `rename` or copy   | The move or copy failed, the source stays             |

- **No overwrite** the rename refuses a taken name. A file that appears after the check makes the move fail and both files stay.
- **Verified copy** writes a hidden temp file next to the destination, compares the bytes, then publishes it without replace.
- **History first** a step that can't be logged can't be undone, so the run stops moving.

The full list of checks is in [architecture.md](../architecture.md#file-safety-guarantees).

## Pause and resume

```mermaid
stateDiagram-v2
  [*] --> Moving: start_moves
  Moving --> Paused: pause_moves
  Paused --> Moving: resume_moves
  Moving --> Waiting: no ready file, checks still running
  Waiting --> Moving: new files ready
  Moving --> Cleaning: checks done, no file left
  Cleaning --> [*]: move:done
```

- **Pause** cancels the batch after the file moving now. Files it didn't reach get no result and stay at the source.
- **Reads pause too** the frontend calls `set_metadata_paused(true)`, so ExifTool starts no new reads.
- **No cleanup while paused** a paused run doesn't finish, so source folders stay until Resume.

## Code

| Side     | Files                                                                                             |
| -------- | ------------------------------------------------------------------------------------------------- |
| Frontend | `src/features/moves/`, `src/features/status-bar/`                                                 |
| Commands | `src-tauri/src/commands/move_execution.rs`, `get_move_confirmation_summary` in `commands/plan.rs` |
| Backend  | `src-tauri/src/scan_session/moves/`, `move_run/`, `mover/`, `safe_fs/`                            |
| Payloads | `src-tauri/src/models/move_execution.rs`                                                          |
| Spec     | `specs/002-streaming-queue-processing/`                                                           |
