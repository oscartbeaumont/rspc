use crate::router_builder2::BuildErrorCause;

pub(crate) fn is_valid_name(name: &str) -> Option<BuildErrorCause> {
    if name.is_empty() || name.len() > 255 {
        return Some(BuildErrorCause::InvalidName);
    }

    for c in name.chars() {
        if !(c.is_alphanumeric() || c == '_' || c == '-' || c == '~') {
            return Some(BuildErrorCause::InvalidCharInName(c));
        }
    }

    if name == "rspc" || name == "_batch" {
        return Some(BuildErrorCause::ReservedName(name.to_string()));
    }

    None
}
