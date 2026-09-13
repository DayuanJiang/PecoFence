"""Original sample artwork and a staged project-board window for the launch film."""
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont
import math

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "public" / "launch"
SAMPLES = OUT / "demo-images"
SAMPLES.mkdir(parents=True, exist_ok=True)
FONT = Path("C:/Windows/Fonts")


def font(size, bold=False):
    return ImageFont.truetype(str(FONT / ("segoeuib.ttf" if bold else "segoeui.ttf")), size)


for name, colors in [
    ("Coast", ("#d2eee7", "#388d9b", "#14687d")),
    ("Dunes", ("#eed8b5", "#da977c", "#925b69")),
    ("Horizon", ("#c2cff7", "#8789d1", "#4d508c")),
]:
    image = Image.new("RGB", (720, 480), colors[0])
    draw = ImageDraw.Draw(image)
    draw.ellipse((510, 45, 610, 145), fill="#fff7e9")
    for level, color in enumerate(colors[1:]):
        points = [
            (x, 270 + level * 115 + int(62 * math.sin(x / 185 + level * 1.5)))
            for x in range(0, 721, 4)
        ]
        draw.polygon(points + [(720, 480), (0, 480)], fill=color)
    image.save(SAMPLES / f"{name}.png")

board = Image.new("RGB", (2360, 1120), "#f3f5f8")
d = ImageDraw.Draw(board)
d.rectangle((0, 0, 2360, 105), fill="#ffffff")
d.text((55, 28), "Launchpad", font=font(34, True), fill="#153149")
d.text((336, 34), "Workspace / Website redesign", font=font(26), fill="#687987")
d.ellipse((2248, 24, 2300, 76), fill="#b2ccc8")
d.text((2260, 32), "A", font=font(26, True), fill="#214a4b")
d.line((0, 105, 2360, 105), fill="#e0e6ec", width=2)
d.rectangle((0, 108, 300, 1120), fill="#eaf0f4")
d.rounded_rectangle((28, 159, 271, 219), 12, fill="#d9e7ee")
for y, label in [(170, "Overview"), (268, "Projects"), (356, "Team files"), (444, "Activity")]:
    d.text((54, y), label, font=font(29, y == 170), fill="#244258")
d.text((366, 167), "Website redesign", font=font(57, True), fill="#17354a")
d.text((370, 250), "A fresh direction. A shared workspace.", font=font(29), fill="#6e8090")
d.rounded_rectangle((2010, 184, 2275, 244), 16, fill="#163d54")
d.text((2042, 197), "+ New task", font=font(26, True), fill="#ffffff")
columns = [
    ("TO EXPLORE", [("Visual direction", "Moodboards and references", "DESIGN"),
                    ("A clearer story", "Collect ideas for the new site", "CONTENT")]),
    ("IN PROGRESS", [("Homepage concepts", "Three directions for review", "DESIGN"),
                     ("Product photography", "Select the final image set", "CREATIVE")]),
    ("READY TO SHARE", [("Brand foundations", "Colors, type and tone", "COMPLETE"),
                       ("Launch checklist", "Everything in one place", "PLANNING")]),
]
for col, (label, cards) in enumerate(columns):
    x = 370 + col * 640
    d.text((x, 351), label, font=font(25, True), fill="#617587")
    for row, (title, subtitle, tag) in enumerate(cards):
        y = 415 + row * 268
        d.rounded_rectangle((x, y + 4, x + 595, y + 230), 23, fill="#e2e8ed")
        d.rounded_rectangle((x, y, x + 595, y + 225), 23, fill="#ffffff")
        d.rounded_rectangle((x + 28, y + 24, x + 236, y + 64), 10, fill="#e7f0ef")
        d.text((x + 42, y + 29), tag, font=font(19, True), fill="#527e79")
        d.text((x + 30, y + 88), title, font=font(32, True), fill="#284356")
        d.text((x + 30, y + 147), subtitle, font=font(24), fill="#80909d")
d.text((370, 1025), "One team. A little more clarity.", font=font(27), fill="#8496a2")
board.save(OUT / "workspace-board.png")
print("Created original demo thumbnails and the staged project workspace.")
