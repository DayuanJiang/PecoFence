// A staged foreground application for the promo. This is not a PecoFence feature.
// It reads an actual sample document and records when its window has been shown.
using System;
using System.Collections.Generic;
using System.Drawing;
using System.IO;
using System.Security.Cryptography;
using System.Text;
using System.Web.Script.Serialization;
using System.Windows.Forms;
using System.Runtime.InteropServices;

static class DemoDocumentViewer {
    [DllImport("user32.dll")]
    static extern bool SetProcessDpiAwarenessContext(IntPtr context);
    [STAThread]
    static void Main(string[] args) {
        if (args.Length != 6) return;
        SetProcessDpiAwarenessContext(new IntPtr(-4));
        string document = Path.GetFullPath(args[0]);
        string proof = Path.GetFullPath(args[1]);
        if (!File.Exists(document)) return;
        string body = File.ReadAllText(document, Encoding.UTF8);
        string digest;
        using (var sha = SHA256.Create()) {
            digest = BitConverter.ToString(sha.ComputeHash(File.ReadAllBytes(document)))
                .Replace("-", "").ToLowerInvariant();
        }
        Application.EnableVisualStyles();
        Application.SetCompatibleTextRenderingDefault(false);
        using (var form = new Form()) {
            form.Text = "Brief — Demo document viewer";
            form.FormBorderStyle = FormBorderStyle.FixedSingle;
            form.MaximizeBox = false;
            form.MinimizeBox = false;
            form.StartPosition = FormStartPosition.Manual;
            form.AutoScaleMode = AutoScaleMode.None;
            form.Bounds = new Rectangle(Int32.Parse(args[2]), Int32.Parse(args[3]),
                Int32.Parse(args[4]), Int32.Parse(args[5]));
            form.BackColor = ColorTranslator.FromHtml("#faf9f6");
            form.TopMost = false;
            var header = new Label {
                Text = "Brief",
                Font = new Font("Segoe UI", 72, FontStyle.Bold, GraphicsUnit.Pixel),
                ForeColor = ColorTranslator.FromHtml("#18354a"),
                Location = new Point(55, 32),
                Size = new Size(form.Width - 110, 100),
            };
            var caption = new Label {
                Text = "WEBSITE LAUNCH",
                Font = new Font("Segoe UI", 26, FontStyle.Bold, GraphicsUnit.Pixel),
                ForeColor = ColorTranslator.FromHtml("#43867f"),
                Location = new Point(60, 146),
                Size = new Size(form.Width - 120, 40),
            };
            var text = new Label {
                Text = body,
                Font = new Font("Segoe UI", 38, FontStyle.Regular, GraphicsUnit.Pixel),
                ForeColor = ColorTranslator.FromHtml("#3b5364"),
                Location = new Point(60, 216),
                Size = new Size(form.Width - 120, form.Height - 250),
            };
            form.Controls.Add(header);
            form.Controls.Add(caption);
            form.Controls.Add(text);
            form.Shown += delegate {
                var record = new Dictionary<string, object> {
                    {"document", document}, {"documentSha256", digest},
                    {"title", form.Text}, {"shownAt", DateTime.UtcNow.ToString("o")},
                    {"pid", System.Diagnostics.Process.GetCurrentProcess().Id},
                };
                File.WriteAllText(proof, new JavaScriptSerializer().Serialize(record), Encoding.UTF8);
            };
            var timer = new Timer {Interval = 12000};
            timer.Tick += delegate { timer.Stop(); form.Close(); };
            timer.Start();
            Application.Run(form);
            timer.Dispose();
        }
    }
}
