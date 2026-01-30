//! CLI commands for tspec management (cargo xt ts ...)

mod hash;
mod list;
mod new;
mod show;

pub use hash::hash_tspec;
pub use list::list_tspecs;
pub use new::new_tspec;
pub use show::show_tspec;
