// TODO use `proc_macro_error` crate
use std::panic as abort_call_site;

use std::env;

use defmt_parser::Level;

pub(crate) struct EnvFilter {
    entries: Vec<Entry>,
}

impl EnvFilter {
    pub(crate) fn from_env_var() -> Self {
        let defmt_log = env::var("DEFMT_LOG").ok();
        let cargo_pkg_name =
            env::var("CARGO_PKG_NAME").unwrap_or_else(|_| abort_call_site!("TODO"));

        Self::new(defmt_log.as_deref(), &cargo_pkg_name)
    }

    fn new(defmt_log: Option<&str>, cargo_pkg_name: &str) -> Self {
        // match `env_logger` behavior
        const DEFAULT_LEVEL: Level = Level::Trace;

        let caller_crate = cargo_pkg_name;

        let entries = if let Some(input) = defmt_log {
            input
                .split(',')
                .filter_map(|item| {
                    let (module_path, level) = if let Some((module_path, level)) =
                        item.rsplit_once('=')
                    {
                        let level = from_str(level).unwrap_or_else(|_| abort_call_site!("TODO"));

                        (module_path, level)
                    } else {
                        (item, DEFAULT_LEVEL)
                    };

                    validate_module_path(module_path);

                    if module_path.starts_with(&caller_crate) {
                        Some(Entry {
                            module_path: module_path.to_string(),
                            level,
                        })
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            vec![]
        };

        EnvFilter { entries }
    }

    pub(crate) fn level(&self) -> Level {
        // match `env_logger` behavior
        const DEFAULT_LEVEL: Level = Level::Error;

        // to match `env_logger` behaviour, use the last entry
        self.entries
            .iter()
            .last()
            .map(|entry| entry.level)
            .unwrap_or(DEFAULT_LEVEL)
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
    // for now we only accept crate name as module paths
    if path.contains("::") {
        abort_call_site!("TODO")
    }

    validate_identifier(path)
}

fn validate_identifier(_ident: &str) {
    // TODO re-use `syn` logic (?)
}

struct Entry {
    #[allow(dead_code)]
    module_path: String,
    level: Level,
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO unclear if we want the same behavior as `env_logger`
    #[test]
    fn when_duplicates_entries_in_defmt_log_use_last_entry() {
        let env_filter = EnvFilter::new(Some("krate=info,krate=debug"), "krate");
        assert_eq!(Level::Debug, env_filter.level());
    }

    #[test]
    fn when_empty_defmt_log_use_error() {
        let env_filter = EnvFilter::new(None, "dont_care");
        assert_eq!(Level::Error, env_filter.level());
    }

    #[test]
    fn when_no_level_in_defmt_log_use_trace() {
        let env_filter = EnvFilter::new(Some("krate"), "krate");
        assert_eq!(Level::Trace, env_filter.level());
    }

    #[test]
    fn when_level_in_defmt_log_use_it() {
        let env_filter = EnvFilter::new(Some("krate=info"), "krate");
        assert_eq!(Level::Info, env_filter.level());
    }
}
