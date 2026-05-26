$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$targetDir = Join-Path $root "docs/agent/upstream"

New-Item -ItemType Directory -Force -Path $targetDir | Out-Null

$files = @(
  @{ Path = "agentsmd-AGENTS.md"; Url = "https://raw.githubusercontent.com/agentsmd/agents.md/main/AGENTS.md" },
  @{ Path = "conventional-commits-v1.0.0.md"; Url = "https://raw.githubusercontent.com/conventional-commits/conventionalcommits.org/master/content/v1.0.0/index.md" },
  @{ Path = "keep-a-changelog-README.md"; Url = "https://raw.githubusercontent.com/olivierlacan/keep-a-changelog/main/README.md" },
  @{ Path = "react-you-might-not-need-an-effect.md"; Url = "https://raw.githubusercontent.com/reactjs/react.dev/main/src/content/learn/you-might-not-need-an-effect.md" },
  @{ Path = "rust-api-guidelines-README.md"; Url = "https://raw.githubusercontent.com/rust-lang/api-guidelines/master/README.md" },
  @{ Path = "tailwind-content-configuration.mdx"; Url = "https://raw.githubusercontent.com/tailwindlabs/tailwindcss.com/master/src/pages/docs/content-configuration.mdx" },
  @{ Path = "tailwind-utility-first.mdx"; Url = "https://raw.githubusercontent.com/tailwindlabs/tailwindcss.com/master/src/pages/docs/utility-first.mdx" },
  @{ Path = "tauri-calling-rust.mdx"; Url = "https://raw.githubusercontent.com/tauri-apps/tauri-docs/v2/src/content/docs/develop/calling-rust.mdx" },
  @{ Path = "tauri-capabilities.mdx"; Url = "https://raw.githubusercontent.com/tauri-apps/tauri-docs/v2/src/content/docs/security/capabilities.mdx" },
  @{ Path = "vite-features.md"; Url = "https://raw.githubusercontent.com/vitejs/vite/main/docs/guide/features.md" },
  @{ Path = "vite-why.md"; Url = "https://raw.githubusercontent.com/vitejs/vite/main/docs/guide/why.md" }
)

foreach ($file in $files) {
  $dest = Join-Path $targetDir $file.Path
  Write-Host "Downloading $($file.Path)..."
  Invoke-WebRequest -Uri $file.Url -OutFile $dest -UseBasicParsing
}

Write-Host "Done. Downloaded $($files.Count) upstream reference files to $targetDir"
