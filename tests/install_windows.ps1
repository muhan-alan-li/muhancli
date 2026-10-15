# Run on Windows with either Windows PowerShell 5.1 or PowerShell 7.
# Downloads are mocked; the archive and executable come from the real CI build.
$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path $PSScriptRoot -Parent
$testRoot = Join-Path ([IO.Path]::GetTempPath()) ('muhan-test-' + [Guid]::NewGuid().ToString('N'))
$destination = Join-Path $testRoot 'install with spaces'
$originalPath = $env:Path
$originalUserPath = [Environment]::GetEnvironmentVariable('Path', 'User')
$originalNoModify = $env:NO_MODIFY_PATH
$env:NO_MODIFY_PATH = '0'
$mockState = @{
    MissingChecksum = $false
    BadChecksum = $false
    Requests = [Collections.Generic.List[string]]::new()
}
$nativeArch = $env:PROCESSOR_ARCHITEW6432
if (-not $nativeArch) { $nativeArch = $env:PROCESSOR_ARCHITECTURE }
$arch = if ($nativeArch -eq 'ARM64') { 'aarch64' } else { 'x86_64' }
$mockState.Fixture = Join-Path $repoRoot "dist/muhan-windows-$arch.zip"

# Capture shared state: $script: inside a mock would refer to install.ps1's scope.
Set-Item -Path Function:Invoke-RestMethod -Value ({
    param($Uri, $Headers)
    $mockState.Requests.Add($Uri)
    return @{ tag_name = 'v0.1.0' }
}.GetNewClosure())

Set-Item -Path Function:Invoke-WebRequest -Value ({
    param([switch]$UseBasicParsing, $Uri, $OutFile)
    $mockState.Requests.Add($Uri)
    if ($Uri.EndsWith('.sha256')) {
        if ($mockState.MissingChecksum) { throw 'Simulated missing checksum' }
        if ($mockState.BadChecksum) {
            Set-Content -LiteralPath $OutFile -Value (('0' * 64) + '  fixture.zip') -Encoding ASCII
        } else {
            Copy-Item -LiteralPath "$($mockState.Fixture).sha256" -Destination $OutFile
        }
    } else {
        Copy-Item -LiteralPath $mockState.Fixture -Destination $OutFile
    }
}.GetNewClosure())

function Assert($Condition, $Message) {
    if (-not $Condition) { throw $Message }
}

try {
    $null = New-Item -ItemType Directory -Path $testRoot
    # Parse the installer explicitly as well as executing it.
    $parseErrors = $null
    $tokens = $null
    $null = [Management.Automation.Language.Parser]::ParseFile(
        (Join-Path $repoRoot 'install.ps1'), [ref]$tokens, [ref]$parseErrors)
    Assert ($parseErrors.Count -eq 0) 'Installer must parse without errors'

    & "$repoRoot/install.ps1" -Destination $destination -Version latest
    $exe = Join-Path $destination 'muhan.exe'
    Assert (Test-Path -LiteralPath $exe) 'Installer must create muhan.exe'
    $about = & $exe --json about | ConvertFrom-Json
    Assert ($LASTEXITCODE -eq 0 -and $about.name) 'Installed CLI must run'
    Assert (($env:Path -split ';') -contains $destination) 'Current session PATH must include install directory'
    Assert (([Environment]::GetEnvironmentVariable('Path', 'User') -split ';') -contains $destination) 'User PATH must include install directory'
    # Exercise PowerShell's actual Tab expansion API with the generated adapter.
    & $exe completions powershell | Out-String | Invoke-Expression
    $null = New-Item -ItemType File -Path (Join-Path $testRoot 'ap-local-decoy')
    $null = New-Item -ItemType File -Path (Join-Path $testRoot 'unrelated-local-decoy')
    Push-Location $testRoot
    try {
        foreach ($case in @(
            @{ Line = 'muhan '; Expected = @('about', 'experience', 'projects', '--json') },
            @{ Line = 'muhan exp'; Expected = @('experience') },
            @{ Line = 'muhan experience '; Expected = @('apera-ai', 'nrcan', 'ubc-cic') },
            @{ Line = 'muhan experience ap'; Expected = @('apera-ai') },
            @{ Line = 'muhan projects dune'; Expected = @('dune-imperium') },
            @{ Line = 'muhan projects --tag Re'; Expected = @('React') },
            @{ Line = 'muhan skills --group Lang'; Expected = @('Languages') },
            @{ Line = 'muhan --plain experience nr'; Expected = @('nrcan') },
            @{ Line = 'muhan projects --j'; Expected = @('--json') }
        )) {
            $candidates = (TabExpansion2 $case.Line $case.Line.Length).CompletionMatches.CompletionText
            foreach ($expected in $case.Expected) {
                Assert ($candidates -contains $expected) "Expected '$expected' for: $($case.Line)"
            }
            Assert (-not ($candidates -match 'local-decoy')) "Completion must not suggest files for: $($case.Line)"
        }
    } finally {
        Pop-Location
    }
    Assert ($mockState.Requests -contains "https://github.com/muhan-alan-li/muhancli/releases/download/v0.1.0/muhan-windows-$arch.zip") 'Latest must resolve to a pinned release'

    $mockState.Requests.Clear()
    & "$repoRoot/install.ps1" -Destination $destination -Version v0.1.0
    $entries = @([Environment]::GetEnvironmentVariable('Path', 'User') -split ';' | Where-Object { $_ -eq $destination })
    Assert ($entries.Count -eq 1) 'Reinstall must not duplicate PATH entries'
    Assert ($mockState.Requests.Count -eq 2) 'Pinned version must skip latest lookup'

    $before = (Get-FileHash -LiteralPath $exe).Hash
    foreach ($failure in @('bad', 'missing')) {
        $mockState.BadChecksum = $failure -eq 'bad'
        $mockState.MissingChecksum = $failure -eq 'missing'
        $rejected = $false
        try { & "$repoRoot/install.ps1" -Destination $destination -Version v0.1.0 -NoModifyPath }
        catch { $rejected = $true }
        Assert $rejected 'Invalid or missing checksums must fail'
        Assert ((Get-FileHash -LiteralPath $exe).Hash -eq $before) 'Failed update must preserve existing executable'
    }
    $mockState.BadChecksum = $false
    $mockState.MissingChecksum = $false
    $otherDestination = Join-Path $testRoot 'no path modification'
    & "$repoRoot/install.ps1" -Destination $otherDestination -Version v0.1.0 -NoModifyPath
    Assert (($env:Path -split ';') -notcontains $otherDestination) 'NoModifyPath must preserve PATH'
    Write-Host 'Windows installer checks passed.'
} finally {
    # Restore the runner's environment even if a test fails.
    [Environment]::SetEnvironmentVariable('Path', $originalUserPath, 'User')
    $env:Path = $originalPath
    $env:NO_MODIFY_PATH = $originalNoModify
    if (Test-Path -LiteralPath $testRoot) { Remove-Item -LiteralPath $testRoot -Recurse -Force }
}
