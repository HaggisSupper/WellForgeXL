import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtemp, readFile, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import JSZip from 'jszip';

const root = fileURLToPath(new URL('..', import.meta.url));
const read = (name) => readFile(path.join(root, name), 'utf8');
const quote = (value) => `'${value.replaceAll("'", "''")}'`;
const windowsOnly = { skip: process.platform !== 'win32' };
function powershell(body) {
  const script = `$ErrorActionPreference = 'Stop'; $ProgressPreference = 'SilentlyContinue'; Set-StrictMode -Version Latest
    Set-Location ${quote(root)}
    ${body}`;
  return spawnSync('powershell.exe', ['-NoProfile', '-NonInteractive', '-EncodedCommand', Buffer.from(script, 'utf16le').toString('base64')],
    { cwd: root, encoding: 'utf8', timeout: 60000, windowsHide: true });
}
function pass(body) {
  const result = powershell(body);
  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
  return result.stdout.trim();
}
const guards = ". ./tools/WellForgeBuildGuards.ps1";
const expectThrow = `function Expect-Throw([scriptblock]$Action) {
  $failed = $false; try { & $Action } catch { $failed = $true }
  if (-not $failed) { throw 'Expected guard rejection' }
}`;
// Load only function ASTs: never execute the builder, its engines, or Excel.
const builderFunctions = `
  $tokens = $null; $errors = $null
  $ast = [System.Management.Automation.Language.Parser]::ParseFile((Join-Path $PWD 'tools/Build-WellForgeVbaSuite.ps1'), [ref]$tokens, [ref]$errors)
  if ($errors.Count) { throw 'Builder parse failed' }
  foreach ($function in $ast.FindAll({ param($n) $n -is [System.Management.Automation.Language.FunctionDefinitionAst] }, $false)) {
    . ([scriptblock]::Create($function.Extent.Text))
  }
  Add-Type -AssemblyName System.IO.Compression.FileSystem
  if (Test-Path ./tools/WellForgeWorkbookPackage.ps1) { . ./tools/WellForgeWorkbookPackage.ps1 }
`;

test('builder normalizes output from a different caller CWD and rejects unsafe workbook basenames', windowsOnly, () => {
  pass(String.raw`${guards}
    ${expectThrow}
    Set-Location $env:TEMP
    $resolved = Resolve-WellForgeOutputDirectory -RepositoryRoot 'C:/bounded/repo' -OutputDirectory './outputs/../package'
    if ($resolved -ne 'C:\bounded\repo\package') { throw "Wrong output: $resolved" }
    if ((Resolve-WellForgeOutputDirectory -RepositoryRoot 'C:/bounded/repo' -OutputDirectory 'C:/bounded/run/package') -ne 'C:\bounded\run\package') { throw 'Absolute output changed' }
    Assert-WellForgeWorkbookNames @('Valid workbook.xlsx', 'VALID.XLSX')
    foreach ($name in @('', ' ', '../bad.xlsx', '..\bad.xlsx', 'dir/bad.xlsx', 'dir\bad.xlsx', '/bad.xlsx', 'C:\bad.xlsx', '\\server\bad.xlsx', 'C:bad.xlsx', 'bad.xlsm', 'bad.xlsx.exe', 'bad.xlsx ', 'bad?.xlsx')) {
      Expect-Throw { Assert-WellForgeWorkbookNames @($name) }
    }
  `);
});

test('recursive cleanup guard rejects root, sibling, traversal and reparse targets', windowsOnly, () => {
  pass(String.raw`${guards}
    ${expectThrow}
    Assert-WellForgeCleanupTarget -Path 'C:/bounded/run/staging' -RunDirectory 'C:/bounded/run'
    foreach ($target in @('C:/bounded/run', 'C:/bounded/run/../user', 'C:/bounded/run-other/staging', 'C:/')) {
      Expect-Throw { Assert-WellForgeCleanupTarget -Path $target -RunDirectory 'C:/bounded/run' }
    }
    # A filesystem boundary double supplies only the reparse attribute; the real guard decides.
    function Get-Item { [pscustomobject]@{ Attributes = [IO.FileAttributes]::ReparsePoint } }
    function Test-Path { $true }
    Expect-Throw { Assert-WellForgeCleanupTarget -Path 'C:/bounded/run/link/staging' -RunDirectory 'C:/bounded/run' }
  `);
});

test('trajectory cleanup guard requires its exact run child and rejects traversal aliases', windowsOnly, () => {
  pass(String.raw`${guards}
    ${expectThrow}
    $parent = 'C:\bounded\WellForgeTrajectory'
    $child = 'release-test-0123456789abcdef0123456789abcdef'
    Assert-WellForgeCleanupTarget -Path "$parent\$child" -RunDirectory $parent -ExpectedChildName $child
    foreach ($target in @($parent, "$parent\sibling", "$parent-other\$child", "$parent\other\..\$child", "C:\")) {
      Expect-Throw { Assert-WellForgeCleanupTarget -Path $target -RunDirectory $parent -ExpectedChildName $child }
    }
  `);
});

test('trajectory helper setup and finally preserve unowned or redirected temporary material', windowsOnly, () => {
  pass(String.raw`${guards}
    $tokens = $null; $errors = $null
    $ast = [System.Management.Automation.Language.Parser]::ParseFile((Join-Path $PWD 'tools/Test-WellForgeTrajectoryEngine.ps1'), [ref]$tokens, [ref]$errors)
    if ($errors.Count) { throw 'Trajectory helper parse failed' }
    $nativeTry = $ast.EndBlock.Statements | Where-Object { $_ -is [System.Management.Automation.Language.TryStatementAst] } | Select-Object -First 1
    $setupStatements = @(); $inSetup = $false
    foreach ($statement in $nativeTry.Body.Statements) {
      if ($statement.Extent.Text -match '^(Assert-WellForgeCleanupTarget|New-Item|New-WellForgeExclusiveDirectory)\b') { $inSetup = $true }
      if ($statement.Extent.Text -match '^& \$executable validate\b') { break }
      if ($inSetup) { $setupStatements += $statement.Extent.Text }
    }
    if ($setupStatements.Count -eq 0) { throw 'Actual directory setup statements not found' }
    $setup = [scriptblock]::Create($setupStatements -join [Environment]::NewLine)
    $cleanup = [scriptblock]::Create($nativeTry.Finally.Extent.Text.Trim().Substring(1).TrimEnd().TrimEnd('}'))
    # Replace only filesystem boundaries. The actual helper setup/finally and
    # shared guard execute; no directory is created/deleted and no engine runs.
    function Test-Path { param($LiteralPath, $PathType) return $true }
    function Get-Item {
      param($LiteralPath, [switch]$Force)
      $attributes = [IO.FileAttributes]::Directory
      if ($LiteralPath -eq $script:redirectedPath) { $attributes = $attributes -bor [IO.FileAttributes]::ReparsePoint }
      [pscustomobject]@{ Attributes = $attributes }
    }
    function New-Item {
      param($ItemType, $Path, [switch]$Force)
      if ($Path -eq $temporaryParent) { $script:parentCreationCalls++; return }
      if ($script:failCreation) { throw 'Directory already exists: exclusive creation failed' }
      $script:creationCalls++
    }
    function New-WellForgeExclusiveDirectory {
      param($LiteralPath)
      if ($LiteralPath -ne "$temporaryParent\$runName") { throw 'Wrong native creation target' }
      if ($script:failCreation) { throw 'Directory already exists: exclusive creation failed' }
      $script:creationCalls++
    }
    function Remove-Item {
      param($LiteralPath, [switch]$Recurse, [switch]$Force)
      $script:deletionCalls++
      if ($LiteralPath -ne "$temporaryParent\$runName" -or -not $Recurse) { throw 'Wrong deletion target' }
    }
    foreach ($scenario in @('unowned-setup-failure', 'exclusive-creation-failure', 'parent-redirect', 'root', 'sibling', 'traversal', 'cleanup-parent-redirect', 'cleanup-child-redirect', 'cleanup-sibling', 'owned-success')) {
      $temporaryParent = 'C:\bounded\WellForgeTrajectory'
      $runName = 'release-test-0123456789abcdef0123456789abcdef'
      $runRoot = "$temporaryParent\$runName"
      $runDirectoryCreated = $false; $succeeded = $true; $NoPause = $true
      $script:creationCalls = 0; $script:parentCreationCalls = 0; $script:deletionCalls = 0; $script:redirectedPath = ''
      $script:failCreation = $scenario -eq 'exclusive-creation-failure'
      if ($scenario -eq 'parent-redirect') { $script:redirectedPath = $temporaryParent }
      if ($scenario -eq 'root') { $runRoot = $temporaryParent }
      if ($scenario -eq 'sibling') { $runRoot = "$temporaryParent\sibling" }
      if ($scenario -eq 'traversal') { $runRoot = "$temporaryParent\other\..\$runName" }
      $setupFailed = $false
      if ($scenario -ne 'unowned-setup-failure') {
        try { . $setup } catch { $setupFailed = $true }
      }
      if ($scenario -eq 'cleanup-parent-redirect') { $script:redirectedPath = $temporaryParent }
      if ($scenario -eq 'cleanup-child-redirect') { $script:redirectedPath = $runRoot }
      if ($scenario -eq 'cleanup-sibling') { $runRoot = "$temporaryParent\sibling" }
      . $cleanup
      if ($scenario -eq 'owned-success') {
        if ($setupFailed -or -not $runDirectoryCreated -or $script:creationCalls -ne 1 -or $script:deletionCalls -ne 1 -or -not $succeeded) { throw 'Owned run did not complete its guarded cleanup' }
      } else {
        if ($script:deletionCalls -ne 0) { throw "Unsafe deletion selected after $scenario" }
        if ($scenario.StartsWith('cleanup-') -and $succeeded) { throw "Cleanup failure was accepted: $scenario" }
        if (-not $scenario.StartsWith('cleanup-') -and $scenario -ne 'unowned-setup-failure' -and -not $setupFailed) { throw "Unsafe setup was accepted: $scenario" }
        if (-not $scenario.StartsWith('cleanup-') -and ($script:creationCalls -ne 0 -or $runDirectoryCreated)) { throw "Unsuccessful setup claimed ownership: $scenario" }
        if ($scenario -in @('parent-redirect', 'root', 'sibling', 'traversal') -and $script:parentCreationCalls -ne 0) { throw "Guard ran after filesystem mutation: $scenario" }
      }
    }
  `);
});

test('trajectory release helper wires exclusive creation and ownership-gated literal cleanup', async () => {
  const [helper, shared, release] = await Promise.all([
    read('tools/Test-WellForgeTrajectoryEngine.ps1'), read('tools/WellForgeBuildGuards.ps1'), read('tools/Invoke-WellForgeWindowsRelease.ps1'),
  ]);
  assert.match(release, /Test-WellForgeTrajectoryEngine\.ps1/);
  assert.match(helper, /WellForgeBuildGuards\.ps1/);
  assert.match(helper, /\$temporaryParent = .*GetFullPath/);
  assert.match(helper, /\$runDirectoryCreated = \$false/);
  assert.match(helper, /New-WellForgeExclusiveDirectory -LiteralPath \$runRoot\s+\$runDirectoryCreated = \$true/);
  assert.match(helper, /Assert-WellForgeCleanupTarget[^\n]*-ExpectedChildName \$runName/g);
  assert.match(helper, /Assert-WellForgeCleanupTarget[^\n]*\n\s*Remove-Item -LiteralPath \$runRoot -Recurse -Force/);
  assert.match(helper, /manual cleanup/i);
  assert.match(shared, /DllImport\("kernel32\.dll",[^\n]*SetLastError = true/);
  assert.match(shared, /if \(-not \[WellForgeDirectoryNative\]::CreateDirectory/);
});

test('Excel cleanup fails closed for live, unrelated, reused and unknown identities without terminating anything', windowsOnly, () => {
  pass(String.raw`${guards}
    ${expectThrow}
    function Stop-Process { throw 'Termination must never be called' }
    function taskkill.exe { throw 'Termination must never be called' }
    $identity = @{ process_id = 123; start_time_utc = '2026-09-07T01:00:00.0000000Z' }
    Assert-WellForgeExcelShutdown -Identity $identity -CurrentProcess $null
    foreach ($candidate in @(
      [pscustomobject]@{ Id = 123; StartTime = [datetime]'2026-09-07T01:00:00Z' },
      [pscustomobject]@{ Id = 456; StartTime = [datetime]'2026-09-07T02:00:00Z' },
      [pscustomobject]@{ Id = 123; StartTime = [datetime]'2026-09-07T03:00:00Z' }
    )) { Expect-Throw { Assert-WellForgeExcelShutdown -Identity $identity -CurrentProcess $candidate } }
    Expect-Throw { Assert-WellForgeExcelShutdown -Identity $null -CurrentProcess $null }
    Expect-Throw { Assert-WellForgeExcelShutdown -Identity @{ process_id = 123 } -CurrentProcess $null }
  `);
});

test('BHA end-to-end diagnostic paths remain outside the exact installable payload', windowsOnly, () => {
  pass(String.raw`${guards}
    ${expectThrow}
    $paths = Get-WellForgeBhaDiagnosticPaths -OutputDirectory 'C:/bounded/run/package' -LogDirectory 'C:/bounded/run/logs'
    if ($paths.result -ne 'C:\bounded\run\logs\bha-e2e-result.json' -or $paths.bridge -ne 'C:\bounded\run\logs\bha-e2e-result.wfbridge') { throw 'Diagnostics leaked into payload' }
    Expect-Throw { Get-WellForgeBhaDiagnosticPaths -OutputDirectory 'C:/bounded/run/package' -LogDirectory 'C:/bounded/run/package' }
    Expect-Throw { Get-WellForgeBhaDiagnosticPaths -OutputDirectory 'C:/bounded/run/package' -LogDirectory 'C:/bounded/run/package/logs' }
  `);
});

test('native policy gate requires observed version and successful frozen licenses/bans/sources command', windowsOnly, () => {
  pass(String.raw`${guards}
    ${expectThrow}
    $script:policyCalls = @()
    # External-tool boundary only: exercise real version/exit validation and command selection.
    function cargo-deny {
      $script:policyCalls += ,@($args)
      $global:LASTEXITCODE = 0
      if ($args[0] -eq '--version') { 'cargo-deny 0.20.2' } else { 'licenses ok, bans ok, sources ok' }
    }
    $log = Join-Path $env:TEMP ('wellforge-policy-test-' + [guid]::NewGuid() + '.log')
    $evidence = Invoke-WellForgeDependencyPolicy -EngineRoot (Join-Path $PWD 'engine') -LogPath $log
    if ($evidence.version -ne 'cargo-deny 0.20.2' -or $evidence.exit_code -ne 0) { throw 'Missing observed evidence' }
    if (($script:policyCalls[1] -join ' ') -ne '--frozen check licenses bans sources') { throw 'Policy restrictions changed' }
    function cargo-deny { $global:LASTEXITCODE = 0; 'cargo-deny 0.19.0' }
    Expect-Throw { Invoke-WellForgeDependencyPolicy -EngineRoot (Join-Path $PWD 'engine') -LogPath $log }
    function cargo-deny { $global:LASTEXITCODE = 7; 'cargo-deny 0.20.2' }
    Expect-Throw { Invoke-WellForgeDependencyPolicy -EngineRoot (Join-Path $PWD 'engine') -LogPath $log }
    function cargo-deny { if ($args[0] -eq '--version') { $global:LASTEXITCODE = 0; 'cargo-deny 0.20.2' } else { $global:LASTEXITCODE = 9 } }
    Expect-Throw { Invoke-WellForgeDependencyPolicy -EngineRoot (Join-Path $PWD 'engine') -LogPath $log }
    function Get-Command { $null }
    Expect-Throw { Invoke-WellForgeDependencyPolicy -EngineRoot (Join-Path $PWD 'engine') -LogPath $log }
  `);
});

test('Excel quit refuses an unexpected user workbook and never closes it', windowsOnly, () => {
  pass(`${guards}
    ${expectThrow}
    $application = [pscustomobject]@{ Workbooks = [pscustomobject]@{ Count = 1 }; QuitCalls = 0 }
    $application | Add-Member ScriptMethod Quit { $this.QuitCalls++ }
    Invoke-WellForgeExcelQuit -Application $null
    Expect-Throw { Invoke-WellForgeExcelQuit -Application $application }
    if ($application.QuitCalls -ne 0 -or $application.Workbooks.Count -ne 1) { throw 'User workbook was touched' }
    $application.Workbooks.Count = 0
    Invoke-WellForgeExcelQuit -Application $application
    if ($application.QuitCalls -ne 1) { throw 'Empty owned application was not quit' }
  `);
});

test('installer includes both release licenses, all four runtimes and every runtime sidecar', async () => {
  const source = await read('tools/Build-WellForgeInstaller.ps1');
  for (const name of ['LICENSE', 'LICENSE-APACHE']) assert.match(source, new RegExp(`Copy-Item[^\\n]*'${name}'[^\\n]*'${name}'`));
  for (const engine of ['bha', 'trajectory', 'torque-drag', 'hydraulics']) {
    for (const suffix of ['.exe', '.exe.sha256']) assert.ok(source.includes(`'wellforge-${engine}${suffix}'`));
  }
});

test('native entrypoints wire guards before sources, isolate diagnostics and retain actual five-dispatcher gates', async () => {
  const [builder, runner, watchdog] = await Promise.all([
    read('tools/Build-WellForgeVbaSuite.ps1'), read('tools/Invoke-WellForgeWindowsRelease.ps1'), read('tools/Invoke-WellForgeWindowsReleaseBounded.ps1'),
  ]);
  const nameGuard = builder.indexOf('Assert-WellForgeWorkbookNames -WorkbookNames $WorkbookNames');
  assert.ok(nameGuard >= 0 && nameGuard < builder.indexOf('foreach ($name in $WorkbookNames)'));
  assert.match(builder, /Resolve-WellForgeOutputDirectory/);
  assert.match(builder, /Get-WellForgeBhaDiagnosticPaths/);
  assert.doesNotMatch(builder, /Join-Path \$OutputDirectory 'bha-e2e-result/);
  assert.match(builder, /Assert-WellForgeCleanupTarget/);
  assert.match(builder, /Invoke-WellForgeExcelQuit/);
  assert.doesNotMatch(builder, /\$excel\.Quit\(/);
  assert.match(runner, /Invoke-WellForgeDependencyPolicy/);
  assert.match(runner, /cargo_deny = \$dependencyPolicy/);
  assert.match(runner, /Assert-WellForgeExcelShutdown/);
  assert.match(runner, /Invoke-WellForgeExcelQuit/);
  assert.doesNotMatch(runner, /\$excel\.Quit\(/);
  assert.match(runner, /Assert-XlsxPackageIntegrity/);
  assert.match(watchdog, /-WindowStyle Hidden/);
  for (const source of [runner, watchdog]) assert.doesNotMatch(source, /Stop-Process|taskkill|Stop-NewExcelProcesses|baselineExcelProcessIds/);
  assert.match(watchdog, /manual.cleanup/i);
  assert.match(runner, /foreach \(\$name in \$workbookNames\)[\s\S]*Invoke-WorkbookMacro -Workbook \$workbook -Macro 'WellForge_BuildInitialize'/);
  assert.match(runner, /status --porcelain=v1 --untracked-files=all/);
  assert.equal((runner.match(/'[^'\r\n]+\.xlsm'/g) ?? []).length, 5);
});

test('watchdog timeout and nonzero-exit branches preserve Excel and invalidate stale acceptance', windowsOnly, async () => {
  const fixtureRoot = await mkdtemp(path.join(os.tmpdir(), 'wellforge-watchdog-contract-'));
  for (const scenario of ['timeout', 'nonzero']) {
    const gatePath = path.join(fixtureRoot, `${scenario}.json`);
    const helperMarker = path.join(fixtureRoot, `${scenario}-helper-only.txt`);
    await writeFile(gatePath, JSON.stringify({ gates: { package_acceptance: { status: 'passed' } } }));
    const result = powershell(`
      $tokens = $null; $errors = $null
      $ast = [System.Management.Automation.Language.Parser]::ParseFile((Join-Path $PWD 'tools/Invoke-WellForgeWindowsReleaseBounded.ps1'), [ref]$tokens, [ref]$errors)
      if ($errors.Count) { throw 'Watchdog parse failed' }
      $definition = $ast.Find({ param($n) $n -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $n.Name -eq 'Save-BlockedCleanup' }, $false)
      if ($null -eq $definition) { throw 'Missing safe failure reporter' }
      . ([scriptblock]::Create($definition.Extent.Text))
      $gateResultsPath = ${quote(gatePath)}
      $helperMarker = ${quote(helperMarker)}
      $RunId = 'test-only'; $ExpectedGitSha = '0123456789abcdef0123456789abcdef01234567'
      $TimeoutMinutes = 1; $finished = $false
      function Stop-Process { throw 'Unexpected process termination' }
      function taskkill.exe { throw 'Unexpected process termination' }
      function Get-Process { throw 'Excel enumeration must not occur' }
      $process = [pscustomobject]@{ ExitCode = 7 }
      $process | Add-Member ScriptMethod Kill { [IO.File]::WriteAllText($helperMarker, 'captured-helper-only') }
      $process | Add-Member ScriptMethod WaitForExit { param($milliseconds) if ($milliseconds -ne 5000) { throw 'Unbounded wait' }; return $true }
      # Execute only the real failure branch AST, with an inert captured helper.
      # No release runner, engine, Excel, or actual process termination is invoked.
      $condition = ${quote(scenario === 'timeout' ? '-not $finished' : '$process.ExitCode -ne 0')}
      $branch = $ast.Find({ param($n) $n -is [System.Management.Automation.Language.IfStatementAst] -and $n.Clauses[0].Item1.Extent.Text -eq $condition }, $false)
      if ($null -eq $branch) { throw 'Failure branch not found' }
      . ([scriptblock]::Create($branch.Extent.Text))
    `);
    assert.equal(result.status, scenario === 'timeout' ? 124 : 7, result.stderr);
    const gate = JSON.parse(await readFile(gatePath, 'utf8'));
    assert.equal(gate.gates.package_acceptance.status, 'failed');
    assert.equal(gate.cleanup, 'blocked_manual_cleanup');
    if (scenario === 'timeout') assert.equal(await readFile(helperMarker, 'utf8'), 'captured-helper-only');
  }
});

const bhaName = 'BHA_Vibration_Bending_and_Drill_Ahead_Tendency_SI.xlsx';
test('retained BHA OOXML part inventory and separate chart layers are portable source contracts', async () => {
  const zip = await JSZip.loadAsync(await readFile(path.join(root, 'workbooks/source', bhaName)));
  const manifest = await zip.file('[Content_Types].xml').async('string');
  for (const [, part] of manifest.matchAll(/PartName="\/([^"]+)"/g)) assert.ok(zip.file(part), `Missing declared part: ${part}`);
  const radar = await zip.file('xl/drawings/charts/chart12.xml').async('string');
  const scatter = await zip.file('xl/drawings/charts/chart16.xml').async('string');
  assert.match(radar, /<c:radarChart>/);
  assert.doesNotMatch(radar, /<c:scatterChart>/);
  assert.match(scatter, /<c:scatterChart>/);
  assert.doesNotMatch(scatter, /<c:radarChart>/);
  for (const ring of ['25', '50', '75', '100']) assert.ok(radar.includes(`Ring ${ring}%`));
  for (const wob of ['WOB 1', 'WOB 2']) assert.ok(scatter.includes(wob));
  const drawing = await zip.file('xl/drawings/drawing4.xml').async('string');
  const anchors = [...drawing.matchAll(/<xdr:from>([\s\S]*?)<\/xdr:from><xdr:to>([\s\S]*?)<\/xdr:to>/g)];
  assert.equal(anchors.length, 2);
  assert.equal(anchors[0][0], anchors[1][0], 'radar grid and WOB traces must occupy the same display bounds');
});
test('retained BHA template preserves radar and XY layers with valid chart relationships', windowsOnly, () => {
  pass(`${builderFunctions}
    Assert-XlsxPackageIntegrity -Path ${quote(path.join(root, 'workbooks/source', bhaName))} -RequireBhaLayers
  `);
});

test('native package validation rejects missing declarations, relationships, WOB traces and displaced layers', windowsOnly, async () => {
  const bytes = await readFile(path.join(root, 'workbooks/source', bhaName));
  const fixtureRoot = await mkdtemp(path.join(os.tmpdir(), 'wellforge-bha-contract-'));
  const mutations = [
    ['missing-part', (z) => z.remove('xl/drawings/charts/chart12.xml')],
    ['dangling-relation', async (z) => z.file('xl/drawings/_rels/drawing4.xml.rels', (await z.file('xl/drawings/_rels/drawing4.xml.rels').async('string')).replace('chart16.xml', 'missing.xml'))],
    ['missing-trace', async (z) => z.file('xl/drawings/charts/chart16.xml', (await z.file('xl/drawings/charts/chart16.xml').async('string')).replace(/<c:ser>[\s\S]*?<\/c:ser>/, ''))],
    ['wrong-ring-data', async (z) => z.file('xl/drawings/charts/chart12.xml', (await z.file('xl/drawings/charts/chart12.xml').async('string')).replaceAll('$I$6:$I$17', '$B$6:$B$17'))],
    ['wrong-wob-data', async (z) => z.file('xl/drawings/charts/chart16.xml', (await z.file('xl/drawings/charts/chart16.xml').async('string')).replaceAll('$B$6:$B$18', '$D$6:$D$18'))],
    ['combined-layers', async (z) => {
      const radar = await z.file('xl/drawings/charts/chart12.xml').async('string');
      const scatter = await z.file('xl/drawings/charts/chart16.xml').async('string');
      z.file('xl/drawings/charts/chart12.xml', radar.replace('</c:plotArea>', `${scatter.match(/<c:scatterChart>[\s\S]*?<\/c:scatterChart>/)[0]}</c:plotArea>`));
    }],
    ['displaced-overlay', async (z) => z.file('xl/drawings/drawing4.xml', (await z.file('xl/drawings/drawing4.xml').async('string')).replace(/(<xdr:from>[\s\S]*?<xdr:col>)\d+/, '$199'))],
    ['invisible-trace', async (z) => z.file('xl/drawings/charts/chart16.xml', (await z.file('xl/drawings/charts/chart16.xml').async('string')).replaceAll('65000', '0'))],
  ];
  const checks = [];
  for (const [label, mutate] of mutations) {
    const zip = await JSZip.loadAsync(bytes);
    await mutate(zip);
    const fixture = path.join(fixtureRoot, `${label}.xlsx`);
    await writeFile(fixture, await zip.generateAsync({ type: 'nodebuffer' }));
    checks.push(`$rejected = $false
      try { Assert-XlsxPackageIntegrity -Path ${quote(fixture)} @bhaOptions }
      catch { if ($_.Exception.Message -notmatch 'OOXML|BHA') { throw }; $rejected = $true }
      if (-not $rejected) { throw 'Accepted ${label}' }`);
  }
  pass(`${builderFunctions}
    $bhaOptions = @{}
    if ((Get-Command Assert-XlsxPackageIntegrity).Parameters.ContainsKey('RequireBhaLayers')) { $bhaOptions.RequireBhaLayers = $true }
    ${checks.join('\n')}`);
});
