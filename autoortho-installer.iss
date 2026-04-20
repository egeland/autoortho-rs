[Setup]
AppName=autoortho
AppVersion=0.6.44
AppPublisher=Frode Egeland
DefaultDirName={pf}\autoortho
OutputDir=target/distrib
OutputBaseFilename=autoortho-installer-0.6.44
Compression=lzma
SolidCompression=yes

[Files]
Source: "target\x86_64-pc-windows-msvc\dist\autoortho.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "target\x86_64-pc-windows-msvc\dist\winfsp-x64.dll"; DestDir: "{app}"; Flags: ignoreversion
Source: "target\x86_64-pc-windows-msvc\dist\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "target\x86_64-pc-windows-msvc\dist\LICENSE-winfsp.txt"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprogramsmenu}\autoortho"; Filename: "{app}\autoortho.exe"
Name: "{commondesktop}\autoortho"; Filename: "{app}\autoortho.exe"

[Run]
Filename: "{app}\autoortho.exe"; Description: "Launch autoortho"; Flags: nowait postinstall skipifsilent