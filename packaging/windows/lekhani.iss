; Lekhani Bengali Input Method - Inno Setup Script
; Generates Lekhani-v3.0.0-Setup.exe for Windows 10/11 (x64)

#define MyAppName "Lekhani"
#define MyAppVersion "3.0.0"
#define MyAppPublisher "Syntenieum"
#define MyAppURL "https://github.com/sintaulsiam/lekhani"
#define MyAppExeName "lekhani-gui.exe"
#define MyAppCliName "lekhani.exe"

#ifndef SourceDir
  #define SourceDir "..\..\target\release"
#endif

[Setup]
AppId={{D1A2E7F3-5C38-4A59-B812-3D7E81F9C942}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
OutputDir=..\..\dist
OutputBaseFilename=Lekhani-v{#MyAppVersion}-Setup
SetupIconFile=..\..\data\icons\favicon.ico
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64
PrivilegesRequired=lowest
CloseApplications=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "autostart"; Description: "Automatically launch Lekhani when Windows starts"; GroupDescription: "Startup Options:"

[Files]
Source: "{#SourceDir}\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\{#MyAppCliName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\lekhani.dll"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist
Source: "..\..\data\layouts\*"; DestDir: "{app}\data\layouts"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "..\..\data\dictionaries\*"; DestDir: "{app}\data\dictionaries"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "..\..\data\icons\*"; DestDir: "{app}\data\icons"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "..\..\README.md"; DestDir: "{app}"; DestName: "README.txt"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\data\icons\favicon.ico"
Name: "{group}\{#MyAppName} Command Line"; Filename: "{app}\{#MyAppCliName}"; IconFilename: "{app}\data\icons\favicon.ico"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon; IconFilename: "{app}\data\icons\favicon.ico"

[Registry]
; Autostart on Windows Login if user checked the task
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "{#MyAppName}"; ValueData: """{app}\{#MyAppExeName}"""; Tasks: autostart; Flags: uninsdeletevalue

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
