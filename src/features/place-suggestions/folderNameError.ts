const FORBIDDEN_CHARACTERS = /[/\\:*?"<>|]/;

// Catches what the backend would reject, plus the characters Windows forbids in folder names.
export function folderNameError(name: string): string | null {
  const trimmed = name.trim();
  if (trimmed === "") {
    return "Enter a folder name.";
  }
  if (FORBIDDEN_CHARACTERS.test(trimmed)) {
    return 'Remove / \\ : * ? " < > |';
  }
  if (/^\.+$/.test(trimmed)) {
    return "Use a name with letters or digits.";
  }
  return null;
}
