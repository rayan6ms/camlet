param(
	[Parameter(Mandatory = $true)][string]$Installer,
	[Parameter(Mandatory = $true)][string]$OutputDirectory
)

$ErrorActionPreference = "Stop"
$Installer = (Resolve-Path $Installer).Path
$Fixture = (Resolve-Path "fixtures/automation/full-smoke.json").Path

if (Test-Path $OutputDirectory) {
	throw "Package test output directory must be new: $OutputDirectory"
}

New-Item -ItemType Directory -Path $OutputDirectory | Out-Null
$Profile = Join-Path $OutputDirectory "profile"
$install = Start-Process -FilePath $Installer -ArgumentList "/S" -PassThru -Wait
if ($install.ExitCode -ne 0) { throw "NSIS install failed with $($install.ExitCode)" }

$InstalledBinary = Join-Path $env:LOCALAPPDATA "Camlet/camlet.exe"
$Uninstaller = Join-Path $env:LOCALAPPDATA "Camlet/uninstall.exe"
if (!(Test-Path $InstalledBinary) -or !(Test-Path $Uninstaller)) {
	throw "NSIS did not install the expected application payload"
}

foreach ($Run in @("first", "reinstall")) {
	$ResultDirectory = Join-Path $OutputDirectory "installed/$Run"
	$smoke = Start-Process -FilePath $InstalledBinary -ArgumentList @(
		"--frame-source", "synthetic",
		"--profile-dir", $Profile,
		"--automation-script", $Fixture,
		"--automation-output", $ResultDirectory
	) -PassThru -Wait
	if ($smoke.ExitCode -ne 0) { throw "Installed product scenario failed with $($smoke.ExitCode)" }

	$complete = Get-Content (Join-Path $ResultDirectory "complete.json") | ConvertFrom-Json
	$diagnostics = Get-Content (Join-Path $ResultDirectory "diagnostics.json") | ConvertFrom-Json
	if ($complete.status -ne "complete" -or $diagnostics.camera.status -ne "preview") {
		throw "Installed product scenario evidence is invalid"
	}
	foreach ($Capture in @("original", "circle", "rounded-square", "diamond", "rectangle-y", "rectangle-x", "overlay")) {
		if ((Get-Item (Join-Path $ResultDirectory "$Capture.ppm")).Length -eq 0) {
			throw "Installed product screenshot is empty: $Capture"
		}
	}

	$NativeSettings = Join-Path $Profile "settings-v1.json"
	if (!(Test-Path $NativeSettings)) { throw "Application settings were not persisted" }

	if ($Run -eq "first") {
		$uninstall = Start-Process -FilePath $Uninstaller -ArgumentList "/S" -PassThru -Wait
		if ($uninstall.ExitCode -ne 0) { throw "NSIS uninstall failed with $($uninstall.ExitCode)" }
		if (Test-Path $InstalledBinary) { throw "NSIS uninstall left the application binary installed" }
		if (!(Test-Path $NativeSettings)) { throw "NSIS uninstall removed application settings" }

		$reinstall = Start-Process -FilePath $Installer -ArgumentList "/S" -PassThru -Wait
		if ($reinstall.ExitCode -ne 0) { throw "NSIS reinstall failed with $($reinstall.ExitCode)" }
		if (!(Test-Path $InstalledBinary) -or !(Test-Path $Uninstaller)) {
			throw "NSIS reinstall did not restore the expected application payload"
		}
	}
}

$secondUninstall = Start-Process -FilePath $Uninstaller -ArgumentList "/S" -PassThru -Wait
if ($secondUninstall.ExitCode -ne 0) { throw "Second NSIS uninstall failed with $($secondUninstall.ExitCode)" }
if (Test-Path $InstalledBinary) { throw "Second NSIS uninstall left the application binary installed" }
if (!(Test-Path $NativeSettings)) { throw "Second NSIS uninstall removed application settings" }

Get-FileHash $Installer -Algorithm SHA256 |
	Select-Object Hash, Path |
	ConvertTo-Json |
	Set-Content (Join-Path $OutputDirectory "SHA256SUMS.json")
