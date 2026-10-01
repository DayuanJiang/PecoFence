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
#define AppMutexNames "Local\PecoFence.SingleInstance,Local\openFence.SingleInstance"
; Integration tests compile this same script with an isolated identity. The public
; packaging command never sets TestIdentity; test installers must never be shipped.
#ifdef TestIdentity
  #define ProductId "PecoFence.InstallerTest." + TestIdentity
  #define ProductName "PecoFence Installer Test " + TestIdentity
  #define CurrentRunName ProductId
  #define LegacyRunName ProductId + ".legacy"
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
LicenseFile={#PayloadDir}\LICENSE
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
Source: "{#PayloadDir}\pecofence.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\pecofence-watchdog.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\pecofence-cli.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\WebView2Loader.dll"; DestDir: "{app}"; Flags: ignoreversion
; PrepareToInstall validates this immutable marker on upgrades. Retain it so an
; interrupted upgrade cannot truncate the identity needed to run setup again.
Source: "{#PayloadDir}\deployment.json"; DestDir: "{app}"; Flags: onlyifdoesntexist
Source: "{#PayloadDir}\release-info.json"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\LICENSE-WebView2Loader.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\THIRD-PARTY-LICENSES.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\UPGRADING.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\SKILL.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#ProductName}"; Filename: "{app}\pecofence.exe"; WorkingDir: "{app}"
Name: "{autodesktop}\{#ProductName}"; Filename: "{app}\pecofence.exe"; WorkingDir: "{app}"; Tasks: desktopicon

[Run]
Filename: "{app}\pecofence.exe"; Description: "{cm:LaunchProgram,{#ProductName}}"; Flags: nowait postinstall skipifsilent unchecked

[CustomMessages]
en.InvalidDestination=Choose an empty folder. To upgrade, use the existing installer-managed folder and leave deployment.json unchanged. Portable folders cannot be converted in place.
zh_CN.InvalidDestination=请选择空文件夹。升级时请使用原安装目录，并保持 deployment.json 不变。不能直接覆盖免安装版文件夹。
zh_TW.InvalidDestination=請選擇空資料夾。升級時請使用原安裝目錄，並保持 deployment.json 不變。不能直接覆蓋免安裝版資料夾。
ja.InvalidDestination=空のフォルダーを選択してください。更新時は既存のインストール先を使用し、deployment.json を変更しないでください。ポータブル版のフォルダーには上書きできません。
ko.InvalidDestination=빈 폴더를 선택하세요. 업그레이드할 때는 기존 설치 폴더를 사용하고 deployment.json을 변경하지 마세요. 포터블 폴더에는 덮어쓸 수 없습니다.
de.InvalidDestination=Wählen Sie einen leeren Ordner. Verwenden Sie für Updates den bisherigen Installationsordner und ändern Sie deployment.json nicht. Portable Ordner können nicht überschrieben werden.
fr.InvalidDestination=Choisissez un dossier vide. Pour une mise à niveau, utilisez le dossier d'installation existant sans modifier deployment.json. Les dossiers portables ne peuvent pas être remplacés.
es.InvalidDestination=Elige una carpeta vacía. Para actualizar, usa la carpeta de instalación existente y no modifiques deployment.json. No se pueden sobrescribir las carpetas portables.
pt_BR.InvalidDestination=Escolha uma pasta vazia. Para atualizar, use a pasta de instalação existente e não altere deployment.json. Pastas portáteis não podem ser sobrescritas.
ru.InvalidDestination=Выберите пустую папку. Для обновления используйте прежнюю папку установки и не изменяйте deployment.json. Портативную папку нельзя перезаписать установщиком.
en.NewerVersion=A newer version is already installed. Uninstall it before installing an older version. Your application data will be kept.
zh_CN.NewerVersion=已安装较新的版本。请先卸载，再安装旧版本。应用数据将会保留。
zh_TW.NewerVersion=已安裝較新的版本。請先解除安裝，再安裝舊版本。應用程式資料將會保留。
ja.NewerVersion=新しいバージョンがインストールされています。古いバージョンを入れる前にアンインストールしてください。アプリのデータは保持されます。
ko.NewerVersion=더 최신 버전이 설치되어 있습니다. 이전 버전을 설치하기 전에 제거하세요. 앱 데이터는 유지됩니다.
de.NewerVersion=Eine neuere Version ist bereits installiert. Deinstallieren Sie diese zuerst. Ihre Anwendungsdaten bleiben erhalten.
fr.NewerVersion=Une version plus récente est déjà installée. Désinstallez-la avant d'installer une ancienne version. Vos données seront conservées.
es.NewerVersion=Ya hay una versión más reciente instalada. Desinstálala antes de instalar una anterior. Se conservarán tus datos.
pt_BR.NewerVersion=Uma versão mais recente já está instalada. Desinstale-a antes de instalar uma versão anterior. Seus dados serão preservados.
ru.NewerVersion=Уже установлена более новая версия. Удалите её перед установкой старой версии. Данные приложения будут сохранены.

[Code]
const
  UninstallKey = 'Software\Microsoft\Windows\CurrentVersion\Uninstall\{#ProductId}_is1';
  RunKey = 'Software\Microsoft\Windows\CurrentVersion\Run';
  InstalledMarker = '{"schema":1,"appId":"PecoFence","mode":"installed"}';

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
  Marker: AnsiString;
  Registered: Boolean;
begin
  Result := '';
  Registered := RegQueryStringValue(HKCU, UninstallKey, 'InstallLocation', PreviousDir);
  if Registered then begin
    if not SameDirectory(PreviousDir, ExpandConstant('{app}')) then begin
      Result := CustomMessage('InvalidDestination');
      Exit;
    end;
    if RegQueryStringValue(HKCU, UninstallKey, 'DisplayVersion', PreviousVersion) then begin
      { Compare the numeric release version; prerelease suffixes do not affect ordering. }
      if Pos('-', PreviousVersion) > 0 then
        PreviousVersion := Copy(PreviousVersion, 1, Pos('-', PreviousVersion) - 1);
      if StrToVersion(PreviousVersion, PreviousNumber) and
        StrToVersion('{#NumericVersion}', NewNumber) then
        if ComparePackedVersion(PreviousNumber, NewNumber) > 0 then begin
          Result := CustomMessage('NewerVersion');
          Exit;
        end;
    end;
  end;
  if DirectoryHasEntries(ExpandConstant('{app}')) then begin
    { This is our generated marker, not a general JSON parser. Do not adopt ZIPs. }
    if not Registered then
      Result := CustomMessage('InvalidDestination')
    else if not LoadStringFromFile(ExpandConstant('{app}\deployment.json'), Marker) then
      Result := CustomMessage('InvalidDestination')
    else if Trim(String(Marker)) <> InstalledMarker then
      Result := CustomMessage('InvalidDestination');
  end;
end;

procedure RemoveOwnedStartupValue(const Name: String);
var
  Command, Executable: String;
begin
  Executable := ExpandConstant('{app}\pecofence.exe');
  if RegQueryStringValue(HKCU, RunKey, Name, Command) then
    if (CompareText(Trim(Command), Executable) = 0) or
      (CompareText(Trim(Command), '"' + Executable + '"') = 0) then
      RegDeleteValue(HKCU, RunKey, Name);
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then begin
    RemoveOwnedStartupValue('{#CurrentRunName}');
    RemoveOwnedStartupValue('{#LegacyRunName}');
  end;
  if CurUninstallStep = usDone then
    Log('PecoFence uninstall cleanup completed');
end;
