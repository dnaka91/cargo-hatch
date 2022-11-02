pub use self::{
    global::{load as load_global, DefaultSetting, Settings as GlobalSettings},
    repo::{fill_context, load as load_repo, new_context, IgnorePattern, RepoSettings},
};

mod global;
mod repo;
