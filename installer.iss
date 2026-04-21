; Inno Setup Script for autoortho
; This script creates a Windows installer for autoortho

#define MyAppName "autoortho"
#define MyAppVersion "0.5.8"
#define MyAppPublisher "autoortho"
#define MyAppExeName "autoortho.exe"

[Setup]
AppId={{E6B7C3D4-8F2A-4E5B-9C1D-3F8E7A2B4C6D}
AppName=#MyAppName
AppVersion=#MyAppVersion
AppPublisher=#MyAppPublisher
DefaultDirName={autopf}\#MyAppName
DefaultGroupName=#MyAppName
OutputBaseFilename=autoortho-setup-#MyAppVersion
Compression=lzma2
SolidCompression=yes
WizardStyle=modern

[Files]
Source: "autoortho.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "winfsp-x64.dll"; DestDir: "{app}"; Flags: ignoreversion
Source: "LICENSE-winfsp.txt"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\#MyAppName"; Filename: "{app}\#MyAppExeName"
Name: "{autodesktop}\#MyAppName"; Filename: "{app}\#MyAppExeName"

[Run]
Filename: "{app}\#MyAppExeName"; Description: "Launch autoortho"; Flags: postinstall nowait skipifsilent