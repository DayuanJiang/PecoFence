# run-hyperv.sh --prep: installs the virtual display driver (MttVDD, signed by SignPath
# Foundation) in the guest and makes its root device once, so the guest gets a second monitor.
# The driver files and the signer certificate come with --extra (C:\pf\extra\vdd).
$d = 'C:\pf\extra\vdd'
$o = @()
New-Item -ItemType Directory -Force C:\VirtualDisplayDriver | Out-Null
Copy-Item "$d\MttVDD.inf", "$d\MttVDD.dll", "$d\mttvdd.cat", "$d\vdd_settings.xml" C:\VirtualDisplayDriver -Force
$o += (certutil -f -addstore TrustedPublisher "$d\signer.cer" 2>&1 | Select-Object -Last 1)
$o += (pnputil /add-driver C:\VirtualDisplayDriver\MttVDD.inf /install 2>&1 | Out-String).Trim() -replace '\s+', ' '
if (-not (Get-PnpDevice -Class Display -ErrorAction SilentlyContinue | Where-Object { $_.HardwareID -contains 'Root\MttVDD' })) {
  Add-Type @'
using System;
using System.Runtime.InteropServices;
using System.Text;
public static class Vdd {
    [StructLayout(LayoutKind.Sequential)] struct SP_DEVINFO_DATA { public uint cbSize; public Guid ClassGuid; public uint DevInst; public IntPtr Reserved; }
    [DllImport("setupapi.dll", SetLastError = true)] static extern IntPtr SetupDiCreateDeviceInfoList(ref Guid g, IntPtr hwnd);
    [DllImport("setupapi.dll", SetLastError = true, CharSet = CharSet.Unicode)] static extern bool SetupDiCreateDeviceInfoW(IntPtr set, string name, ref Guid g, string desc, IntPtr hwnd, uint flags, ref SP_DEVINFO_DATA d);
    [DllImport("setupapi.dll", SetLastError = true, CharSet = CharSet.Unicode)] static extern bool SetupDiSetDeviceRegistryPropertyW(IntPtr set, ref SP_DEVINFO_DATA d, uint prop, byte[] buf, uint size);
    [DllImport("setupapi.dll", SetLastError = true)] static extern bool SetupDiCallClassInstaller(uint fn, IntPtr set, ref SP_DEVINFO_DATA d);
    [DllImport("setupapi.dll", SetLastError = true)] static extern bool SetupDiDestroyDeviceInfoList(IntPtr set);
    [DllImport("newdev.dll", SetLastError = true, CharSet = CharSet.Unicode)] static extern bool UpdateDriverForPlugAndPlayDevicesW(IntPtr hwnd, string hwid, string inf, uint flags, out bool reboot);
    public static string AddRootDevice(string inf, string hwid) {
        Guid g = new Guid("4d36e968-e325-11ce-bfc1-08002be10318");
        IntPtr set = SetupDiCreateDeviceInfoList(ref g, IntPtr.Zero);
        var d = new SP_DEVINFO_DATA();
        d.cbSize = (uint)Marshal.SizeOf(typeof(SP_DEVINFO_DATA));
        try {
            if (!SetupDiCreateDeviceInfoW(set, "Display", ref g, null, IntPtr.Zero, 1, ref d)) return "create " + Marshal.GetLastWin32Error();
            byte[] ids = Encoding.Unicode.GetBytes(hwid + "\0\0");
            if (!SetupDiSetDeviceRegistryPropertyW(set, ref d, 1, ids, (uint)ids.Length)) return "prop " + Marshal.GetLastWin32Error();
            if (!SetupDiCallClassInstaller(0x19, set, ref d)) return "register " + Marshal.GetLastWin32Error();
        } finally { SetupDiDestroyDeviceInfoList(set); }
        bool reboot;
        if (!UpdateDriverForPlugAndPlayDevicesW(IntPtr.Zero, hwid, inf, 1, out reboot)) return "update " + Marshal.GetLastWin32Error();
        return "ok reboot=" + reboot;
    }
}
'@
  $o += 'root device: ' + [Vdd]::AddRootDevice('C:\VirtualDisplayDriver\MttVDD.inf', 'Root\MttVDD')
}
Start-Sleep -Seconds 5
$o += (Get-PnpDevice -Class Display | ForEach-Object { "$($_.FriendlyName) [$($_.Status)]" }) -join ' | '
$o -join ' ;; '
