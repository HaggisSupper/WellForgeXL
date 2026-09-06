[CmdletBinding()]
param(
    [string]$RepositoryRoot,
    [string]$Branch = 'feat/wellforge-rag',
    [string]$EmbeddingBaseUrl = 'http://127.0.0.1:1234/v1',
    [string]$EmbeddingModel = 'text-embedding-nomic-embed-text-v1.5',
    [int]$EmbeddingDimension = 768,
    [string]$ConceptModel = 'local',
    [string]$IngestRoot,
    [ValidateSet('Auto', 'None', 'Server', 'Mcp', 'Cli')]
    [string]$Launch = 'Auto',
    [bool]$InstallPrerequisites = $true,
    [bool]$SyncGit = $true,
    [bool]$RunTests = $true,
    [bool]$CheckModels = $true,
    [bool]$Release = $true
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$script:LogPath = $null
$script:ConfigPath = $null
$script:SidecarPython = $null
$script:RagRoot = $null
$script:RepoRoot = $null
$script:EffectiveIngestRoot = $null

function Write-Step {
    param([Parameter(Mandatory)][string]$Message)
    Write-Host "`n==> $Message" -ForegroundColor Cyan
}

function Refresh-ProcessPath {
    $machine = [Environment]::GetEnvironmentVariable('Path', 'Machine')
    $user = [Environment]::GetEnvironmentVariable('Path', 'User')
    $env:Path = (($machine, $user) -join ';')
    $cargoBin = Join-Path $HOME '.cargo\bin'
    if (Test-Path -LiteralPath $cargoBin) {
        $env:Path = "$cargoBin;$env:Path"
    }
}

function Get-Executable {
    param([Parameter(Mandatory)][string]$Name)
    $command = Get-Command $Name -ErrorAction SilentlyContinue
    if ($null -eq $command) { return $null }
    return $command.Source
}

function Invoke-Native {
    param(
        [Parameter(Mandatory)][string]$FilePath,
        [string[]]$Arguments = @(),
        [string]$WorkingDirectory
    )

    if ($WorkingDirectory) { Push-Location $WorkingDirectory }
    try {
        & $FilePath @Arguments
        if ($LASTEXITCODE -ne 0) {
            throw "$FilePath failed with exit code $LASTEXITCODE. Arguments: $($Arguments -join ' ')"
        }
    }
    finally {
        if ($WorkingDirectory) { Pop-Location }
    }
}

function Test-Administrator {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($identity)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Install-WingetPackage {
    param(
        [Parameter(Mandatory)][string]$Id,
        [string[]]$ExtraArguments = @()
    )

    $winget = Get-Executable 'winget.exe'
    if (-not $winget) { throw 'winget.exe is required for automatic prerequisite installation.' }

    $arguments = @(
        'install', '--id', $Id, '--exact', '--source', 'winget',
        '--accept-source-agreements', '--accept-package-agreements', '--silent'
    ) + $ExtraArguments

    Write-Host "Installing $Id ..." -ForegroundColor Yellow
    if (Test-Administrator) {
        Invoke-Native -FilePath $winget -Arguments $arguments
    }
    else {
        $quoted = $arguments | ForEach-Object {
            if ($_ -match '\s') { '"' + ($_ -replace '"', '\"') + '"' } else { $_ }
        }
        $process = Start-Process -FilePath $winget -ArgumentList ($quoted -join ' ') -Verb RunAs -Wait -PassThru
        if ($process.ExitCode -ne 0) {
            throw "winget failed installing $Id with exit code $($process.ExitCode)."
        }
    }
    Refresh-ProcessPath
}

function Find-VCTools {
    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
    if (-not (Test-Path -LiteralPath $vswhere)) { return $null }
    $installation = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($installation)) { return $null }
    return $installation.Trim()
}

function Ensure-Prerequisites {
    Write-Step $(if ($InstallPrerequisites) { 'Checking/installing Windows build prerequisites' } else { 'Checking Windows build prerequisites' })

    $packages = @(
        @{ Command = 'git.exe'; Id = 'Git.Git' },
        @{ Command = 'rustup.exe'; Id = 'Rustlang.Rustup' },
        @{ Command = 'uv.exe'; Id = 'astral-sh.uv' },
        @{ Command = 'cmake.exe'; Id = 'Kitware.CMake' },
        @{ Command = 'tesseract.exe'; Id = 'Tesseract-OCR.Tesseract' }
    )

    foreach ($package in $packages) {
        if (-not (Get-Executable $package.Command)) {
            if (-not $InstallPrerequisites) {
                throw "Missing prerequisite $($package.Command). Re-run with -InstallPrerequisites `$true."
            }
            Install-WingetPackage -Id $package.Id
        }
    }

    if (-not (Find-VCTools)) {
        if (-not $InstallPrerequisites) { throw 'Microsoft Visual C++ Build Tools are missing.' }
        Install-WingetPackage -Id 'Microsoft.VisualStudio.2022.BuildTools' -ExtraArguments @(
            '--override',
            '--wait --passive --add Microsoft.VisualStudio.Component.VC.Tools.x86.x64 --add Microsoft.VisualStudio.Component.Windows11SDK.22621 --includeRecommended'
        )
    }

    Refresh-ProcessPath
    foreach ($command in @('git.exe', 'rustup.exe', 'uv.exe', 'cmake.exe')) {
        if (-not (Get-Executable $command)) {
            throw "Prerequisite installation completed but $command is not visible in PATH. Open a new PowerShell window and rerun the script."
        }
    }
}

function Resolve-Roots {
    if ($RepositoryRoot) {
        $candidate = (Resolve-Path -LiteralPath $RepositoryRoot).Path
        if ((Split-Path $candidate -Leaf) -eq 'WellForgeRag') {
            $script:RagRoot = $candidate
            $script:RepoRoot = Split-Path $candidate -Parent
        }
        elseif (Test-Path -LiteralPath (Join-Path $candidate 'WellForgeRag\Cargo.toml')) {
            $script:RepoRoot = $candidate
            $script:RagRoot = Join-Path $candidate 'WellForgeRag'
        }
        else {
            throw "RepositoryRoot does not contain WellForgeRag: $candidate"
        }
    }
    else {
        $script:RagRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
        $script:RepoRoot = (Resolve-Path -LiteralPath (Join-Path $script:RagRoot '..')).Path
    }

    if (-not (Test-Path -LiteralPath (Join-Path $script:RagRoot 'Cargo.toml'))) {
        throw "WellForgeRag Cargo.toml not found under $script:RagRoot"
    }

    $script:EffectiveIngestRoot = if ($IngestRoot) {
        (Resolve-Path -LiteralPath $IngestRoot).Path
    }
    else {
        $script:RepoRoot
    }
}

function Initialize-Logging {
    $logDir = Join-Path $script:RagRoot 'data\logs'
    New-Item -ItemType Directory -Force -Path $logDir | Out-Null
    $script:LogPath = Join-Path $logDir 'bootstrap.jsonl'
}

function Write-Event {
    param(
        [Parameter(Mandatory)][string]$Stage,
        [Parameter(Mandatory)][string]$Status,
        [string]$Detail = ''
    )

    if (-not $script:LogPath) { return }
    $event = [ordered]@{
        timestamp = [DateTimeOffset]::Now.ToString('o')
        stage = $Stage
        status = $Status
        detail = $Detail
    }
    ($event | ConvertTo-Json -Compress) | Add-Content -LiteralPath $script:LogPath -Encoding UTF8
}

function Sync-Repository {
    if (-not $SyncGit) { return }
    $git = Get-Executable 'git.exe'
    if (-not (Test-Path -LiteralPath (Join-Path $script:RepoRoot '.git'))) {
        Write-Host 'Repository is not a Git working tree; skipping sync.' -ForegroundColor Yellow
        return
    }

    Write-Step "Synchronizing Git branch $Branch"
    $dirty = & $git -C $script:RepoRoot status --porcelain
    if ($LASTEXITCODE -ne 0) { throw 'git status failed.' }
    if ($dirty) {
        Write-Host 'Working tree has local changes; preserving them and skipping checkout/pull.' -ForegroundColor Yellow
        Write-Event -Stage 'git-sync' -Status 'skipped' -Detail 'working tree has local changes'
        return
    }

    Invoke-Native -FilePath $git -Arguments @('-C', $script:RepoRoot, 'fetch', 'origin', $Branch)
    Invoke-Native -FilePath $git -Arguments @('-C', $script:RepoRoot, 'checkout', $Branch)
    Invoke-Native -FilePath $git -Arguments @('-C', $script:RepoRoot, 'pull', '--ff-only', 'origin', $Branch)
    Write-Event -Stage 'git-sync' -Status 'pass' -Detail $Branch
}

function Initialize-Rust {
    Write-Step 'Installing/selecting the pinned Rust toolchain'
    $rustup = Get-Executable 'rustup.exe'
    Invoke-Native -FilePath $rustup -Arguments @('toolchain', 'install', '1.98.0', '--profile', 'minimal', '--component', 'rustfmt', '--component', 'clippy')
    Write-Event -Stage 'rust-toolchain' -Status 'pass' -Detail '1.98.0'
}

function Initialize-PythonSidecar {
    Write-Step 'Preparing isolated Python extraction sidecar'
    $uv = Get-Executable 'uv.exe'
    $venv = Join-Path $script:RagRoot '.venv'
    $python = Join-Path $venv 'Scripts\python.exe'
    $requirements = Join-Path $script:RagRoot 'adapters\requirements.txt'

    Invoke-Native -FilePath $uv -Arguments @('python', 'install', '3.12')
    if (-not (Test-Path -LiteralPath $python)) {
        # Contract marker: uv venv
        Invoke-Native -FilePath $uv -Arguments @('venv', '--python', '3.12', $venv)
    }
    Invoke-Native -FilePath $uv -Arguments @('pip', 'install', '--python', $python, '-r', $requirements)

    if (-not (Test-Path -LiteralPath $python)) { throw 'Python sidecar virtual environment was not created.' }
    $script:SidecarPython = $python
    Write-Event -Stage 'python-sidecar' -Status 'pass' -Detail $python
}

function Convert-ToTomlPath {
    param([Parameter(Mandatory)][string]$Path)
    return ($Path -replace '\\', '/') -replace '"', '\"'
}

function Write-LocalConfig {
    Write-Step 'Writing machine-local RAG configuration'
    $configPath = Join-Path $script:RagRoot 'config\wellforge-rag.local.toml'
    $ingest = Convert-ToTomlPath $script:EffectiveIngestRoot
    $python = Convert-ToTomlPath $script:SidecarPython
    $adapter = Convert-ToTomlPath (Join-Path $script:RagRoot 'adapters\extract.py')
    $baseUrl = $EmbeddingBaseUrl.TrimEnd('/')

    $config = @"
[storage]
sqlite = "../data/wellforge-rag.sqlite3"
lancedb = "../data/lancedb"
okf = "../data/okf"

[server]
bind = "127.0.0.1:8765"

[ingest]
roots = ["$ingest"]
max_file_bytes = 536870912
max_extraction_bytes = 67108864
python = "$python"
python_adapter = "$adapter"

[embedding]
enabled = true
base_url = "$baseUrl"
model = "$EmbeddingModel"
dimension = $EmbeddingDimension
batch_size = 32
timeout_seconds = 60

[concepts]
enabled = true
base_url = "$baseUrl"
model = "$ConceptModel"
timeout_seconds = 90

[search]
lexical_weight = 1.0
semantic_weight = 1.0
candidate_limit = 40
"@

    Set-Content -LiteralPath $configPath -Value $config -Encoding UTF8
    $script:ConfigPath = $configPath
    Write-Event -Stage 'config' -Status 'pass' -Detail $configPath
}

function Test-LocalModelEndpoint {
    if (-not $CheckModels) { return }

    Write-Step 'Checking local OpenAI-compatible model endpoint'
    $baseUrl = $EmbeddingBaseUrl.TrimEnd('/')
    $modelsUri = "$baseUrl/models"  # /v1/models
    $embeddingsUri = "$baseUrl/embeddings"  # /v1/embeddings

    try {
        [void](Invoke-RestMethod -Method Get -Uri $modelsUri -TimeoutSec 5)
        Write-Host "Model endpoint reachable: $modelsUri" -ForegroundColor Green
        Write-Event -Stage 'model-endpoint' -Status 'pass' -Detail $modelsUri
    }
    catch {
        Write-Host "Local model endpoint is not reachable at $modelsUri. Start LM Studio's local server and load the configured embedding model before semantic indexing." -ForegroundColor Yellow
        Write-Event -Stage 'model-endpoint' -Status 'warning' -Detail $_.Exception.Message
        return
    }

    try {
        $body = @{ model = $EmbeddingModel; input = 'WellForge RAG health check' } | ConvertTo-Json -Depth 4
        $response = Invoke-RestMethod -Method Post -Uri $embeddingsUri -ContentType 'application/json' -Body $body -TimeoutSec 15
        if (-not $response.data -or -not $response.data[0].embedding) { throw 'Embedding endpoint returned no embedding vector.' }
        $dimension = @($response.data[0].embedding).Count
        if ($dimension -ne $EmbeddingDimension) {
            throw "Embedding dimension mismatch. Configured $EmbeddingDimension, endpoint returned $dimension."
        }
        Write-Host "Embedding endpoint ready: $EmbeddingModel ($dimension dimensions)" -ForegroundColor Green
        Write-Event -Stage 'embeddings' -Status 'pass' -Detail "$EmbeddingModel/$dimension"
    }
    catch {
        Write-Host "Embedding check failed: $($_.Exception.Message)" -ForegroundColor Yellow
        Write-Host "Load '$EmbeddingModel' in LM Studio or pass the correct -EmbeddingModel/-EmbeddingDimension values." -ForegroundColor Yellow
        Write-Event -Stage 'embeddings' -Status 'warning' -Detail $_.Exception.Message
    }
}

function Invoke-RustVerification {
    Write-Step 'Building and verifying WellForgeRag'
    $cargo = Get-Executable 'cargo.exe'
    if (-not $cargo) { throw 'cargo.exe is unavailable after Rust initialization.' }

    if ($RunTests) {
        # Contract markers: cargo fmt / cargo clippy / cargo test
        Invoke-Native -FilePath $cargo -Arguments @('fmt', '--all', '--', '--check') -WorkingDirectory $script:RagRoot
        Invoke-Native -FilePath $cargo -Arguments @('clippy', '--workspace', '--all-targets', '--', '-D', 'warnings') -WorkingDirectory $script:RagRoot
        Invoke-Native -FilePath $cargo -Arguments @('test', '--workspace') -WorkingDirectory $script:RagRoot
        Write-Event -Stage 'rust-verification' -Status 'pass' -Detail 'fmt/clippy/tests'
    }

    $buildArgs = @('build', '--workspace')
    if ($Release) { $buildArgs += '--release' }
    Invoke-Native -FilePath $cargo -Arguments $buildArgs -WorkingDirectory $script:RagRoot
    Write-Event -Stage 'rust-build' -Status 'pass' -Detail ($buildArgs -join ' ')
}

function Resolve-Binary {
    param([Parameter(Mandatory)][string]$Name)
    $profile = if ($Release) { 'release' } else { 'debug' }
    $candidate = Join-Path $script:RagRoot "target\$profile\$Name.exe"
    if (Test-Path -LiteralPath $candidate) { return $candidate }
    return $null
}

function Invoke-EntryPoint {
    $cli = Resolve-Binary 'wellforge-rag'
    $server = Resolve-Binary 'wellforge-rag-server'
    $mcp = Resolve-Binary 'wellforge-rag-mcp'

    Write-Step 'Checking RAG entry points'
    $available = @()
    if ($cli) { $available += 'wellforge-rag' }
    if ($server) { $available += 'wellforge-rag-server' }
    if ($mcp) { $available += 'wellforge-rag-mcp' }
    Write-Host ('Available: ' + ($(if ($available.Count) { $available -join ', ' } else { 'none yet' })))

    if ($cli) {
        Invoke-Native -FilePath $cli -Arguments @('--config', $script:ConfigPath, 'doctor')
        Write-Event -Stage 'doctor' -Status 'pass' -Detail $cli
    }

    $mode = $Launch
    if ($mode -eq 'Auto') {
        if ($server) { $mode = 'Server' }
        elseif ($cli) { $mode = 'Cli' }
        else { $mode = 'None' }
    }

    switch ($mode) {
        'Server' {
            if (-not $server) { throw 'wellforge-rag-server has not been built on this branch yet.' }
            Start-Process -FilePath $server -ArgumentList @('--config', "`"$script:ConfigPath`"") -WorkingDirectory $script:RagRoot
            Write-Host 'WellForge RAG HTTP server launched.' -ForegroundColor Green
        }
        'Mcp' {
            if (-not $mcp) { throw 'wellforge-rag-mcp has not been built on this branch yet.' }
            & $mcp --config $script:ConfigPath
        }
        'Cli' {
            if (-not $cli) { throw 'wellforge-rag CLI has not been built on this branch yet.' }
            Write-Host "CLI ready: $cli" -ForegroundColor Green
        }
        'None' {
            if (-not $available.Count) {
                Write-Host 'Core RAG workspace built, but the CLI/server/MCP crates are not present on this branch yet. The script will automatically detect them after they are implemented.' -ForegroundColor Yellow
            }
        }
    }
}

try {
    Resolve-Roots
    Initialize-Logging
    Write-Event -Stage 'bootstrap' -Status 'start' -Detail $script:RagRoot

    Ensure-Prerequisites
    Sync-Repository
    Resolve-Roots
    Initialize-Rust
    Initialize-PythonSidecar
    Write-LocalConfig
    Test-LocalModelEndpoint
    Invoke-RustVerification
    Invoke-EntryPoint

    Write-Event -Stage 'bootstrap' -Status 'pass' -Detail 'completed'
    Write-Host "`nWellForgeRag local setup/verification completed." -ForegroundColor Green
    Write-Host "Config: $script:ConfigPath"
    Write-Host "Log:    $script:LogPath"
}
catch {
    Write-Event -Stage 'bootstrap' -Status 'fail' -Detail $_.Exception.Message
    Write-Error $_
    exit 1
}
