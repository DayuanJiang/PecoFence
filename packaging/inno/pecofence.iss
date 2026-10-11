; Compile through scripts/make-windows.ps1 using the shared release payload.
#ifndef PayloadDir
  #error PayloadDir is required
#endif
#ifndef AppVersion
  #error AppVersion is required
#endif
#ifndef Repository
  #error Repository is required
#endif
#define NumericVersion Copy(AppVersion, 1, Pos('-', AppVersion + '-') - 1)
#define ProductId "DayuanJiang.PecoFence"
#define ProductName "PecoFence"
#define CurrentRunName "PecoFence"
#define LegacyRunName "openFence"
#define CurrentTaskName "PecoFence"
#define LegacyTaskName "openFence"
#define AppMutexNames "Local\PecoFence.SingleInstance,Local\openFence.SingleInstance"
; Integration tests compile this same script with an isolated identity. The public
; packaging command never sets TestIdentity; test installers must never be shipped.
#ifdef TestIdentity
  #define ProductId "PecoFence.InstallerTest." + TestIdentity
  #define ProductName "PecoFence Installer Test " + TestIdentity
  #define CurrentRunName ProductId
  #define LegacyRunName ProductId + ".legacy"
  #define CurrentTaskName ProductId + ".task"
  #define LegacyTaskName ProductId + ".legacy-task"
  #define AppMutexNames "Local\" + ProductId
#endif

[Setup]
AppId={#ProductId}
AppName={#ProductName}
AppVersion={#AppVersion}
VersionInfoVersion={#NumericVersion}
AppPublisher=Dayuan Jiang
AppPublisherURL=https://github.com/{#Repository}
AppSupportURL=https://github.com/{#Repository}/issues
AppUpdatesURL=https://github.com/{#Repository}/releases
DefaultDirName={localappdata}\Programs\{#ProductName}
DefaultGroupName={#ProductName}
PrivilegesRequired=lowest
UsePreviousAppDir=yes
DisableProgramGroupPage=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0.22621
SetupIconFile=..\..\crates\app\assets\pecofence.ico
UninstallDisplayIcon={app}\pecofence.exe
OutputBaseFilename=pecofence-v{#AppVersion}-x64-setup
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
AppMutex={#AppMutexNames}
SetupMutex=Local\{#ProductId}.Setup
; The app/watchdog must restore desktop icons through their normal exit path.
CloseApplications=no
RestartApplications=no

[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"
Name: "zh_CN"; MessagesFile: "compiler:Languages\ChineseSimplified.isl"
Name: "zh_TW"; MessagesFile: "compiler:Languages\ChineseTraditional.isl"
Name: "ja"; MessagesFile: "compiler:Languages\Japanese.isl"
Name: "ko"; MessagesFile: "compiler:Languages\Korean.isl"
Name: "de"; MessagesFile: "compiler:Languages\German.isl"
Name: "fr"; MessagesFile: "compiler:Languages\French.isl"
Name: "es"; MessagesFile: "compiler:Languages\Spanish.isl"
Name: "pt_BR"; MessagesFile: "compiler:Languages\BrazilianPortuguese.isl"
Name: "ru"; MessagesFile: "compiler:Languages\Russian.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
; Explicit payload: never collect config, caches or arbitrary files from a checkout.
Source: "{#PayloadDir}\pecofence-watchdog.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\pecofence-cli.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\WebView2Loader.dll"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\LICENSE-WebView2Loader.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\THIRD-PARTY-LICENSES.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\UPGRADING.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\SKILL.md"; DestDir: "{app}"; Flags: ignoreversion
; Last: an update aborted halfway (a file in use) keeps the old app, which offers it again.
Source: "{#PayloadDir}\pecofence.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#ProductName}"; Filename: "{app}\pecofence.exe"; WorkingDir: "{app}"
Name: "{autodesktop}\{#ProductName}"; Filename: "{app}\pecofence.exe"; WorkingDir: "{app}"; Tasks: desktopicon

[Run]
Filename: "{app}\pecofence.exe"; Description: "{cm:LaunchProgram,{#ProductName}}"; Flags: nowait postinstall skipifsilent
; The in-app updater runs setup silently and has exited: start the new version.
Filename: "{app}\pecofence.exe"; Flags: nowait; Check: RestartAfterUpdate

[CustomMessages]
en.AlreadyInstalled=PecoFence is already installed in %1. To install it in another folder, uninstall it first.
zh_CN.AlreadyInstalled=PecoFence 已经装在 %1。想换个位置，请先卸载。
zh_TW.AlreadyInstalled=PecoFence 已經安裝在 %1。想換個位置，請先解除安裝。
ja.AlreadyInstalled=PecoFence はすでに %1 にインストールされています。別のフォルダーに入れるには、先にアンインストールしてください。
ko.AlreadyInstalled=PecoFence가 이미 %1에 설치되어 있습니다. 다른 폴더에 설치하려면 먼저 제거하세요.
de.AlreadyInstalled=PecoFence ist bereits in %1 installiert. Um es in einen anderen Ordner zu installieren, deinstalliere es zuerst.
fr.AlreadyInstalled=PecoFence est déjà installé dans %1. Pour l'installer dans un autre dossier, désinstallez-le d'abord.
es.AlreadyInstalled=PecoFence ya está instalado en %1. Para instalarlo en otra carpeta, desinstálalo primero.
pt_BR.AlreadyInstalled=O PecoFence já está instalado em %1. Para instalá-lo em outra pasta, desinstale-o primeiro.
ru.AlreadyInstalled=PecoFence уже установлен в %1. Чтобы установить его в другую папку, сначала удалите его.
en.InvalidDestination=This folder already contains other files. Choose an empty folder, or the folder that already contains PecoFence.
zh_CN.InvalidDestination=这个文件夹里已经有别的文件了。请选一个空文件夹，或者原来放 PecoFence 的文件夹。
zh_TW.InvalidDestination=這個資料夾裡已經有其他檔案。請選一個空資料夾，或原本放 PecoFence 的資料夾。
ja.InvalidDestination=このフォルダーには別のファイルがあります。空のフォルダーか、PecoFence が入っているフォルダーを選んでください。
ko.InvalidDestination=이 폴더에는 이미 다른 파일이 있습니다. 빈 폴더나 PecoFence가 들어 있는 폴더를 선택하세요.
de.InvalidDestination=Dieser Ordner enthält bereits andere Dateien. Wähle einen leeren Ordner oder den Ordner, in dem PecoFence schon liegt.
fr.InvalidDestination=Ce dossier contient déjà d'autres fichiers. Choisissez un dossier vide ou celui qui contient déjà PecoFence.
es.InvalidDestination=Esta carpeta ya tiene otros archivos. Elige una carpeta vacía o la que ya contiene PecoFence.
pt_BR.InvalidDestination=Esta pasta já tem outros arquivos. Escolha uma pasta vazia ou a que já contém o PecoFence.
ru.InvalidDestination=В этой папке уже есть другие файлы. Выберите пустую папку или ту, где уже лежит PecoFence.
en.NewerVersion=A newer version of PecoFence is already installed. Uninstall it first to install this older version. Your settings and fences are kept.
zh_CN.NewerVersion=已经装了更新的 PecoFence。要装这个旧版本，请先卸载。设置和栅栏都会保留。
zh_TW.NewerVersion=已經安裝了較新的 PecoFence。要安裝這個舊版本，請先解除安裝。設定和圍欄都會保留。
ja.NewerVersion=新しいバージョンの PecoFence がインストールされています。この古いバージョンを入れるには、先にアンインストールしてください。設定とフェンスは残ります。
ko.NewerVersion=더 최신 버전의 PecoFence가 설치되어 있습니다. 이 이전 버전을 설치하려면 먼저 제거하세요. 설정과 펜스는 그대로 남습니다.
de.NewerVersion=Eine neuere Version von PecoFence ist bereits installiert. Um diese ältere Version zu installieren, deinstalliere sie zuerst. Deine Einstellungen und Bereiche bleiben erhalten.
fr.NewerVersion=Une version plus récente de PecoFence est déjà installée. Pour installer cette version plus ancienne, désinstallez-la d'abord. Vos réglages et vos groupes sont conservés.
es.NewerVersion=Ya hay una versión más reciente de PecoFence instalada. Para instalar esta versión anterior, desinstálala primero. Tus ajustes y grupos se conservan.
pt_BR.NewerVersion=Uma versão mais recente do PecoFence já está instalada. Para instalar esta versão anterior, desinstale-a primeiro. Suas configurações e grupos são mantidos.
ru.NewerVersion=Уже установлена более новая версия PecoFence. Чтобы установить старую версию, сначала удалите новую. Настройки и области сохранятся.

[Code]
const
  UninstallKey = 'Software\Microsoft\Windows\CurrentVersion\Uninstall\{#ProductId}_is1';
  RunKey = 'Software\Microsoft\Windows\CurrentVersion\Run';

var
  Updating, Installed: Boolean;

function OpenProcess(Access: DWORD; Inherit: BOOL; Pid: DWORD): THandle;
  external 'OpenProcess@kernel32.dll stdcall';
function WaitForSingleObject(Handle: THandle; Milliseconds: DWORD): DWORD;
  external 'WaitForSingleObject@kernel32.dll stdcall';
function CloseHandle(Handle: THandle): BOOL;
  external 'CloseHandle@kernel32.dll stdcall';

{ /UPDATE=<pid>,<pid>...: started by PecoFence's updater, which exits right away. Wait, 30 s
  at most in all, until it, its watchdogs and its instance lock are gone, so the AppMutex
  check that follows passes and no file is in use. Every pid is opened first, so one that
  exits early cannot be reused by another process meanwhile. }
function InitializeSetup: Boolean;
var
  Pids: String;
  Comma, Count, I, Round: Integer;
  Processes: array of THandle;
  Waiting: Boolean;
begin
  Pids := ExpandConstant('{param:UPDATE|}');
  Updating := Pids <> '';
  Count := 0;
  while Pids <> '' do begin
    Comma := Pos(',', Pids);
    if Comma = 0 then
      Comma := Length(Pids) + 1;
    SetArrayLength(Processes, Count + 1);
    Processes[Count] := OpenProcess($00100000 { SYNCHRONIZE }, False, StrToIntDef(Copy(Pids, 1, Comma - 1), 0));
    if Processes[Count] <> 0 then
      Count := Count + 1;
    Delete(Pids, 1, Comma);
  end;
  if Updating then begin
    for Round := 0 to 300 do begin
      Waiting := CheckForMutexes('{#AppMutexNames}');
      for I := 0 to Count - 1 do
        if WaitForSingleObject(Processes[I], 0) <> 0 then
          Waiting := True;
      if not Waiting or (Round = 300) then begin
        Log(Format('Update: waited %d ms for PecoFence to exit', [Round * 100]));
        Break;
      end;
      Sleep(100);
    end;
  end;
  for I := 0 to Count - 1 do
    CloseHandle(Processes[I]);
  Result := True;
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then
    Installed := True;
end;

{ Test installers carry the real app: they log the restart instead of running it. }
function StartApp(const Executable: String): Boolean;
#ifndef TestIdentity
var
  Code: Integer;
#endif
begin
#ifdef TestIdentity
  Log('Test identity: would start ' + Executable);
  Result := False;
#else
  Result := ExecAsOriginalUser(Executable, '', '', SW_SHOWNORMAL, ewNoWait, Code);
#endif
end;

function RestartAfterUpdate: Boolean;
begin
  Result := Updating;
#ifdef TestIdentity
  if Result then
    StartApp(ExpandConstant('{app}\pecofence.exe'));
  Result := False;
#endif
end;

{ An update that failed or was cancelled must not leave the user without PecoFence (unless it
  never exited: then it is still running). }
procedure DeinitializeSetup;
var
  PreviousDir: String;
begin
  if Updating and not Installed and not CheckForMutexes('{#AppMutexNames}') and
    RegQueryStringValue(HKCU, UninstallKey, 'InstallLocation', PreviousDir) and
    FileExists(AddBackslash(PreviousDir) + 'pecofence.exe') then
    StartApp(AddBackslash(PreviousDir) + 'pecofence.exe');
end;

function SameDirectory(const Left, Right: String): Boolean;
begin
  Result := CompareText(RemoveBackslashUnlessRoot(ExpandFileName(Left)),
    RemoveBackslashUnlessRoot(ExpandFileName(Right))) = 0;
end;

function DirectoryHasEntries(const Path: String): Boolean;
var
  Entry: TFindRec;
begin
  Result := False;
  if FindFirst(AddBackslash(Path) + '*', Entry) then begin
    try
      repeat
        if (Entry.Name <> '.') and (Entry.Name <> '..') then begin
          Result := True;
          Break;
        end;
      until not FindNext(Entry);
    finally
      FindClose(Entry);
    end;
  end;
end;

function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  PreviousDir, PreviousVersion: String;
  PreviousNumber, NewNumber: Int64;
begin
  Result := '';
  { A registration whose folder was deleted by hand no longer pins the location. }
  if RegQueryStringValue(HKCU, UninstallKey, 'InstallLocation', PreviousDir) and
    DirExists(PreviousDir) then begin
    if not SameDirectory(PreviousDir, ExpandConstant('{app}')) then begin
      Result := FmtMessage(CustomMessage('AlreadyInstalled'), [RemoveBackslashUnlessRoot(PreviousDir)]);
      Exit;
    end;
    if RegQueryStringValue(HKCU, UninstallKey, 'DisplayVersion', PreviousVersion) then begin
      { Compare the numeric release version; prerelease suffixes do not affect ordering. }
      if Pos('-', PreviousVersion) > 0 then
        PreviousVersion := Copy(PreviousVersion, 1, Pos('-', PreviousVersion) - 1);
      if StrToVersion(PreviousVersion, PreviousNumber) and
        StrToVersion('{#NumericVersion}', NewNumber) then
        if ComparePackedVersion(PreviousNumber, NewNumber) > 0 then
          Result := CustomMessage('NewerVersion');
    end;
  end else if DirectoryHasEntries(ExpandConstant('{app}')) and
    not FileExists(ExpandConstant('{app}\pecofence.exe')) then
    { A folder the ZIP was extracted into is upgraded in place; unrelated files are not. }
    Result := CustomMessage('InvalidDestination');
end;

function StartupExecutable(const Command: String): String;
var
  Value: String;
  ClosingQuote, Space: Integer;
begin
  Value := Trim(Command);
  if (Length(Value) > 0) and (Value[1] = '"') then begin
    ClosingQuote := Pos('"', Copy(Value, 2, Length(Value) - 1));
    if ClosingQuote > 0 then
      Result := Copy(Value, 2, ClosingQuote - 1)
    else
      Result := Copy(Value, 2, Length(Value) - 1);
  end else begin
    Space := Pos(' ', Value);
    if Space > 0 then
      Result := Copy(Value, 1, Space - 1)
    else
      Result := Value;
  end;
end;

procedure RemoveOwnedStartupValue(const Name: String);
var
  Command, Executable: String;
begin
  Executable := ExpandConstant('{app}\pecofence.exe');
  if RegQueryStringValue(HKCU, RunKey, Name, Command) then
    if CompareText(StartupExecutable(Command), Executable) = 0 then
      RegDeleteValue(HKCU, RunKey, Name);
end;

function DecodeUtf16Le(const Bytes: AnsiString): String;
var
  I, First: Integer;
  Code: Word;
begin
  Result := '';
  First := 1;
  if (Length(Bytes) >= 2) and (Ord(Bytes[1]) = $FF) and (Ord(Bytes[2]) = $FE) then
    First := 3;
  I := First;
  while I + 1 <= Length(Bytes) do begin
    Code := Ord(Bytes[I]) + (Ord(Bytes[I + 1]) shl 8);
    Result := Result + Chr(Code);
    I := I + 2;
  end;
end;

function IsUtf8(const Bytes: AnsiString): Boolean;
var
  I, J, Need, Value: Integer;
begin
  I := 1;
  while I <= Length(Bytes) do begin
    Value := Ord(Bytes[I]);
    if Value <= $7F then
      Need := 0
    else if (Value >= $C2) and (Value <= $DF) then
      Need := 1
    else if (Value >= $E0) and (Value <= $EF) then
      Need := 2
    else if (Value >= $F0) and (Value <= $F4) then
      Need := 3
    else begin
      Result := False;
      Exit;
    end;
    if I + Need > Length(Bytes) then begin
      Result := False;
      Exit;
    end;
    for J := 1 to Need do
      if (Ord(Bytes[I + J]) < $80) or (Ord(Bytes[I + J]) > $BF) then begin
        Result := False;
        Exit;
      end;
    I := I + Need + 1;
  end;
  Result := True;
end;

function DecodeTaskXml(const Bytes: AnsiString): String;
begin
  { schtasks declares UTF-16 in the XML prologue, but its redirected console output is
    commonly UTF-8 or the active ANSI code page and has no BOM. Only use UTF-16 decoding
    when the bytes prove it; otherwise let Inno convert the AnsiString using Windows' code page. }
  if (Length(Bytes) >= 2) and (Ord(Bytes[1]) = $FF) and (Ord(Bytes[2]) = $FE) then
    Result := DecodeUtf16Le(Bytes)
  else if (Length(Bytes) >= 3) and (Ord(Bytes[1]) = $EF) and
    (Ord(Bytes[2]) = $BB) and (Ord(Bytes[3]) = $BF) then
    Result := UTF8Decode(Copy(Bytes, 4, Length(Bytes) - 3))
  else if IsUtf8(Bytes) then
    Result := UTF8Decode(Bytes)
  else
    Result := Bytes;
end;

function QueryTaskXml(const Name: String; var Xml: String): Boolean;
var
  FileName, Parameters: String;
  Raw: AnsiString;
  ExitCode: Integer;
begin
  FileName := AddBackslash(ExpandConstant('{tmp}')) + 'pecofence-task.xml';
  Parameters := '/C ""' + ExpandConstant('{sys}\schtasks.exe') +
    '" /Query /TN "' + Name + '" /XML > "' + FileName + '""';
  Xml := '';
  Raw := '';
  Result := Exec(ExpandConstant('{sys}\cmd.exe'), Parameters, '', SW_HIDE,
    ewWaitUntilTerminated, ExitCode) and (ExitCode = 0) and FileExists(FileName) and
    LoadStringFromFile(FileName, Raw);
  if Result then
    Xml := DecodeTaskXml(Raw);
  DeleteFile(FileName);
end;

function XmlElement(const Xml, Name: String): String;
var
  OpenTag, CloseTag: String;
  StartAt, EndAt, RelativeEnd: Integer;
begin
  Result := '';
  OpenTag := '<' + Name + '>';
  CloseTag := '</' + Name + '>';
  StartAt := Pos(OpenTag, Xml);
  if StartAt = 0 then
    Exit;
  StartAt := StartAt + Length(OpenTag);
  RelativeEnd := Pos(CloseTag, Copy(Xml, StartAt, Length(Xml) - StartAt + 1));
  if RelativeEnd > 0 then
    EndAt := StartAt + RelativeEnd - 1
  else
    EndAt := 0;
  if EndAt > StartAt then
    Result := Copy(Xml, StartAt, EndAt - StartAt);
end;

function XmlUnescape(const Value: String): String;
begin
  Result := Value;
  StringChangeEx(Result, '&quot;', '"', True);
  StringChangeEx(Result, '&apos;', '''', True);
  StringChangeEx(Result, '&lt;', '<', True);
  StringChangeEx(Result, '&gt;', '>', True);
  StringChangeEx(Result, '&amp;', '&', True);
end;

function Unquote(const Value: String): String;
begin
  Result := Trim(Value);
  if (Length(Result) >= 2) and (Result[1] = '"') and
    (Result[Length(Result)] = '"') then
    Result := Copy(Result, 2, Length(Result) - 2);
end;

function HasPortableArgument(const Arguments: String): Boolean;
var
  Normalized: String;
begin
  { Be conservative about ownership when a user has added other switches or quoted
    --portable. A portable owner must survive uninstalling the installed copy. }
  Normalized := Lowercase(Arguments);
  StringChangeEx(Normalized, '"', '', True);
  StringChangeEx(Normalized, #9, ' ', True);
  Result := Pos(' --portable ', ' ' + Normalized + ' ') > 0;
end;

procedure RemoveOwnedStartupTask(const Name: String);
var
  Xml, Actions, Action, Command, Arguments, Executable: String;
  ResultCode: Integer;
begin
  { A portable copy or another installation may use the same task name. Only remove a task
    whose Exec command belongs to this install directory. }
  if not QueryTaskXml(Name, Xml) then
    Exit;
  Actions := XmlElement(Xml, 'Actions');
  { Actions has a Context attribute in tasks generated by PecoFence. Match the Exec
    block directly, but preserve any task customized to contain multiple actions. }
  if Actions = '' then
    Actions := Xml;
  Action := XmlElement(Actions, 'Exec');
  if (Action = '') or
    (Pos('<Exec>', Copy(Actions, Pos('<Exec>', Actions) + 6, Length(Actions))) > 0) or
    (Pos('<ComHandler', Actions) > 0) then
    Exit;
  Command := Unquote(XmlUnescape(XmlElement(Action, 'Command')));
  Arguments := Trim(XmlUnescape(XmlElement(Action, 'Arguments')));
  if (Command = '') or HasPortableArgument(Arguments) then
    Exit;
  Executable := ExpandConstant('{app}\pecofence.exe');
  if CompareText(RemoveBackslashUnlessRoot(ExpandFileName(Command)),
    RemoveBackslashUnlessRoot(ExpandFileName(Executable))) <> 0 then
    Exit;
  Exec(ExpandConstant('{sys}\schtasks.exe'),
    '/Delete /TN "' + Name + '" /F', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then begin
    RemoveOwnedStartupTask('{#CurrentTaskName}');
    RemoveOwnedStartupTask('{#LegacyTaskName}');
    RemoveOwnedStartupValue('{#CurrentRunName}');
    RemoveOwnedStartupValue('{#LegacyRunName}');
  end;
  if CurUninstallStep = usDone then
    Log('PecoFence uninstall cleanup completed');
end;
