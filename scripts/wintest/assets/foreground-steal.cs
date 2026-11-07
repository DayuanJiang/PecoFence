// A toast-like window in a process of its own that takes the foreground on a signal, the way
// the Windows Security notification did in the fence review (S14).
//   foreground-steal.exe <delayMs> <durationMs> <hideAfterMs> <log>
// Shows itself without activation, sets Local\PfStealReady, waits for Local\PfStealGo, sleeps
// delayMs, then for durationMs takes the foreground whenever another window has it (at least
// once), keeps the window hideAfterMs longer and closes it (the foreground goes elsewhere).
using System;
using System.Diagnostics;
using System.Drawing;
using System.IO;
using System.Runtime.InteropServices;
using System.Threading;
using System.Windows.Forms;

public class Steal : Form {
  [DllImport("user32.dll")] static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, IntPtr pid);
  [DllImport("kernel32.dll")] static extern uint GetCurrentThreadId();
  [DllImport("user32.dll")] static extern bool AttachThreadInput(uint a, uint b, bool attach);
  [DllImport("user32.dll")] static extern bool BringWindowToTop(IntPtr h);
  [DllImport("winmm.dll")] static extern uint timeBeginPeriod(uint p);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetClassNameW(IntPtr h, System.Text.StringBuilder s, int n);

  static string Cls(IntPtr h) { var s = new System.Text.StringBuilder(256); GetClassNameW(h, s, 256); return s.ToString(); }

  protected override bool ShowWithoutActivation { get { return true; } }
  protected override CreateParams CreateParams {
    get { var p = base.CreateParams; p.ExStyle |= 0x8 | 0x80; return p; } // WS_EX_TOPMOST | WS_EX_TOOLWINDOW
  }

  [STAThread]
  public static void Main(string[] args) {
    int delay = int.Parse(args[0]), duration = int.Parse(args[1]), hideAfter = int.Parse(args[2]);
    string log = args[3];
    timeBeginPeriod(1);
    var f = new Steal();
    f.FormBorderStyle = FormBorderStyle.None;
    f.StartPosition = FormStartPosition.Manual;
    f.Bounds = new Rectangle(1220, 680, 364, 150);
    f.BackColor = Color.FromArgb(43, 43, 43);
    f.TopMost = true;
    f.Text = "PfSteal toast";
    var l = new Label();
    l.Text = "Windows Security (test toast)\r\nVirus & threat protection";
    l.ForeColor = Color.White; l.Font = new Font("Segoe UI", 11); l.Dock = DockStyle.Fill;
    f.Controls.Add(l);
    f.Show();
    Application.DoEvents();
    var ready = new EventWaitHandle(false, EventResetMode.ManualReset, "Local\\PfStealReady");
    var go = new EventWaitHandle(false, EventResetMode.AutoReset, "Local\\PfStealGo");
    ready.Set();
    var w = new StreamWriter(log, true);
    if (!go.WaitOne(30000)) { w.WriteLine("no go signal"); w.Close(); return; }
    var sw = Stopwatch.StartNew();
    while (sw.ElapsedMilliseconds < delay) { Thread.Sleep(1); }
    IntPtr me = f.Handle;
    uint mine = GetCurrentThreadId();
    int steals = 0;
    string first = "";
    var until = sw.ElapsedMilliseconds + duration;
    do {
      IntPtr fg = GetForegroundWindow();
      if (fg != me) {
        uint theirs = GetWindowThreadProcessId(fg, IntPtr.Zero);
        bool att = theirs != 0 && theirs != mine && AttachThreadInput(mine, theirs, true);
        bool ok = SetForegroundWindow(me);
        BringWindowToTop(me);
        if (att) AttachThreadInput(mine, theirs, false);
        if (steals == 0) first = string.Format("first steal at {0} ms from '{1}': ok={2} attached={3}", sw.ElapsedMilliseconds, Cls(fg), ok, att);
        steals++;
      }
      Application.DoEvents();
      Thread.Sleep(1);
    } while (sw.ElapsedMilliseconds < until || steals == 0 && sw.ElapsedMilliseconds < until + 2000);
    w.WriteLine(string.Format("{0}; steals={1}; foreground at the end '{2}'", first, steals, Cls(GetForegroundWindow())));
    var hide = Stopwatch.StartNew();
    while (hide.ElapsedMilliseconds < hideAfter) { Application.DoEvents(); Thread.Sleep(5); }
    f.Close();
    Application.DoEvents();
    Thread.Sleep(100);
    w.WriteLine("closed; foreground now '" + Cls(GetForegroundWindow()) + "'");
    w.Close();
  }
}
