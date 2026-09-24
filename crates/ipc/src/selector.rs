//! Resolving the human/agent-friendly selectors the CLI accepts into ids.

use uuid::Uuid;

/// Shortest UUID prefix accepted, so ordinary titles like `cafe` or `bead` are not mistaken
/// for hex.
pub const MIN_ID_PREFIX: usize = 6;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectorError {
    NotFound,
    /// `(id, name)` of every candidate.
    Ambiguous(Vec<(Uuid, String)>),
}

/// Resolves `sel` against `(id, name)` candidates:
/// 1. a full UUID matches by id;
/// 2. otherwise a hex prefix of at least [`MIN_ID_PREFIX`] characters matches ids starting with
///    it, and the name is matched exactly (case-insensitive) at the same time — a hit on both
///    is ambiguous;
/// 3. otherwise a unique case-insensitive substring of the name.
pub fn resolve<'a, I>(candidates: I, sel: &str) -> Result<Uuid, SelectorError>
where
    I: IntoIterator<Item = (Uuid, &'a str)>,
{
    let sel = sel.trim();
    if sel.is_empty() {
        return Err(SelectorError::NotFound);
    }
    let all: Vec<(Uuid, &str)> = candidates.into_iter().collect();
    if let Ok(id) = Uuid::parse_str(sel) {
        return all
            .iter()
            .find(|(c, _)| *c == id)
            .map(|(c, _)| *c)
            .ok_or(SelectorError::NotFound);
    }
    let lower = sel.to_lowercase();
    let looks_like_prefix =
        lower.len() >= MIN_ID_PREFIX && lower.chars().all(|c| c.is_ascii_hexdigit() || c == '-');
    let mut hits: Vec<(Uuid, String)> = Vec::new();
    for (id, name) in &all {
        let by_prefix = looks_like_prefix && id.to_string().starts_with(&lower);
        let by_name = name.to_lowercase() == lower;
        if by_prefix || by_name {
            hits.push((*id, name.to_string()));
        }
    }
    match hits.len() {
        1 => return Ok(hits[0].0),
        n if n > 1 => return Err(SelectorError::Ambiguous(hits)),
        _ => {}
    }
    let partial: Vec<(Uuid, String)> = all
        .iter()
        .filter(|(_, name)| name.to_lowercase().contains(&lower))
        .map(|(id, name)| (*id, name.to_string()))
        .collect();
    match partial.len() {
        0 => Err(SelectorError::NotFound),
        1 => Ok(partial[0].0),
        _ => Err(SelectorError::Ambiguous(partial)),
    }
}

/// Like [`resolve`], but for ordered lists (rules): a bare non-negative integer is a 0-based
/// index. Returns the position in `candidates`.
pub fn resolve_indexed<'a, I>(candidates: I, sel: &str) -> Result<usize, SelectorError>
where
    I: IntoIterator<Item = (Uuid, &'a str)>,
{
    let all: Vec<(Uuid, &str)> = candidates.into_iter().collect();
    let sel = sel.trim();
    if let Ok(index) = sel.parse::<usize>() {
        return if index < all.len() {
            Ok(index)
        } else {
            Err(SelectorError::NotFound)
        };
    }
    let id = resolve(all.iter().copied(), sel)?;
    all.iter()
        .position(|(c, _)| *c == id)
        .ok_or(SelectorError::NotFound)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(s: &str) -> Uuid {
        Uuid::parse_str(s).unwrap()
    }

    fn fences() -> Vec<(Uuid, String)> {
        vec![
            (id("11111111-2222-3333-4444-555555555555"), "Work".into()),
            (id("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee"), "Games".into()),
            (
                id("aaaaaaaa-1111-cccc-dddd-eeeeeeeeeeee"),
                "Game dev".into(),
            ),
            (id("cafe0000-1111-2222-3333-444444444444"), "cafe".into()),
        ]
    }

    fn run(sel: &str) -> Result<Uuid, SelectorError> {
        let f = fences();
        resolve(f.iter().map(|(i, n)| (*i, n.as_str())), sel)
    }

    #[test]
    fn full_uuid() {
        assert_eq!(
            run("11111111-2222-3333-4444-555555555555"),
            Ok(fences()[0].0)
        );
        assert_eq!(
            run("00000000-0000-0000-0000-000000000000"),
            Err(SelectorError::NotFound)
        );
    }

    #[test]
    fn unique_prefix_needs_six_hex_digits() {
        assert_eq!(run("111111"), Ok(fences()[0].0));
        assert_eq!(run("aaaaaaaa-b"), Ok(fences()[1].0));
        // Too short for a prefix → treated as a name substring → no fence contains "1111".
        assert_eq!(run("1111"), Err(SelectorError::NotFound));
        // Shared prefix is ambiguous.
        assert!(matches!(run("aaaaaaaa"), Err(SelectorError::Ambiguous(c)) if c.len() == 2));
    }

    #[test]
    fn exact_title_is_case_insensitive_and_beats_substring() {
        assert_eq!(run("work"), Ok(fences()[0].0));
        assert_eq!(run("GAMES"), Ok(fences()[1].0));
        // "Game" is a substring of both "Games" and "Game dev" → ambiguous.
        assert!(matches!(run("game"), Err(SelectorError::Ambiguous(c)) if c.len() == 2));
        assert_eq!(run("dev"), Ok(fences()[2].0));
    }

    #[test]
    fn title_that_looks_like_hex_still_resolves_by_name() {
        // "cafe" is 4 chars: never a prefix, exact title wins.
        assert_eq!(run("cafe"), Ok(fences()[3].0));
        // "cafe00" is a valid prefix of the same fence and no other → fine.
        assert_eq!(run("cafe00"), Ok(fences()[3].0));
    }

    #[test]
    fn prefix_and_title_hit_on_different_fences_is_ambiguous() {
        let list: [(Uuid, &str); 2] = [
            (id("abcdef00-0000-0000-0000-000000000000"), "One"),
            (id("00000000-0000-0000-0000-000000000001"), "abcdef"),
        ];
        let r = resolve(list.iter().copied(), "abcdef");
        assert!(matches!(r, Err(SelectorError::Ambiguous(c)) if c.len() == 2));
    }

    #[test]
    fn empty_and_missing() {
        assert_eq!(run(""), Err(SelectorError::NotFound));
        assert_eq!(run("nothing"), Err(SelectorError::NotFound));
    }

    #[test]
    fn indexed_accepts_positions_ids_and_names() {
        let f = fences();
        let it = || f.iter().map(|(i, n)| (*i, n.as_str()));
        assert_eq!(resolve_indexed(it(), "2"), Ok(2));
        assert_eq!(resolve_indexed(it(), "4"), Err(SelectorError::NotFound));
        assert_eq!(resolve_indexed(it(), "Work"), Ok(0));
        assert_eq!(resolve_indexed(it(), "cafe00"), Ok(3));
    }
}
