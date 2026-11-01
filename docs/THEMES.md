# Custom themes

A custom theme is a JSON file that starts from one of the built-in styles, Fluent or Liquid
Glass, and changes some of its values: the frosting and tint of the wallpaper behind the fences,
the Liquid Glass lens, the corners, the shadow, the fonts and the colours. Sizes and layout
(title bar height, icon spacing) stay as they are, so a theme never moves or resizes a fence.

## Using a theme

1. Settings › 常规 › 自定义主题 › 打开主题文件夹 opens the `themes` folder next to
   `config.json` (`%APPDATA%\PecoFence\themes`; the portable edition uses `config\themes` next to
   the program). When the folder is missing, the button makes it and puts three example
   themes in it, named in the interface language: Frosted Glass (a frosted Liquid Glass), Warm
   (Fluent in warm colours) and Rounded (Fluent with larger corners and a softer shadow).
   Deleted examples do not come back.
2. Put a theme file there, or copy an example and change it. Each `.json` file is one theme,
   and its file name is the name Settings lists.
3. Choose it in the 自定义主题 list. A click on the Fluent or Liquid Glass tile goes back to that
   built-in style.

The fences change as soon as the chosen file is saved, so you can keep the file open in an
editor and try values. When the file cannot be read or has a mistake, the fences keep the
built-in style, and Settings shows what is wrong (the log has the same line).

From a terminal: `pecofence-cli settings set customTheme "Frosted Glass" --string`, and
`pecofence-cli settings set customTheme null` to go back.

## The file

```json
{
  "base": "liquidGlass",
  "blur": 8,
  "refraction": 0.85,
  "rimLight": 1.2,
  "cornerRadius": 22,
  "light": { "tint": "#FFFFFF38", "luminosity": 0.15 },
  "dark": { "tint": "#1E212866", "luminosity": 0.15 }
}
```

- `base` is required: `"fluent"` or `"liquidGlass"`.
- Keys at the top level apply in both colour modes. The same keys inside `"light"` or `"dark"`
  apply only in that mode and win over the top level. Fonts go at the top level only.
- Colours are `"#RRGGBB"` or `"#RRGGBBAA"`; the last pair is the opacity (`00` clear, `FF` solid).
- Every value is optional; anything left out keeps the base style's value. Values outside the
  range below are moved to its nearest end. An unknown key is an error, so a typo shows up in
  Settings instead of doing nothing.

### Wallpaper material

The wallpaper behind each fence is blurred and tinted with these values once, when the
wallpaper or the theme changes. Liquid Glass then bends the result through its lens.

| Key | Range | What it does |
|---|---|---|
| `blur` | 0 to 60 | Blur radius in DIPs. Fluent uses 30, Liquid Glass 0 (clear glass). |
| `tint` | colour | Colour laid over the wallpaper; its opacity is the strength. |
| `luminosity` | 0 to 1 | How far the wallpaper's brightness is pulled towards the tint's. |
| `chroma` | 0 to 1.5 | Colour saturation kept: 0 grey, 1 unchanged, above 1 stronger. |
| `noise` | 0 to 0.2 | Grain. |

### Liquid Glass lens

| Key | Range | What it does |
|---|---|---|
| `refraction` | 0 to 2 | Strength of the bending at the edges, as a multiple of the built-in one. |
| `rimLight` | 0 to 3 | Brightness of the lit rim, as a multiple of the built-in one. |

### Corners and shadow

| Key | Range | What it does |
|---|---|---|
| `cornerRadius` | 0 to 32 | Corner radius in DIPs; 0 gives square corners. Built in, Fluent uses 8 and Liquid Glass 16. |
| `shadowOpacity` | 0 to 1 | Darkness of the shadow at its edge (built in 0.16 light, 0.30 dark); 0 removes it. |
| `shadowBlur` | 0 to 24 | Softness of the shadow in DIPs (built in 8 Fluent, 12 Liquid Glass). |
| `shadowOffset` | 0 to 16 | How far the shadow falls below the fence, in DIPs (built in 4 Fluent, 5 Liquid Glass). |

### Fonts

| Key | Range | What it does |
|---|---|---|
| `labelFont` | font name | Item names, and all the text of the 列表 and 详细信息 rows. Built in, they use the font Windows gives desktop icons. |
| `labelFontSize` | 8 to 24 | Item name size in DIPs (Windows' default is 12), times Windows' text size setting (Settings › Accessibility › Text size). The grid makes its rows taller or shorter to fit; 列表 and 详细信息 rows grow with larger text. |
| `labelFontWeight` | 100 to 900 | 400 regular, 600 semibold, 700 bold. |
| `titleFont` | font name | Fence titles and tabs (built in Segoe UI Variable). Each fence's 标题字号 option still picks the size. |
| `titleFontWeight` | 100 to 900 | Built in 600. Resting tabs in a left tab column stay regular. |

A font name is a family name, in English or in the font's own language, such as
`"Microsoft YaHei UI"`, `"微软雅黑"` or `"Georgia"`; for Segoe UI Variable write
`"Segoe UI Variable Text"`. A font that is not installed is an error, so a misspelt name shows
up in Settings. Text the font has no letters for (Chinese in a Latin font, for
example) is drawn in a fallback font.

### Colours

`accent` replaces the Windows accent colour: selected items, the drop highlight and the tab
marker take it, with the opacity they normally have. The other keys set one colour each.

| Key | Where it shows |
|---|---|
| `glassLayerFill` | The layer over the wallpaper on every fence. |
| `solidFill` | The plate where no wallpaper shows: without a wallpaper picture, the 更厚实 opacity, a floating Liquid Glass fence. |
| `layerFill` | The scrollbar track. |
| `glassRimTop`, `glassRimBottom` | Fluent: the edge and its sheen, at the top and at the bottom. Liquid Glass lights its own rim (see `rimLight`). |
| `stroke` | The outline of a fence with no wallpaper behind it, and the lines in the details view. |
| `textPrimary` | Titles and item names. |
| `textSecondary` | Details, dates and other secondary text. |
| `textTertiary` | The item count and the text of an empty fence. |
| `controlStrong` | The scrollbar thumb. |
| `tabActiveFill` | The active tab. |
| `subtleHover`, `subtlePressed` | An item or button under the pointer, and while it is pressed. |
| `selectionFill`, `selectionStroke` | Selected items while the fence is the active window. |
| `selectionFillInactive`, `selectionStrokeInactive` | Selected items while another window is active. |
| `focusOuter`, `focusInner` | The keyboard focus ring. |
| `dropHighlight` | The wash while something is dragged over a fence. |

In Liquid Glass, and on fully transparent fences, the text follows the wallpaper behind each
fence: over a bright wallpaper it takes the theme's `light` text colours, over a dark one the
`dark` ones, whatever the colour mode. So set text colours for both modes. 图标名称颜色 白色 and
黑色 take the theme's `dark` and `light` text colour for item names, and for the titles and tabs
of fences without a title colour of their own.

The per-fence options (opacity, tint, title colour) apply on top of the theme as they do on the
built-in styles.

## A Fluent example

```json
{
  "base": "fluent",
  "accent": "#E8710A",
  "light": {
    "tint": "#FFDDB8CC",
    "luminosity": 0.3,
    "glassLayerFill": "#FFF0DC59",
    "textPrimary": "#3B2412",
    "textSecondary": "#6B4A2E",
    "tabActiveFill": "#FFFFFFCC",
    "stroke": "#B4642840"
  },
  "dark": {
    "tint": "#3A2414D9",
    "glassLayerFill": "#2A180C40",
    "textPrimary": "#FFF3E6",
    "textSecondary": "#F2DCC6",
    "tabActiveFill": "#FFE4C833"
  }
}
```
