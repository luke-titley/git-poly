//------------------------------------------------------------------------------
use crate::branch_regex::BranchRegex;
use crate::filter;
use crate::repoiterator::RepoIterator;
use crate::result::{get, Result};

//------------------------------------------------------------------------------
pub fn run(regex: &regex::Regex, branch_regex: &BranchRegex) -> Result<()> {
    // Filtered traversal
    if let Some(pattern) = branch_regex {
        for path in RepoIterator::new(regex) {
            if filter::branch(&pattern, &path)? {
                let display = get(path.as_path().to_str())?;
                println!("{0}", display);
            }
        }

    // Unfiltered traversal
    } else {
        for path in RepoIterator::new(regex) {
            let display = get(path.as_path().to_str())?;
            println!("{0}", display);
        }
    }

    Ok(())
}
