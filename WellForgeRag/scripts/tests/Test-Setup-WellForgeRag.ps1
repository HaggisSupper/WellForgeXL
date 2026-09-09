$ErrorActionPreference = 'Stop'

$scriptPath = Join-Path (Split-Path $PSScriptRoot -Parent) 'Setup-WellForgeRag.ps1'
if (-not (Test-Path -LiteralPath $scriptPath)) {
    throw "Expected bootstrap script is missing: $scriptPath"
}

$tokens = $null
$errors = $null
[void][System.Management.Automation.Language.Parser]::ParseFile($scriptPath, [ref]$tokens, [ref]$errors)
if ($errors.Count -gt 0) {
    throw ("PowerShell parser errors:`n" + (($errors | ForEach-Object Message) -join "`n"))
}

$text = Get-Content -LiteralPath $scriptPath -Raw
if ($text -match '(?i)docker') {
    throw 'Bootstrap script must not depend on or invoke Docker.'
}

$required = @(
    'Rustlang.Rustup',
    'astral-sh.uv',
    'Microsoft.VisualStudio.2022.BuildTools',
    'Kitware.CMake',
    'Tesseract-OCR.Tesseract',
    'cargo fmt',
    'cargo clippy',
    'cargo test',
    'uv venv',
    'requirements.txt',
    '/v1/models',
    '/v1/embeddings',
    'wellforge-rag-server',
    'wellforge-rag-mcp',
    'wellforge-rag'
)

foreach ($needle in $required) {
    if (-not $text.Contains($needle)) {
        throw "Bootstrap script is missing required behavior marker: $needle"
    }
}

Write-Host 'WellForgeRag PowerShell bootstrap contract: PASS'
