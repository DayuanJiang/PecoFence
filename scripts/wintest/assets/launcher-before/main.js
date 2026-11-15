// The launcher sample as the build before interface 2 can run it (for the before pictures of
// plugin-launcher.ps1 Part B): its hint only.
const hint = pf.lang.startsWith("zh") ? "把程序、文件或文件夹拖到这里，或者右键这个栅栏来添加。" : "Drop programs, files or folders here, or right-click this fence to add one.";
pf.on("start", () => pf.show({ rows: [{ text: hint }] }));
