import { Button } from "../../components/design-system/Button";

interface EmptyFileListProps {
  selectedFolder: string | null;
  isScanning: boolean;
  chooseDisabled: boolean;
  onChooseFolder: () => void;
}

export function EmptyFileList({
  selectedFolder,
  isScanning,
  chooseDisabled,
  onChooseFolder
}: EmptyFileListProps) {
  if (isScanning) {
    return <p className="p-6 text-sm text-slate-400">Looking for photos and videos...</p>;
  }

  return (
    <section className="mx-auto mt-16 max-w-md px-6 text-center">
      <h2 className="text-lg font-semibold text-slate-100">
        {selectedFolder
          ? "No photos or videos here"
          : "Organize photos and videos into folders by date"}
      </h2>
      <p className="mt-2 text-sm text-slate-400">
        {selectedFolder
          ? "This folder has no photos or videos to organize. Try another folder."
          : "Choose a folder. The app reads when each file was taken and shows where it will go."}
      </p>
      {selectedFolder ? null : (
        <p className="mt-2 text-sm text-slate-400">
          Files move only after you confirm. You can undo a run in History.
        </p>
      )}
      <Button
        variant="primary"
        className="mt-5 px-4 py-2 text-sm"
        disabled={chooseDisabled}
        onClick={onChooseFolder}
      >
        {selectedFolder ? "Choose Another Folder" : "Choose Folder"}
      </Button>
    </section>
  );
}
