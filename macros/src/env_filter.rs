use std::{
    collections::{BTreeMap, BTreeSet},
    env,
};
// TODO use `proc_macro_error` crate
use std::panic as abort_call_site;

use defmt_parser::Level;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

pub(crate) struct EnvFilter {
    entries: BTreeMap<String, Level>,
}

impl EnvFilter {
    pub(crate) fn from_env_var() -> Self {
        let defmt_log = env::var("DEFMT_LOG").ok();
        let cargo_crate_name =
            env::var("CARGO_CRATE_NAME").unwrap_or_else(|_| abort_call_site!("TODO"));

        Self::new(defmt_log.as_deref(), &cargo_crate_name)
    }

    fn new(defmt_log: Option<&str>, cargo_crate_name: &str) -> Self {
        // match `env_logger` behavior
        const LEVEL_WHEN_LEVEL_IS_NOT_SPECIFIED: Level = Level::Trace;
        const LEVEL_WHEN_CRATE_IS_NOT_SPECIFIED: Level = Level::Error;

        let caller_crate = cargo_crate_name;

        let mut entries = BTreeMap::new();
        if let Some(input) = defmt_log {
            for item in input.rsplit(',') {
                let (module_path, level) = if let Some((module_path, level)) = item.rsplit_once('=')
                {
                    let level = from_str(level).unwrap_or_else(|_| abort_call_site!("TODO"));

                    (module_path, level)
                } else {
                    (item, LEVEL_WHEN_LEVEL_IS_NOT_SPECIFIED)
                };

                validate_module_path(module_path);

                if module_path.starts_with(&caller_crate) && !entries.contains_key(module_path) {
                    entries.insert(module_path.to_string(), level);
                }
            }
        }

        if !entries.contains_key(caller_crate) {
            entries.insert(caller_crate.to_string(), LEVEL_WHEN_CRATE_IS_NOT_SPECIFIED);
        }

        EnvFilter { entries }
    }

    /// Builds a compile-time check that returns `true` when `module_path!` can emit logs at the
    /// requested log `level`
    ///
    /// Returns `None` if the caller crate (at any module path) will never emit logs at requested log `level`
    pub(crate) fn path_check(&self, level: Level) -> Option<TokenStream2> {
        let paths = self.paths_for_level(level);

        if paths.is_empty() {
            return None;
        }

        let conds = paths
            .iter()
            .map(|needle| {
                let needle = needle.as_bytes();
                let needle_len = needle.len();
                let byte_checks = needle
                    .iter()
                    .enumerate()
                    .map(|(index, byte)| quote!(haystack[#index] == #byte))
                    .collect::<Vec<_>>();

                quote!(
                    // start of const-context `[u8]::starts_with(needle)`
                    if #needle_len > haystack.len() {
                        false
                    } else {
                        #(#byte_checks &&)*
                    // end of const-context `[u8]::starts_with`

                    // check that what follows the `needle` is the end of a path segment
                    if #needle_len == haystack.len() {
                        true
                    } else {
                        // `haystack` comes from `module_path!`; we assume it's well-formed so we
                        // don't check *everything* that comes after `needle`; just a single character of
                        // what should be the path separator ("::")
                        haystack[#needle_len] == b':'
                    }
                })
            })
            .collect::<Vec<_>>();

        Some(quote!({
            const fn check() -> bool {
                let haystack = module_path!().as_bytes();
                false #(|| #conds)*
            }

            check()
        }))
    }

    fn paths_for_level(&self, level: Level) -> BTreeSet<&str> {
        self.entries
            .iter()
            .rev()
            .filter_map(|(module_path, min_level)| {
                if level >= *min_level {
                    Some(module_path.as_str())
                } else {
                    None
                }
            })
            .collect()
    }
}

// TODO should be `impl FromStr for Level`
fn from_str(s: &str) -> Result<Level, ()> {
    Ok(match s {
        "debug" => Level::Debug,
        "info" => Level::Info,
        "error" => Level::Error,
        "trace" => Level::Trace,
        "warn" => Level::Warn,
        _ => return Err(()),
    })
}

fn validate_module_path(path: &str) {
    for segment in path.split("::") {
        validate_identifier(segment)
    }
}

fn validate_identifier(_ident: &str) {
    // TODO re-use `syn` logic (?)
}

#[cfg(test)]
mod tests {
    use maplit::btreeset;

    use super::*;

    #[test]
    fn when_duplicates_entries_in_defmt_log_use_last_entry() {
        let env_filter = EnvFilter::new(Some("krate=info,krate=debug"), "krate");
        assert_eq!(btreeset!["krate"], env_filter.paths_for_level(Level::Debug));
        assert_eq!(btreeset![], env_filter.paths_for_level(Level::Trace));
    }

    #[test]
    fn when_empty_defmt_log_use_error() {
        let env_filter = EnvFilter::new(None, "krate");
        assert_eq!(btreeset!["krate"], env_filter.paths_for_level(Level::Error));
        assert_eq!(btreeset![], env_filter.paths_for_level(Level::Warn));
    }

    #[test]
    fn when_no_level_in_defmt_log_use_trace() {
        let env_filter = EnvFilter::new(Some("krate"), "krate");
        assert_eq!(btreeset!["krate"], env_filter.paths_for_level(Level::Trace));
    }

    #[test]
    fn when_level_in_defmt_log_use_it() {
        let env_filter = EnvFilter::new(Some("krate=info"), "krate");
        assert_eq!(btreeset!["krate"], env_filter.paths_for_level(Level::Info));
        assert_eq!(btreeset![], env_filter.paths_for_level(Level::Debug));
    }

    #[test]
    fn module_paths_different_levels() {
        let env_filter = EnvFilter::new(Some("krate=info,krate::module=debug"), "krate");
        assert_eq!(
            btreeset!["krate", "krate::module"],
            env_filter.paths_for_level(Level::Info)
        );
        assert_eq!(
            btreeset!["krate::module"],
            env_filter.paths_for_level(Level::Debug)
        );
        assert_eq!(btreeset![], env_filter.paths_for_level(Level::Trace));
    }

    #[ignore = "TODO(optimization): impl & more test cases"]
    #[test]
    fn when_module_paths_with_same_level_remove_inner_ones() {
        let env_filter = EnvFilter::new(Some("krate=info,krate::module=info"), "krate");
        assert_eq!(btreeset!["krate"], env_filter.paths_for_level(Level::Info));
        assert_eq!(btreeset![], env_filter.paths_for_level(Level::Debug));
    }

    #[ignore = "TODO"]
    #[test]
    #[should_panic]
    fn invalid_identifier() {
        EnvFilter::new(Some("krate::some-module"), "krate");
    }
}
