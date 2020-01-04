//------------------------------------------------------------------------------
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Tracking {
    Staged,
    Unmerged,
    Unstaged,
    Untracked,
}

//------------------------------------------------------------------------------
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Staging {
    Added,
    Deleted,
    Modified,
    BothModified,
    Untracked,
}

//------------------------------------------------------------------------------
pub type Status = (Tracking, Staging);
