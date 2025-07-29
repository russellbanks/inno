use bitflags::Flags;
use num_traits::NumCast;

pub fn int_constraint<T>(int: T) -> u16
where
    T: NumCast + Copy,
{
    int.to_usize()
        .and_then(|usize| u16::try_from(usize.checked_ilog10()? + 1).ok())
        .unwrap_or_default()
}

pub fn ints_constraint<I, T>(ints: I, title: &str) -> u16
where
    I: IntoIterator<Item = T>,
    T: NumCast,
{
    ints.into_iter()
        .filter_map(|int| int.to_usize())
        .max()
        .and_then(usize::checked_ilog10)
        .map_or(0, |log10| (log10 + 1).max(title.len() as u32))
        .try_into()
        .unwrap_or_default()
}

pub fn strings_constraint<'a, I, T>(strings: I, title: &str) -> u16
where
    I: IntoIterator<Item = T>,
    T: Into<Option<&'a str>>,
{
    strings
        .into_iter()
        .filter_map(T::into)
        .map(str::len)
        .max()
        .map_or(0, |len| len.max(title.len()))
        .try_into()
        .unwrap_or_default()
}

pub fn flags_constraint<I, F>(flags_iter: I, title: &str) -> u16
where
    I: IntoIterator<Item = F>,
    F: Flags,
{
    flags_iter
        .into_iter()
        .map(|flags| {
            itertools::intersperse(
                flags.iter_names().map(|(name, _flag)| name.len()),
                " | ".len(),
            )
            .sum::<usize>()
        })
        .max()
        .map_or(0, |len| len.max(title.len()))
        .try_into()
        .unwrap_or_default()
}
