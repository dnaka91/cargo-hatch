use std::fs;

use anyhow::Result;
use camino::Utf8Path;
use crates_index::Index;
use semver::{Version, VersionReq};
use toml_edit::{Document, Formatted, Item, Value};

use crate::templates::RepoFile;

pub fn update_all_cargo_tomls(target: &Utf8Path, files: &[RepoFile]) -> Result<()> {
    let mut index = Index::new_cargo_default()?;
    index.update()?;

    for file in files {
        if file.name().file_name() == Some("Cargo.toml") {
            let target_file = target.join(file.name());
            let file_content = fs::read_to_string(&target_file)?;
            let mut doc = file_content.parse::<Document>()?;

            for table in ["dependencies", "dev-dependencies", "build-dependencies"] {
                let updates = update_versions(&index, &mut doc, table);
                print_updates(file.name(), updates);
            }

            fs::write(target_file, doc.to_string())?;
        }
    }

    Ok(())
}

struct Update {
    name: String,
    old: String,
    new: String,
}

fn update_versions(index: &impl CrateIndex, doc: &mut Document, table: &str) -> Vec<Update> {
    let mut updates = Vec::new();

    if let Some(deps) = doc.get_mut(table).and_then(Item::as_table_like_mut) {
        for (name, spec) in deps.iter_mut() {
            let version = match spec {
                // plain string version like `anyhow = "1.0.0"`
                Item::Value(Value::String(version)) => Some(version),
                // inline table like `anyhow = { version = "1.0.0" }`
                Item::Value(Value::InlineTable(table)) => match table.get_mut("version") {
                    Some(Value::String(version)) => Some(version),
                    _ => None,
                },
                // dependency as full table like:
                // ```
                // [dependencies.anyhow]
                // version = "1.0.0"
                // ```
                Item::Table(table) => match table.get_mut("version") {
                    Some(Item::Value(Value::String(version))) => Some(version),
                    _ => None,
                },
                _ => None,
            };

            if let Some(version) = version {
                if let Some(latest) = index.find_latest_version(name.get(), version.value()) {
                    let mut latest = Formatted::new(latest.to_string());

                    if version.value() != latest.value() {
                        updates.push(Update {
                            name: name.get().to_owned(),
                            old: version.value().clone(),
                            new: latest.value().clone(),
                        });
                    }

                    std::mem::swap(version.decor_mut(), latest.decor_mut());
                    std::mem::swap(version, &mut latest);
                }
            }
        }
    }

    updates
}

fn print_updates(file: &Utf8Path, updates: Vec<Update>) {
    if updates.is_empty() {
        return;
    }

    println!("updated versions of {file:?}:\n");

    let mins = updates
        .iter()
        .fold(("name".len(), "old".len(), "new".len()), |acc, update| {
            (
                acc.0.max(update.name.len()),
                acc.1.max(update.old.len()),
                acc.2.max(update.new.len()),
            )
        });

    println!(
        "{0:1$} | {2:3$} | {4:5$}",
        "name", mins.0, "old", mins.1, "new", mins.2,
    );
    println!(
        "{0:-<1$} | {2:-<3$} | {4:-<5$}",
        "", mins.0, "", mins.1, "", mins.2,
    );

    for update in updates {
        println!(
            "{0:1$} | {2:3$} | {4:5$}",
            update.name, mins.0, update.old, mins.1, update.new, mins.2,
        );
    }

    println!();
}

trait CrateIndex {
    fn find_latest_version(&self, name: &str, req: &str) -> Option<Version>;
}

impl CrateIndex for Index {
    fn find_latest_version(&self, name: &str, version: &str) -> Option<Version> {
        let req = version.parse::<VersionReq>().ok()?;
        let crate_ = self.crate_(name)?;

        crate_
            .versions()
            .iter()
            .filter(|v| !v.is_yanked())
            .filter_map(|v| {
                v.version()
                    .parse::<Version>()
                    .ok()
                    .filter(|v| req.matches(v))
            })
            .max()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestIndex;

    impl CrateIndex for TestIndex {
        fn find_latest_version(&self, name: &str, req: &str) -> Option<Version> {
            (name == "anyhow" && req == "1.0.0").then(|| Version::new(1, 1, 0))
        }
    }

    #[test]
    fn plain_version() {
        let toml = r#"
            [dependencies]
            anyhow = "1.0.0"
        "#;
        let mut toml = toml.parse::<Document>().unwrap();
        update_versions(&TestIndex, &mut toml, "dependencies");

        let want = r#"
            [dependencies]
            anyhow = "1.1.0"
        "#;

        assert_eq!(want, toml.to_string());
    }

    #[test]
    fn inline_table_version() {
        let toml = r#"
            [dependencies]
            anyhow = { version = "1.0.0", git = "https://github.com/dtolnay/anyhow" }
        "#;
        let mut toml = toml.parse::<Document>().unwrap();
        update_versions(&TestIndex, &mut toml, "dependencies");

        let want = r#"
            [dependencies]
            anyhow = { version = "1.1.0", git = "https://github.com/dtolnay/anyhow" }
        "#;

        assert_eq!(want, toml.to_string());
    }

    #[test]
    fn full_table_version() {
        let toml = r#"
            [dependencies.anyhow]
            version = "1.0.0"
            git = "https://github.com/dtolnay/anyhow"
        "#;
        let mut toml = toml.parse::<Document>().unwrap();
        update_versions(&TestIndex, &mut toml, "dependencies");

        let want = r#"
            [dependencies.anyhow]
            version = "1.1.0"
            git = "https://github.com/dtolnay/anyhow"
        "#;

        assert_eq!(want, toml.to_string());
    }
}
