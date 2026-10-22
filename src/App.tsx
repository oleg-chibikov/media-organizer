import { useAppSession } from "./app/useAppSession";
import {
  DestinationSettingsMenu,
  GroupByPlaceToggle,
  RenameInPlaceToggle
} from "./features/destination-settings";
import { EmptyFileList, FileTable } from "./features/file-table";
import { HistoryMenu } from "./features/history";
import { HomeLocationSection } from "./features/home-location";
import { MoveConfirmationDialog } from "./features/moves";
import { StatusBar } from "./features/status-bar";
import { Toolbar } from "./features/toolbar";

export default function App() {
  const session = useAppSession();
  const { scan, planning, moves, places, history, destination, destinationActions } = session;
  const { effectiveDestination } = session;
  const { execution } = moves;
  const chooseFolder = () => void scan.chooseFolderAndScan();
  const renamesInPlace = destination.settings?.rename_in_place ?? false;
  const settingsLocked = destination.loading || destination.pending || destinationActions.locked;

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
        placeToggle={
          <GroupByPlaceToggle
            checked={destination.settings?.group_by_place ?? true}
            renamesInPlace={renamesInPlace}
            disabled={settingsLocked}
            placesHint={places.hint}
            onChange={destinationActions.setGroupByPlace}
          />
        }
        renameToggle={
          <RenameInPlaceToggle
            checked={renamesInPlace}
            disabled={settingsLocked}
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
            onOpen={destination.refresh}
            homeSection={
              <HomeLocationSection
                home={destination.settings?.home ?? null}
                disabled={destination.loading || destination.pending}
                onChange={destinationActions.setHome}
                onLocate={destinationActions.locateHome}
              />
            }
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
            filePlaces={places.filePlaces}
            locatingPlaces={places.locating}
          />
        )}
      </main>

      <StatusBar
        health={session.health}
        scan={scan.progress}
        metadata={places.progress}
        planning={planning}
        groupByPlace={session.groupByPlace}
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
          groupByPlace={session.groupByPlace}
          onConfirm={moves.confirmation.confirm}
          onCancel={moves.confirmation.cancel}
        />
      ) : null}
    </div>
  );
}
