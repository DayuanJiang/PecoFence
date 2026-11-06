# Shows a Vista-style folder picker (IFileOpenDialog with FOS_PICKFOLDERS) and writes the result
# (HRESULT and the chosen folder) to C:\pf\run\picker.out. Started by test steps.
Add-Type @'
using System;
using System.Runtime.InteropServices;
[ComImport, Guid("DC1C5A9C-E88A-4dde-A5A1-60F82A20AEF7")] public class FileOpenDialogCo {}
[ComImport, Guid("43826d1e-e718-42ee-bc55-a1e261c37bfe"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
public interface IShellItem { void BindToHandler(); void GetParent(); [PreserveSig] int GetDisplayName(uint sigdn, [MarshalAs(UnmanagedType.LPWStr)] out string name); }
[ComImport, Guid("42f85136-db7e-439c-85f1-e4075d135fc8"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
public interface IFileDialog {
  [PreserveSig] int Show(IntPtr parent);
  void SetFileTypes(uint c, IntPtr f); void SetFileTypeIndex(uint i); void GetFileTypeIndex(out uint i);
  void Advise(IntPtr p, out uint c); void Unadvise(uint c);
  void SetOptions(uint fos); void GetOptions(out uint fos);
  void SetDefaultFolder(IntPtr si); void SetFolder(IntPtr si); void GetFolder(out IntPtr si);
  void GetCurrentSelection(out IntPtr si); void SetFileName([MarshalAs(UnmanagedType.LPWStr)] string n);
  void GetFileName([MarshalAs(UnmanagedType.LPWStr)] out string n); void SetTitle([MarshalAs(UnmanagedType.LPWStr)] string t);
  void SetOkButtonLabel(string t); void SetFileNameLabel(string t);
  void GetResult(out IShellItem si);
}
public static class Picker {
  public static string Run() {
    var d = (IFileDialog)new FileOpenDialogCo();
    uint o; d.GetOptions(out o); d.SetOptions(o | 0x20);
    int hr = d.Show(IntPtr.Zero);
    if (hr != 0) return "hr=" + hr.ToString("X8");
    IShellItem si; d.GetResult(out si); string n; si.GetDisplayName(0x80058000, out n);
    return "chosen=" + n;
  }
}
'@
[Picker]::Run() | Set-Content C:\pf\run\picker.out -Encoding UTF8
