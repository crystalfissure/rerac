#Requires -Version 5.1
# Standalone Windows entry point. Uses only packaged executables and paths selected
# by the player. The starter does not install software or change system settings.
$ErrorActionPreference = 'Stop'

function Select-DataFolder {
    param([string]$Description, [bool]$AllowNewFolder)
    $dialog = New-Object System.Windows.Forms.FolderBrowserDialog
    try {
        $dialog.Description = $Description
        $dialog.ShowNewFolderButton = $AllowNewFolder
        $defaultData = Join-Path $env:LOCALAPPDATA 'rerac\games\rac1\data'
        if (Test-Path -LiteralPath $defaultData -PathType Container) {
            $dialog.SelectedPath = $defaultData
        }
        if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
            return $dialog.SelectedPath
        }
        return $null
    } finally {
        $dialog.Dispose()
    }
}

try {
    Add-Type -AssemblyName System.Windows.Forms
    $runtime = Join-Path $PSScriptRoot 'rerac.exe'
    $extractor = Join-Path $PSScriptRoot 'rerac-extract.exe'
    foreach ($required in @($runtime, $extractor)) {
        if (-not (Test-Path -LiteralPath $required -PathType Leaf)) {
            throw 'Extract the complete mod ZIP into a folder before starting it.'
        }
    }
    Set-Location -LiteralPath $PSScriptRoot

    Write-Host 'ReRAC Skybox Mod - unofficial preview'
    Write-Host 'In a level: F6 picker, Page Up/Down choose, F7 apply, F8 original.'
    $choice = [System.Windows.Forms.MessageBox]::Show(
        "Do you already have extracted ReRAC game data?`n`nYes: choose your existing data folder.`nNo: extract your supported disc ISO into an empty folder.`nCancel: exit.",
        'ReRAC Skybox Mod',
        [System.Windows.Forms.MessageBoxButtons]::YesNoCancel,
        [System.Windows.Forms.MessageBoxIcon]::Question
    )
    if ($choice -eq [System.Windows.Forms.DialogResult]::Cancel) { exit 0 }

    if ($choice -eq [System.Windows.Forms.DialogResult]::Yes) {
        $dataPath = Select-DataFolder 'Choose the ReRAC data folder containing toc.bin and extract-info.json.' $false
        if (-not $dataPath) { exit 0 }
    } else {
        $isoDialog = New-Object System.Windows.Forms.OpenFileDialog
        try {
            $isoDialog.Title = 'Choose your Ratchet & Clank NTSC-U SCUS_971.99 v1.00 ISO'
            $isoDialog.Filter = 'Disc image (*.iso)|*.iso'
            $isoDialog.CheckFileExists = $true
            $isoDialog.Multiselect = $false
            if ($isoDialog.ShowDialog() -ne [System.Windows.Forms.DialogResult]::OK) { exit 0 }
            $isoPath = $isoDialog.FileName
        } finally {
            $isoDialog.Dispose()
        }
        $dataPath = Select-DataFolder 'Choose or create an EMPTY folder for the extracted game data.' $true
        if (-not $dataPath) { exit 0 }
        if (@(Get-ChildItem -LiteralPath $dataPath -Force).Count -ne 0) {
            throw 'The extraction folder must be empty. Choose a new empty folder, or select Yes to use existing data.'
        }
        Write-Host "Extracting game data to: $dataPath"
        & $extractor extract --iso $isoPath --out $dataPath
        if ($LASTEXITCODE -ne 0) {
            throw "Extraction failed (exit $LASTEXITCODE). Read the extractor message above. Incomplete data was left in the selected folder; choose a new empty folder for another attempt."
        }
    }

    foreach ($marker in @('toc.bin', 'extract-info.json')) {
        if (-not (Test-Path -LiteralPath (Join-Path $dataPath $marker) -PathType Leaf)) {
            throw "The selected folder is not a complete ReRAC data folder: missing $marker. Select the folder containing both files, or extract your ISO."
        }
    }

    # Log in the writable package folder where possible. If it is read-only,
    # launch directly so capturing a report does not block playing the mod.
    $logPath = Join-Path $PSScriptRoot 'skybox-mod-last-run.log'
    $canLog = $true
    try {
        Set-Content -LiteralPath $logPath -Value "ReRAC Skybox Mod started $(Get-Date -Format o)" -Encoding UTF8
    } catch {
        $canLog = $false
    }
    Write-Host "Starting with game data: $dataPath"
    if ($canLog) {
        Write-Host "Game log: $logPath"
        # Native stderr is also captured. Continue allows diagnostic stderr from
        # native executables without converting it into a terminating PS error.
        $ErrorActionPreference = 'Continue'
        & $runtime --data-dir $dataPath 2>&1 | Tee-Object -FilePath $logPath -Append
        $runtimeExit = $LASTEXITCODE
        $ErrorActionPreference = 'Stop'
    } else {
        & $runtime --data-dir $dataPath
        $runtimeExit = $LASTEXITCODE
    }
    if ($runtimeExit -ne 0) { throw "The game exited with code $runtimeExit. See the messages above and skybox-mod-last-run.log if available." }
    exit 0
} catch {
    Write-Host ''
    Write-Host $_.Exception.Message -ForegroundColor Red
    exit 1
}
