pub use self::{
    global::{load as load_global, DefaultSetting, Settings as GlobalSettings},
    repo::{fill_context, load as load_repo, new_context, IgnoreFrom, IgnorePattern, RepoSettings},
};

mod global;
mod repo;
