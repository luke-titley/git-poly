use super::git;
use super::path;
use super::result;

//------------------------------------------------------------------------------
pub fn branch(
    expression: &regex::Regex,
    path: &path::PathBuf,
) -> result::Result<bool> {
    let branch_name = git::get_branch_name(path)?;

    Ok(expression.is_match(branch_name.as_str()))
}
