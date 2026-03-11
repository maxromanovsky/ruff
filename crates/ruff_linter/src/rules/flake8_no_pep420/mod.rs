//! Rules from [flake8-no-pep420](https://pypi.org/project/flake8-no-pep420/).
pub(crate) mod rules;

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use anyhow::Result;
    use test_case::test_case;

    use crate::package::PackageRoot;
    use crate::registry::Rule;

    use crate::assert_diagnostics;
    use crate::settings::LinterSettings;
    use crate::settings::types::PreviewMode;
    use crate::test::{test_path, test_path_with_package, test_resource_path};

    #[test_case(Path::new("test_fail_empty"), Path::new("example.py"))]
    #[test_case(Path::new("test_fail_nonempty"), Path::new("example.py"))]
    #[test_case(Path::new("test_ignored"), Path::new("example.py"))]
    #[test_case(Path::new("test_pass_init"), Path::new("example.py"))]
    #[test_case(Path::new("test_pass_namespace_package"), Path::new("example.py"))]
    #[test_case(Path::new("test_pass_pep723"), Path::new("script.py"))]
    #[test_case(Path::new("test_pass_pyi"), Path::new("example.pyi"))]
    #[test_case(Path::new("test_pass_script"), Path::new("script"))]
    #[test_case(Path::new("test_pass_shebang"), Path::new("example.py"))]
    // All intermediate dirs have `__init__.py`; detect_package_root returns the outermost
    // `foo` as a Root — no INP001 should fire even with preview mode.
    #[test_case(
        Path::new("test_pass_nested_init/foo/bar/baz"),
        Path::new("__init__.py")
    )]
    fn default(path: &Path, filename: &Path) -> Result<()> {
        let snapshot = format!("{}", path.to_string_lossy());
        let p = PathBuf::from(format!(
            "flake8_no_pep420/{}/{}",
            path.display(),
            filename.display()
        ));
        let diagnostics = test_path(
            p.as_path(),
            &LinterSettings {
                namespace_packages: vec![test_resource_path(
                    "fixtures/flake8_no_pep420/test_pass_namespace_package",
                )],
                ..LinterSettings::for_rule(Rule::ImplicitNamespacePackage)
            },
        )?;
        insta::with_settings!({filters => vec![(r"\\", "/")]}, {
            assert_diagnostics!(snapshot, diagnostics);
        });
        Ok(())
    }

    /// Tests that INP001 fires correctly when a `PackageRoot::Nested` is provided. This mirrors
    /// the fix for <https://github.com/astral-sh/ruff/issues/14752> where the language server
    /// always used `PackageRoot::Root` and therefore never triggered this diagnostic.
    ///
    /// The fixture `test_fail_nested` has `foo/__init__.py` and `foo/bar/baz/__init__.py` but
    /// no `foo/bar/__init__.py`, making `foo/bar/baz` a nested package under the implicit
    /// namespace package `foo/bar`.
    #[test]
    fn nested() -> Result<()> {
        let package_dir =
            test_resource_path("fixtures/flake8_no_pep420/test_fail_nested/foo/bar/baz");
        let package = Some(PackageRoot::nested(package_dir.as_path()));
        let diagnostics = test_path_with_package(
            "flake8_no_pep420/test_fail_nested/foo/bar/baz/__init__.py",
            package,
            &LinterSettings {
                preview: PreviewMode::Enabled,
                ..LinterSettings::for_rule(Rule::ImplicitNamespacePackage)
            },
        )?;
        insta::with_settings!({filters => vec![(r"\\", "/")]}, {
            assert_diagnostics!("test_fail_nested", diagnostics);
        });
        Ok(())
    }
}
