# Usage statistics

PecoFence sends usage statistics to the PecoFence service. They show which features people use
and in which order, and whether the program starts and runs well, so that the work on PecoFence
goes where the use is.

The statistics are on by default. To switch them off, open **Settings › About** and turn off
**Help improve PecoFence** (CLI: `pecofence-cli settings set usageStats false`). Setup has the
same choice on its options page at a first installation.

## What a report holds

There is one report per installation and day. PecoFence sends a day's report once, after the day
has ended: at its next start, or when the date changes while it runs. A day that could not be
sent is tried again at the next start or day change, and dropped after seven later days. On the
first day of an installation the report also goes out once, 30 minutes after the start.

```json
{ "v": 1, "id": "5f0c2a9e7b1d4c38a6e0f1b2c3d4e5f6", "day": "2026-11-14",
  "app": "0.3.4", "edition": "installer", "windows": "10.0.26200", "language": "zh-CN",
  "settings": { "theme": "dark", "hideRealIcons": true, "customTheme": "unset" },
  "state": { "fences": 6, "tabs": 2, "portals": 1, "smart": 0, "tools": 1, "rules": 5, "items": 73 },
  "health": { "startupMs": 420, "memoryMB": 31, "crashed": false },
  "events": [ [32400, "start", "at_sign_in"], [32655, "hotkey", "peek"], [32671, "launch_item"] ] }
```

| Part | What it is |
|---|---|
| `id` | A random number made at the first start. It stands for this installation; nothing on your PC goes into it. |
| `day` | The local date of the report. |
| `app`, `edition` | The PecoFence version, and whether it is the Store, installer or ZIP edition (and portable). |
| `windows`, `language` | The Windows build and PecoFence's display language. |
| `settings` | The settings that are a switch, a choice from a list or a number, under their names in `config.json`. A setting that holds text (a custom theme, a shortcut, a tint colour) appears only as `"set"` or `"unset"`. |
| `state` | How many fences, tabs, folder fences, smart fences, tool fences, rules and items there are. |
| `health` | How long the last start took, the most memory the program held that day, and whether the run before a start of that day crashed or was cut off. |
| `events` | What was used that day, in order: the second of the day, a name from the list below, and for some names a second word. At most 2,000 a day. |

Between uploads the reports wait in `usage.json`, in `%LOCALAPPDATA%\PecoFence` (beside the
configuration for the portable edition). The file is written when PecoFence exits and at most
every five minutes while it runs.

## What is never sent

- File names, paths, fence titles, note texts, search words and anything else you typed.
- The keys of your shortcuts and the names of your themes.
- Anything about the feedback form.

## Switching it off

Switching the statistics off stops the recording at once and deletes `usage.json`, and the
number of the installation with it. Switched on again, the installation gets a new number.
Importing a configuration or restoring a backup does not move the switch.

## Event names

An event says that something was used. It does not say which file or fence it was used on, and
it is recorded when the action was asked for, also when the action then failed. The names come
from a few central places of the program:

- **`start`** with `at_sign_in`, `by_hand`, `after_update` or `after_crash`; **`exit`**.
- **`hotkey`** with `peek`, `double_ctrl`, `dialog_jump` or `windows_desktop`.
- **`tray`** with `click` (the icon) or `menu` (its menu was opened).
- **`menu`**, when an entry of a menu was chosen, with `tray`, `fence`, `item`, `columns`,
  `search_row` or `right_drag`.
- **`settings.page`** with `general`, `layout`, `fences`, `rules`, `peek`, `tools` or `about`.
- **`settings.set`**, when the settings window changed a setting, with the setting's name
  (never its value): `rules` for the rules, or one of `language`, `theme`, `theme_style`,
  `custom_theme`, `label_color`, `title_align`, `tab_bar`, `icon_size`, `quick_hide.enabled`,
  `roll_up.hover_peek`, `roll_up.click_to_expand`, `roll_up.hover_speed`,
  `roll_up.title_on_hover`, `roll_up.hide_inactive_scrollbar`, `roll_up.push_neighbors`,
  `roll_up.switch_tab_on_hover`, `roll_up.hide_count_and_chevron`, `show_desktop`,
  `hide_real_icons`, `keep_icons_hidden`, `hide_inbox_when_icons_shown`, `offer_new_fence`,
  `per_desktop_fences`, `feature_tips`, `auto_check_updates`, `snapping.enabled`,
  `snapping.gap_px`, `snapping.size_to_cells`, `snapping.guide_lines`, `zorder`, `autostart`,
  `peek.enabled`, `peek.dim`, `peek.double_ctrl`, `peek.custom_hotkey`,
  `windows_desktop_hotkey`, `icons.tint_rgb`, `icons.tint_strength`, `icons.chameleon`,
  `icons.shortcut_arrows`, `tools.enabled`, `tools.clock_seconds`, `tools.dialog_jump`,
  `tools.everything`.
- **`settings.fence`**, when the settings window changed an option of a fence, with the
  option's name (never its value): `title`, `icon_size`, `spacing`, `spacing_px`, `icon_align`,
  `auto_height`, `locked`, `expand_upward`, `exclude_from_quick_hide`, `opacity`, `tint`,
  `tint_title_only`, `tab_color`, `title_color`, `title_size`, `text_size`, `title_on_hover`,
  `layout`, `sort`, `reverse`, `group_by_date`, `folders_first`, `label_lines`,
  `open_folders_inside`, `portal_navigate`, `portal_title_icon`, `hide_when_empty`,
  `desktop_layout`, `free_placement`, `note_text`, `script_path`, `script_interval`,
  `smart_folder`, `smart_match`, `smart_subfolders`, `smart_within_days`, `smart_conditions`,
  `smart_name`, `smart_everything`, `smart_limit`, `virtual_desktops`, `dock_top`.
- **`cli`**, for a request of `pecofence-cli` (or of a second start of PecoFence, which opens
  the settings) that succeeded, with the method: `status.get`, `monitors.list`,
  `virtual_desktops.list`, `fences.list`, `fences.get`, `items.list`, `settings.get`,
  `rules.get`, `snapshots.list`, `fences.create`, `fences.delete`, `fences.rename`,
  `fences.refresh`, `fences.up`, `fences.forward`, `fences.open_folder`, `fences.set_bounds`,
  `fences.move_to_monitor`, `fences.set_option`, `fences.fit`, `fences.roll`,
  `fences.dock_top`, `fences.merge`, `fences.detach`, `fences.set_visible`,
  `fences.open_options`, `items.move`, `items.plan_move`, `items.rename`, `settings.patch`,
  `rules.set`, `rules.add`, `rules.remove`, `rules.enable`, `rules.move`, `rules.apply`,
  `snapshots.save`, `snapshots.restore`, `snapshots.plan_restore`, `snapshots.delete`,
  `config.export`, `config.import`, `backups.list`, `backups.restore`, `peek.start`,
  `peek.end`, `peek.search`, `windows_desktop.start`, `windows_desktop.end`,
  `settings.open_ui`, `events.subscribe`.
- **A command of the program**, one name for each thing a click, a key or a drop asks it to
  do: `fence_dropped` (a fence was moved), `toggle_roll_up`, `fit_to_content`,
  `commit_expanded`, `launch_item`, `move_items`, `reorder_items`, `drop_into_folder`,
  `drag_out_finished`, `portal_enter`, `portal_up`, `portal_forward`, `open_portal_folder`,
  `focus_next_fence`, `merge_fence`, `set_column_widths`, `switch_tab`, `detach_tab`,
  `cancel_detach`, `reorder_tab`, `move_items_to_inbox`, `external_drop`, `external_url_drop`,
  `shell_drop`, `header_menu`, `fence_menu`, `item_menu` (a menu was opened), `clipboard_verb`,
  `paste`, `new_folder`, `step_icon_size`, `item_properties`, `refresh_fence`, `rename_fence`,
  `rename_item`, `rename_item_commit`, `rename_item_invalid`, `delete_items`, `delete_fence`,
  `new_fence`, `new_note`, `new_clock`, `new_script`, `new_script_example`,
  `open_smart_editor`, `open_fence_filter`, `fence_filter_closed`, `clear_fence_filter`,
  `edit_note`, `note_edited`, `toggle_note_line`, `script_row_clicked`, `new_fence_rect`,
  `marquee_offer`, `toggle_peek`, `end_peek`, `peek_search_step`, `peek_search_open`,
  `peek_search_open_row`, `peek_search_open_folder`, `peek_search_actions`,
  `peek_search_row_menu`, `apply_rules_now`, `open_settings`, `open_options_for_fence`,
  `sort_column`, `notice` (a card in the corner was clicked or went away), `undo`.

Not recorded: what the program does by itself (redrawing, raising a fence that was clicked,
taking a search result), what happens many times during one drag, and every typed character.
Resizing a fence and the buttons of the settings window are not recorded either.
