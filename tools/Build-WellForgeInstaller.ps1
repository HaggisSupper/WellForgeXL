[CmdletBinding()]
param(
    [string]$OutputPath
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repositoryRoot = Split-Path -Parent $PSScriptRoot
$ifBlankOutput = [string]::IsNullOrWhiteSpace($OutputPath)
if ($ifBlankOutput) { $OutputPath = Join-Path $repositoryRoot 'outputs\WellForgeXL-Setup.exe' }
$sourceDirectory = Join-Path $repositoryRoot 'outputs\vba-engine'
$stagingDirectory = Join-Path $repositoryRoot 'package\wellforge-installer-staging'
$workbookNames = @(
    'API_7G_Drill_String_Strength_and_Torque_SI.xlsm',
    'Steady_State_Hydraulics_and_Nozzle_Optimization_SI.xlsm',
    'Torque_Drag_and_Buckling_SI.xlsm',
    'BHA_Vibration_Bending_and_Drill_Ahead_Tendency_SI.xlsm',
    'Directional_Drilling_Wellplan_and_Survey_SI.xlsm'
)
$runtimeNames = @(
    'wellforge-bha.exe', 'wellforge-bha.exe.sha256',
    'wellforge-trajectory.exe', 'wellforge-trajectory.exe.sha256',
    'wellforge-torque-drag.exe', 'wellforge-torque-drag.exe.sha256',
    'wellforge-hydraulics.exe', 'wellforge-hydraulics.exe.sha256'
)

if (-not (Get-Command iexpress.exe -ErrorAction SilentlyContinue)) {
    throw 'IExpress (iexpress.exe) is required to create the self-extracting installer.'
}
foreach ($name in @($workbookNames + $runtimeNames)) {
    $path = Join-Path $sourceDirectory $name
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "Required build artifact is missing: $path"
    }
}

if (Test-Path -LiteralPath $stagingDirectory) {
    Remove-Item -LiteralPath $stagingDirectory -Recurse -Force
}
New-Item -ItemType Directory -Path $stagingDirectory -Force | Out-Null

foreach ($name in @($workbookNames + $runtimeNames)) {
    Copy-Item -LiteralPath (Join-Path $sourceDirectory $name) -Destination (Join-Path $stagingDirectory $name)
}
Copy-Item -LiteralPath (Join-Path $repositoryRoot 'README.md') -Destination (Join-Path $stagingDirectory 'README.md')
Copy-Item -LiteralPath (Join-Path $repositoryRoot 'LICENSE') -Destination (Join-Path $stagingDirectory 'LICENSE')

$installScript = @'
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = [Security.Principal.WindowsPrincipal]::new($identity)
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    $args = '-NoProfile -ExecutionPolicy Bypass -File "{0}"' -f $PSCommandPath
    Start-Process powershell.exe -Verb RunAs -ArgumentList $args -Wait
    exit $LASTEXITCODE
}
$target = Join-Path ${env:ProgramFiles} 'WellForgeXL'
New-Item -ItemType Directory -Path $target -Force | Out-Null
$skip = @('install.cmd', 'install.ps1')
Get-ChildItem -LiteralPath $PSScriptRoot -File | Where-Object { $skip -notcontains $_.Name } | ForEach-Object {
    Copy-Item -LiteralPath $_.FullName -Destination (Join-Path $target $_.Name) -Force
}
$startMenu = Join-Path ${env:ProgramData} 'Microsoft\Windows\Start Menu\Programs\WellForgeXL'
New-Item -ItemType Directory -Path $startMenu -Force | Out-Null
$shell = New-Object -ComObject WScript.Shell
Get-ChildItem -LiteralPath $target -Filter '*.xlsm' | ForEach-Object {
    $shortcut = $shell.CreateShortcut((Join-Path $startMenu ($_.BaseName + '.lnk')))
    $shortcut.TargetPath = $_.FullName
    $shortcut.WorkingDirectory = $target
    $shortcut.Save()
}
[System.Runtime.InteropServices.Marshal]::FinalReleaseComObject($shell)
Write-Output "Installed WellForgeXL to $target"
'@
Set-Content -LiteralPath (Join-Path $stagingDirectory 'install.ps1') -Value $installScript -Encoding UTF8
$installCmd = '@echo off' + [Environment]::NewLine + 'powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File "%~dp0install.ps1"' + [Environment]::NewLine
Set-Content -LiteralPath (Join-Path $stagingDirectory 'install.cmd') -Value $installCmd -Encoding ASCII

$files = Get-ChildItem -LiteralPath $stagingDirectory -File | Sort-Object Name
$stringLines = [System.Collections.Generic.List[string]]::new()
$sourceLines = [System.Collections.Generic.List[string]]::new()
for ($index = 0; $index -lt $files.Count; $index++) {
    $key = "FILE$index"
    $stringLines.Add(('{0}="{1}"' -f $key, $files[$index].Name))
    $sourceLines.Add("%$key%=")
}

$outputDirectory = Split-Path -Parent $OutputPath
New-Item -ItemType Directory -Path $outputDirectory -Force | Out-Null
$sedPath = Join-Path $stagingDirectory 'WellForgeXL-Setup.sed'
$sed = @(
    '[Version]',
    'Class=IEXPRESS',
    'SEDVersion=3',
    '[Options]',
    'PackagePurpose=InstallApp',
    'ShowInstallProgramWindow=1',
    'HideExtractAnimation=1',
    'UseLongFileName=1',
    'InsideCompressed=1',
    'CAB_FixedSize=0',
    'CAB_ResvCodeSigning=0',
    'RebootMode=N',
    'InstallPrompt=',
    'DisplayLicense=',
    'FinishMessage=WellForgeXL installation completed.',
    ('TargetName={0}' -f $OutputPath),
    'FriendlyName=WellForgeXL',
    'AppLaunched=install.cmd',
    'PostInstallCmd=<None>',
    'AdminQuietInstCmd=install.cmd',
    'UserQuietInstCmd=install.cmd',
    'SourceFiles=SourceFiles',
    '',
    '[SourceFiles]',
    ('SourceFiles0={0}' -f $stagingDirectory),
    '',
    '[SourceFiles0]'
) + $sourceLines + @('', '[Strings]') + $stringLines
Set-Content -LiteralPath $sedPath -Value $sed -Encoding ASCII

$iexpress = Start-Process -FilePath 'iexpress.exe' -ArgumentList @('/N', '/Q', $sedPath) -Wait -PassThru
if ($iexpress.ExitCode -ne 0 -or -not (Test-Path -LiteralPath $OutputPath -PathType Leaf)) {
    throw "IExpress failed to create the installer at $OutputPath"
}
$hash = (Get-FileHash -LiteralPath $OutputPath -Algorithm SHA256).Hash.ToLowerInvariant()
Set-Content -LiteralPath ($OutputPath + '.sha256') -Value ("{0} *{1}" -f $hash, (Split-Path -Leaf $OutputPath)) -Encoding ASCII
Write-Output "Created installer: $OutputPath"
