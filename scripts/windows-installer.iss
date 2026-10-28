; Inno Setup script for the SerialRUN Windows installer.
; Built by scripts/package-windows.ps1, which passes /DMyAppVersion, /DStageDir,
; /DOutputDir and /DRepoRoot on the ISCC command line.

#ifndef MyAppVersion
  #define MyAppVersion "0.0.0"
#endif
#ifndef StageDir
  #define StageDir "..\dist\SerialRUN-" + MyAppVersion + "-windows-x64"
#endif
#ifndef OutputDir
  #define OutputDir "..\dist"
#endif
#ifndef RepoRoot
  #define RepoRoot ".."
#endif

#define MyAppName "SerialRUN"
#define MyAppPublisher "YaoIsAI"
#define MyAppURL "https://www.serialrun.com"
#define MyAppExeName "serialrun.exe"

[Setup]
AppId={{30798AFE-C4D1-402E-AAE5-5652D993C04F}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL=https://github.com/YaoIsAI/SerialRUN/issues
AppUpdatesURL=https://github.com/YaoIsAI/SerialRUN/releases
DefaultDirName={autopf}\SerialRUN
DefaultGroupName=SerialRUN
DisableProgramGroupPage=yes
LicenseFile={#StageDir}\LICENSE
OutputDir={#OutputDir}
OutputBaseFilename=serialrun-{#MyAppVersion}-windows-x64-setup
SetupIconFile={#RepoRoot}\crates\serialrun-gui\icon.ico
UninstallDisplayIcon={app}\{#MyAppExeName}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
CloseApplications=yes
RestartApplications=no
SetupLogging=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#StageDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\SerialRUN"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\{cm:UninstallProgram,SerialRUN}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\SerialRUN"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,SerialRUN}"; Flags: nowait postinstall skipifsilent
