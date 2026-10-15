# Windows PowerShell 5.1+ and PowerShell 7+. No administrator access required.
[CmdletBinding()]
param(
    [string]$Version = $(if ($env:VERSION) { $env:VERSION } else { 'latest' }),
    [string]$Destination = $(if ($env:DEST) { $env:DEST } else { Join-Path $env:LOCALAPPDATA 'muhan\bin' }),
    [string]$Repository = $(if ($env:REPO) { $env:REPO } else { 'muhan-alan-li/muhancli' }),
    [switch]$NoModifyPath
)

# Keep preference changes local even when the installer is invoked with iex.
& {
    $ErrorActionPreference = 'Stop'
    $ProgressPreference = 'SilentlyContinue'
    if ([Environment]::OSVersion.Platform -ne [PlatformID]::Win32NT) {
        throw 'This installer is for Windows. On Linux or macOS, use install.sh.'
    }
    if (-not [IO.Path]::IsPathRooted($Destination)) {
        throw 'Destination must be an absolute path.'
    }
    $Destination = [IO.Path]::GetFullPath($Destination)
    $nativeArch = $env:PROCESSOR_ARCHITEW6432
    if (-not $nativeArch) { $nativeArch = $env:PROCESSOR_ARCHITECTURE }
    $arch = switch ($nativeArch) {
        'AMD64' { 'x86_64' }
        'ARM64' { 'aarch64' }
        default { throw "Unsupported architecture: $nativeArch. Use 64-bit Windows on x64 or ARM64." }
    }

    # Windows PowerShell 5.1 may otherwise negotiate an older TLS version.
    $previousTls = [Net.ServicePointManager]::SecurityProtocol
    [Net.ServicePointManager]::SecurityProtocol = $previousTls -bor [Net.SecurityProtocolType]::Tls12
    $tempDir = Join-Path ([IO.Path]::GetTempPath()) ('muhan-' + [Guid]::NewGuid().ToString('N'))
    $staged = $null
    try {
        $null = New-Item -ItemType Directory -Path $tempDir
        if ($Version -eq 'latest') {
            try {
                $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repository/releases/latest" -Headers @{ 'User-Agent' = 'muhan-installer' }
                $Version = $release.tag_name
            } catch {
                throw 'Could not find the latest release. Check the repository releases page or pass -Version vX.Y.Z.'
            }
        }
        if ($Version -notmatch '^v[0-9][A-Za-z0-9._-]*$') {
            throw 'Version must be a release tag such as v0.1.0.'
        }
        $asset = "muhan-windows-$arch.zip"
        $url = "https://github.com/$Repository/releases/download/$Version/$asset"
        $archive = Join-Path $tempDir $asset
        Write-Host "Installing muhan $Version for Windows/$arch..."
        Invoke-WebRequest -UseBasicParsing -Uri $url -OutFile $archive
        $checksumPath = Join-Path $tempDir 'expected.sha256'
        Invoke-WebRequest -UseBasicParsing -Uri "$url.sha256" -OutFile $checksumPath
        $expected = ((Get-Content -LiteralPath $checksumPath -Raw).Trim() -split '\s+')[0]
        if ($expected -notmatch '^[0-9a-fA-F]{64}$') { throw 'Invalid SHA-256 checksum file.' }
        $actual = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash
        if ($expected -ne $actual) { throw 'Checksum mismatch. Nothing was installed.' }

        # Extract only the executable, never arbitrary archive paths.
        Add-Type -AssemblyName System.IO.Compression.FileSystem
        $zip = [IO.Compression.ZipFile]::OpenRead($archive)
        $binary = Join-Path $tempDir 'muhan.exe'
        try {
            $entry = $zip.GetEntry('muhan.exe')
            if ($null -eq $entry) { throw 'Release archive is missing muhan.exe.' }
            [IO.Compression.ZipFileExtensions]::ExtractToFile($entry, $binary)
        } finally { $zip.Dispose() }
        & $binary --version
        if ($LASTEXITCODE -ne 0) { throw 'The downloaded binary cannot run on this system.' }
        $null = New-Item -ItemType Directory -Path $Destination -Force
        $staged = Join-Path $Destination ('.muhan-' + [Guid]::NewGuid().ToString('N') + '.exe')
        Copy-Item -LiteralPath $binary -Destination $staged
        Move-Item -LiteralPath $staged -Destination (Join-Path $Destination 'muhan.exe') -Force
        $staged = $null

        if (-not $NoModifyPath -and $env:NO_MODIFY_PATH -ne '1') {
            $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
            $userEntries = @($userPath -split ';' | Where-Object { $_ })
            if ($userEntries -notcontains $Destination) {
                $newUserPath = (@($userEntries) + $Destination) -join ';'
                [Environment]::SetEnvironmentVariable('Path', $newUserPath, 'User')
            }
            if (($env:Path -split ';') -notcontains $Destination) {
                $env:Path = "$Destination;$env:Path"
            }
            Write-Host 'Added muhan to PATH for this PowerShell session and future terminals.'
        }
        Write-Host "Installed to $(Join-Path $Destination 'muhan.exe')"
        Write-Host 'Explore with: muhan about'
        Write-Host 'More commands: muhan --help'
    } finally {
        [Net.ServicePointManager]::SecurityProtocol = $previousTls
        if ($staged -and (Test-Path -LiteralPath $staged)) { Remove-Item -LiteralPath $staged -Force }
        if (Test-Path -LiteralPath $tempDir) { Remove-Item -LiteralPath $tempDir -Recurse -Force }
    }
}
