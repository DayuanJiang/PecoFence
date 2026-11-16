//! The data files a plugin may write into a folder the user picked and drag out of its rows.

pub const DATA_TYPES: [&str; 10] = [
    "txt", "md", "json", "csv", "log", "ics", "png", "jpg", "gif", "webp",
];

pub fn is_data_file(name: &str) -> bool {
    name.rsplit_once('.').is_some_and(|(stem, ext)| {
        !stem.is_empty() && DATA_TYPES.contains(&ext.to_ascii_lowercase().as_str())
    })
}
