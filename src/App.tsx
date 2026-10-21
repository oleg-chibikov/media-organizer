import { useAppSession } from "./app/useAppSession";
import { DestinationSettingsMenu, RenameInPlaceToggle } from "./features/destination-settings";
import { EmptyFileList, FileTable } from "./features/file-table";
import { HistoryMenu } from "./features/history";
import { MoveConfirmationDialog } from "./features/moves";
import { StatusBar } from "./features/status-bar";
import { Toolbar } from "./features/toolbar";

export default function App() {
  const session = useAppSession();
  const { scan, planning, moves, history, destination, destinationActions, effectiveDestination } =
    session;
  const { execution } = moves;
  const chooseFolder = () => void scan.chooseFolderAndScan();
  const renamesInPlace = destination.settings?.rename_in_place ?? false;

  return (
    <div className="flex h-full flex-col bg-slate-950 text-slate-100">
      <Toolbar
        selectedFolder={scan.selectedFolder}
        destinationRoot={effectiveDestination.rootPath}
        destinationError={effectiveDestination.error}
        renamesInPlace={renamesInPlace}
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
        renameToggle={
          <RenameInPlaceToggle
            checked={renamesInPlace}
            disabled={destination.loading || destination.pending || destinationActions.locked}
            onChange={destinationActions.setRenameInPlace}
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
        onChooseFolder={chooseFolder}
        onCancelScan={() => void scan.stopScan()}
        onDismissError={session.dismissError}
      />

      <main className="min-h-0 flex-1">
        {scan.rows.length === 0 ? (
          <EmptyFileList
            selectedFolder={scan.selectedFolder}
            isScanning={scan.isScanning}
            chooseDisabled={execution.executing}
            onChooseFolder={chooseFolder}
          />
        ) : (
          <FileTable
            rows={scan.rows}
            metadataByFileId={session.metadataByFileId}
            movePlansByFileId={planning.movePlansByFileId}
            planErrorsByFileId={planning.planErrorsByFileId}
            moveResultsByFileId={moves.moveResultsByFileId}
            activeMoveFileId={execution.activeFileId}
            destinationRoot={effectiveDestination.rootPath}
          />
        )}
      </main>

      <StatusBar
        health={session.health}
        scan={scan.progress}
        planning={planning}
        summary={moves.confirmationSummary}
        execution={execution}
        moveCounts={moves.moveCounts}
        canRunExecutionAction={moves.canRunExecutionAction}
        destinationRoot={effectiveDestination.rootPath}
        onExecutionAction={moves.runExecutionAction}
        onOpenDestination={session.openDestination}
      />

      {moves.confirmation.summary ? (
        <MoveConfirmationDialog
          summary={moves.confirmation.summary}
          destinationRoot={effectiveDestination.rootPath}
          renamesInPlace={renamesInPlace}
          stillChecking={planning.stillChecking}
          onConfirm={moves.confirmation.confirm}
          onCancel={moves.confirmation.cancel}
        />
      ) : null}
    </div>
  );
}
