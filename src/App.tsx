import { useAppSession } from "./app/useAppSession";
import { DestinationSettingsMenu } from "./features/destination-settings";
import { FileTable } from "./features/file-table";
import { HistoryMenu } from "./features/history";
import { StatusBar } from "./features/status-bar";
import { Toolbar } from "./features/toolbar";

export default function App() {
  const session = useAppSession();
  const { scan, planning, moves, history, destination, destinationActions } = session;
  const { execution } = moves;
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
            onChooseRoot={destinationActions.chooseRoot}
            onClearRoot={destinationActions.clearRoot}
            onSetForceProcessed={destinationActions.setForceProcessed}
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
        scan={scan.progress}
        planning={planning}
        summary={moves.confirmationSummary}
        execution={execution}
        moveCounts={moves.moveCounts}
        hasPendingExecutableItems={moves.hasPendingExecutableItems}
        onExecutionAction={moves.runExecutionAction}
      />
    </div>
  );
}
