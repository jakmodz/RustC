use std::str::FromStr;

pub(crate) fn number_from<T: FromStr>(str: &str,suffix:(&str,&str))->Option<T>{
    match str.trim_end_matches(suffix.0).trim_end_matches(suffix.1).parse::<T>() {
        Ok(t) => Some(t),
        Err(_) => None,
    }
}