; pdf2md-gui Windows installer (Inno Setup 6).
; Build: ISCC.exe installer\pdf2md-gui.iss
;   (install Inno Setup 6 first: winget install -e --id JRSoftware.InnoSetup)
; Keep MyAppVersion in sync with Cargo.toml.

#define MyAppName "PDF 轉 Markdown"
#define MyAppExe "pdf2md-gui.exe"
#define MyAppVersion "0.13.0"
#define MyAppPublisher "pdf2md-gui"

[Setup]
AppId={{3F4E6B1A-7C2D-4E8A-9F1B-5A6C7D8E9F01}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={autopf}\pdf2md-gui
DefaultGroupName={#MyAppName}
PrivilegesRequired=lowest
OutputDir=..\dist
OutputBaseFilename=pdf2md-gui-setup-{#MyAppVersion}
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
UninstallDisplayName={#MyAppName}
UninstallDisplayIcon={app}\{#MyAppExe}

[Languages]
Name: "chinesetraditional"; MessagesFile: "Languages\ChineseTraditional.isl"

[Tasks]
Name: "desktopicon"; Description: "建立桌面捷徑"; GroupDescription: "附加捷徑："

[Files]
Source: "..\target\release\{#MyAppExe}"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExe}"
Name: "{group}\解除安裝 {#MyAppName}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExe}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExe}"; Description: "立即啟動 {#MyAppName}"; Flags: nowait postinstall skipifsilent
