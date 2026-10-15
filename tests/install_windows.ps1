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
$script:missingChecksum = $false
$script:badChecksum = $false
$script:requests = [Collections.Generic.List[string]]::new()
$nativeArch = $env:PROCESSOR_ARCHITEW6432
if (-not $nativeArch) { $nativeArch = $env:PROCESSOR_ARCHITECTURE }
$arch = if ($nativeArch -eq 'ARM64') { 'aarch64' } else { 'x86_64' }
$script:fixture = Join-Path $repoRoot "dist/muhan-windows-$arch.zip"

function Invoke-RestMethod {
    param($Uri, $Headers)
    $script:requests.Add($Uri)
    return @{ tag_name = 'v0.1.0' }
}

function Invoke-WebRequest {
    param([switch]$UseBasicParsing, $Uri, $OutFile)
    $script:requests.Add($Uri)
    if ($Uri.EndsWith('.sha256')) {
        if ($script:missingChecksum) { throw 'Simulated missing checksum' }
        if ($script:badChecksum) {
            Set-Content -LiteralPath $OutFile -Value (('0' * 64) + '  fixture.zip') -Encoding ASCII
        } else {
            Copy-Item -LiteralPath "${script:fixture}.sha256" -Destination $OutFile
        }
    } else {
        Copy-Item -LiteralPath $script:fixture -Destination $OutFile
    }
}

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
    Assert ($script:requests -contains "https://github.com/muhan-alan-li/muhancli/releases/download/v0.1.0/muhan-windows-$arch.zip") 'Latest must resolve to a pinned release'

    $script:requests.Clear()
    & "$repoRoot/install.ps1" -Destination $destination -Version v0.1.0
    $entries = @([Environment]::GetEnvironmentVariable('Path', 'User') -split ';' | Where-Object { $_ -eq $destination })
    Assert ($entries.Count -eq 1) 'Reinstall must not duplicate PATH entries'
    Assert ($script:requests.Count -eq 2) 'Pinned version must skip latest lookup'

    $before = (Get-FileHash -LiteralPath $exe).Hash
    foreach ($failure in @('bad', 'missing')) {
        $script:badChecksum = $failure -eq 'bad'
        $script:missingChecksum = $failure -eq 'missing'
        $rejected = $false
        try { & "$repoRoot/install.ps1" -Destination $destination -Version v0.1.0 -NoModifyPath }
        catch { $rejected = $true }
        Assert $rejected 'Invalid or missing checksums must fail'
        Assert ((Get-FileHash -LiteralPath $exe).Hash -eq $before) 'Failed update must preserve existing executable'
    }
    $script:badChecksum = $false
    $script:missingChecksum = $false
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
