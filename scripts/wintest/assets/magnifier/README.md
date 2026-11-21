# Magnifier 放大镜

A zoomed view of the part of the screen around the mouse pointer, shown in a fence. PecoFence
captures the screen with the graphics card and draws the picture itself: this plugin only chooses
how (zoom, following, smoothing), and never receives, keeps or sends a pixel.

- **Zoom factor** in the plugin settings: 1x to 20x. The wheel over the fence changes it too
  (away from you: larger), and the fence menu has Zoom in and Zoom out.
- **Follow the pointer**: the picture moves with the pointer, also from one monitor to another
  (monitors with different scaling show the same size to the eye).
- **Follow the keyboard focus** (the button, list item or box that has it) and **Follow the text
  insertion point** (the caret of the box you type in), as in the Windows Magnifier. They work in
  most programs that UI Automation can read (browsers, editors and Windows' own apps included);
  a program that does not tell where its focus or caret is leaves the picture where it was. With
  several of the three on, the picture goes to whichever moved last.
- **Allow locking the picture**: a click in the fence, or the shortcut you give to the Lock
  command (plugin details), holds the picture where it is; again lets it follow. While it is
  held nothing is captured. With all the Follow options off, the same click or shortcut takes
  the picture again at the pointer's place.
- **Smooth enlargement**, **Show the mouse pointer** and **Stay above all windows** (the fence
  would hide behind the program you work in otherwise).

Asks for the permission to show the screen (`screen: magnify`): PecoFence says so before the
install. Sends nothing over the network. The fence itself is not part of the picture, and
content that Windows protects from capture (some video players, password windows) shows black.
Needs Windows 10 version 2004 or later.

---

在栅栏里显示鼠标指针周围屏幕区域的放大画面。由 PecoFence 用显卡截取并绘制画面，本插件只决定
怎么显示（倍数、是否跟随、是否平滑），拿不到、保存不了也发不出任何像素。

- 「放大倍数」在插件设置里，1 到 20 倍；鼠标滚轮（向前滚为放大）和栅栏菜单里的「放大」「缩小」也能调。
- 「跟随鼠标指针」：画面跟着指针走，跨显示器也行（缩放比例不同的显示器上，看到的大小一样）。
- 「跟随键盘焦点」（拥有焦点的按钮、列表项或输入框）和「跟随文本插入点」（正在输入的文本框的光标），
  与 Windows 放大镜相同。通过 UI 自动化能读取的程序（浏览器、编辑器和 Windows 自带应用）里都能用；
  程序不告知焦点或光标位置时，画面保持原处。三项同时打开时，画面去往最近移动的那一个。
- 「允许锁定画面」：在栅栏里点一下，或按给「锁定或解锁画面」命令设的快捷键，画面就停住；再点一下
  继续跟随。停住时不再截屏。关掉所有「跟随」选项后，点击或快捷键是在指针当前位置重新取景。
- 「平滑放大」、「显示鼠标指针」和「始终显示在窗口最前」。

需要「显示屏幕」权限（`screen: magnify`），安装前 PecoFence 会提示。不联网。栅栏本身不会出现在
画面里；Windows 禁止截取的内容（部分视频播放器、密码窗口）会显示为黑色。需要 Windows 10 2004 或更高版本。
