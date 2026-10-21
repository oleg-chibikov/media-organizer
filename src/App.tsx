import { useAppSession } from "./app/useAppSession";
import { DestinationSettingsMenu } from "./features/destination-settings";
import { FileTable } from "./features/file-table";
import { HistoryMenu } from "./features/history";
import { StatusBar } from "./features/status-bar";
import { Toolbar } from "./features/toolbar";

export default function App() {
  const session = useAppSession();
  const { scan, planning, moves, history, destination, destinationActions, effectiveDestination } =
    session;
  const { execution } = moves;

  return (
    <div className="flex h-full flex-col bg-slate-950 text-slate-100">
      <Toolbar
        selectedFolder={scan.selectedFolder}
        destinationRoot={effectiveDestination.rootPath}
        destinationError={effectiveDestination.error}
        folderError={session.folderError}
        isScanning={scan.isScanning}
        movesRunning={execution.executing}
        historyMenu={
          <HistoryMenu
            runs={history.runs}
            selectedRunId={history.selectedRunId}
            loading={history.loading}
            details={history.details}
            detailsLoading={history.detailsLoading}
            error={history.error}
            onSelectRun={history.selectRun}
            onUndoRun={history.undoRun}
            onRedoRun={history.redoRun}
            onDeleteRun={history.deleteRun}
            onOpenLog={history.openLog}
          />
        }
        settingsMenu={
          <DestinationSettingsMenu
            settings={destination.settings}
            loading={destination.loading}
            pending={destination.pending}
            locked={destinationActions.locked}
            error={destination.error}
            onChooseRoot={destinationActions.chooseRoot}
            onClearRoot={destinationActions.clearRoot}
            onSetForceProcessed={destinationActions.setForceProcessed}
          />
        }
        onChooseFolder={() => void scan.chooseFolderAndScan()}
        onCancelScan={() => void scan.stopScan()}
        onDismissError={session.dismissError}
      />

      <main className="min-h-0 flex-1">
        <FileTable
          rows={scan.rows}
          metadataByFileId={session.metadataByFileId}
          movePlansByFileId={planning.movePlansByFileId}
          moveResultsByFileId={moves.moveResultsByFileId}
          activeMoveFileId={execution.activeFileId}
          rootMediaFolder={destination.settings?.root_media_folder ?? null}
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
