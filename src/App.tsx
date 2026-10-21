import { useAppSession } from "./app/useAppSession";
import { DestinationSettingsMenu } from "./components/DestinationSettingsMenu";
import { FileTable } from "./features/file-table/FileTable";
import { HistoryMenu } from "./features/history/HistoryMenu";
import { Toolbar } from "./features/toolbar/Toolbar";
import { StatusBar } from "./components/StatusBar";
import { chooseRootMediaFolder, clearRootMediaFolder, setForceProcessedFolder } from "./lib/api";

export default function App() {
  const session = useAppSession();
  const { scan, planning, moves, history, destination, changeDestinationSettings } = session;
  const { execution, moveCounts } = moves;
  const rootMediaFolder = destination.settings?.root_media_folder ?? null;

  return (
    <div className="flex h-full flex-col bg-slate-950 text-slate-100">
      <Toolbar
        selectedFolder={scan.selectedFolder}
        folderError={session.folderError}
        isScanning={scan.isScanning}
        historyMenu={
          <HistoryMenu
            runs={history.runs}
            selectedRunId={history.selectedRunId}
            loading={history.loading}
            details={history.details}
            detailsLoading={history.detailsLoading}
            detailsError={history.error}
            onSelectRun={history.selectRun}
            onUndoRun={history.undoRun}
            onRedoRun={history.redoRun}
            onDeleteRun={history.deleteRun}
          />
        }
        rootMediaFolder={rootMediaFolder}
        settingsMenu={
          <DestinationSettingsMenu
            settings={destination.settings}
            loading={destination.loading}
            pending={destination.pending}
            locked={execution.executing}
            error={destination.error}
            onChooseRoot={() => changeDestinationSettings(chooseRootMediaFolder)}
            onClearRoot={() => changeDestinationSettings(clearRootMediaFolder)}
            onSetForceProcessed={(nextValue) =>
              changeDestinationSettings(() => setForceProcessedFolder(nextValue))
            }
          />
        }
        onChooseFolder={() => void scan.chooseFolderAndScan()}
        onCancelScan={() => void scan.stopScan()}
      />

      <main className="min-h-0 flex-1">
        <FileTable
          rows={scan.rows}
          metadataByFileId={session.metadataByFileId}
          movePlansByFileId={planning.movePlansByFileId}
          moveResultsByFileId={moves.moveResultsByFileId}
          activeMoveFileId={execution.activeFileId}
          rootMediaFolder={rootMediaFolder}
        />
      </main>

      <StatusBar
        health={session.health}
        discovered={scan.progress.discovered}
        skipped={scan.progress.skipped}
        errors={scan.progress.errors}
        planErrors={planning.planErrors}
        confirmationSummary={moves.confirmationSummary}
        executionConfirmed={execution.confirmed}
        isPlanning={planning.isPlanning}
        planCompleted={planning.planCompleted}
        completed={scan.progress.completed}
        cancelled={scan.progress.cancelled}
        isExecutingMoves={execution.executing}
        isMoveDispatchPaused={execution.paused}
        moveStatusMessage={execution.statusMessage}
        moveCompleted={execution.completed}
        moved={moveCounts.moved}
        copied={moveCounts.copied}
        moveSkipped={moveCounts.skipped}
        moveFailed={moveCounts.failed}
        hasPendingExecutableItems={moves.hasPendingExecutableItems}
        onExecutionAction={moves.runExecutionAction}
      />
    </div>
  );
}
