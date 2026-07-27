use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use xshell::{Shell, cmd};

const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const RESET: &str = "\x1b[0m";

mod flags {
    xflags::xflags! {
        cmd config {
            /// Run `brew bundle` with the Brewfile.
            cmd bundle {
                /// When set, will remove any unspecified Brew entries.
                optional --cleanup
            }

            /// Create symlinks from config/home to the home directory.
            cmd setup {}

            /// Remove stale symlinks in $HOME pointing into config/home.
            /// Only prints what would be removed unless --force is given.
            cmd clean {
                /// Actually remove the stale links.
                optional -f, --force
            }

            /// Open the config directory in the editor.
            cmd edit {
                /// Specify the editor to use; defaults to 'code'.
                optional -e, --editor editor: String
            }
        }
    }
}

fn main() -> Result<()> {
    let flags = flags::Config::from_env_or_exit();
    let sh = Shell::new()?;

    match flags.subcommand {
        flags::ConfigCmd::Edit(edit) => {
            let editor = edit.editor.unwrap_or_else(|| "code".to_string());
            let config_dir = get_config_dir(&sh)?;
            cmd!(sh, "{editor} {config_dir}")
                .run()
                .with_context(|| format!("open editor `{editor}`"))?;
        }
        flags::ConfigCmd::Bundle(bundle) => {
            let brewfile = get_config_dir(&sh)?.join("Brewfile");
            let cleanup_flag = bundle.cleanup.then_some("--cleanup");
            cmd!(sh, "brew bundle --file={brewfile} {cleanup_flag...}")
                .run()
                .context("run `brew bundle`")?;
        }
        flags::ConfigCmd::Setup(_) => symlink(&sh)?,
        flags::ConfigCmd::Clean(flags) => clean(&sh, flags.force)?,
    }

    Ok(())
}

fn get_home_dir(sh: &Shell) -> Result<PathBuf> {
    sh.var("HOME")
        .map(PathBuf::from)
        .context("get HOME environment variable")
}

fn get_config_dir(sh: &Shell) -> Result<PathBuf> {
    let config_dir = get_home_dir(sh)?.join("config");
    if !sh.path_exists(&config_dir) {
        bail!("config directory not found at {}", config_dir.display());
    }
    Ok(config_dir)
}

fn symlink(sh: &Shell) -> Result<()> {
    let home = get_home_dir(sh)?;
    let config_home = get_config_dir(sh)?.join("home");
    if !sh.path_exists(&config_home) {
        bail!(
            "config home directory not found at {}",
            config_home.display()
        );
    }

    walkdir(&config_home)?
        .into_iter()
        .try_for_each(|rel_path| {
            let entry = config_home.join(&rel_path);
            let dest = home.join(&rel_path);

            // create parent directory if needed
            if let Some(parent) = dest.parent() {
                sh.create_dir(parent)
                    .with_context(|| format!("create parent directory {}", parent.display()))?;
            }

            if dest.symlink_metadata().is_ok() {
                if dest.is_symlink() {
                    // replace existing symlinks outright
                    eprintln!(
                        "{RED}replacing existing symlink {}{RESET}",
                        rel_path.display()
                    );
                    // not sh.remove_path: it stats the link's target and
                    // treats NotFound as "already gone", so it would leave a
                    // dangling symlink in place and the symlink() below would
                    // fail with EEXIST
                    std::fs::remove_file(&dest)
                        .with_context(|| format!("remove existing symlink {}", dest.display()))?;
                } else {
                    // refuse to destroy real stuff
                    bail!(
                        "{} exists and is not a symlink; refusing to remove it",
                        dest.display()
                    );
                }
            }

            // create symlink
            eprintln!("{GREEN}creating symlink for {}{RESET}", rel_path.display());
            // no xshell helper exists for symlinks, so use std
            std::os::unix::fs::symlink(&entry, &dest)
                .with_context(|| format!("symlink {} to {}", entry.display(), dest.display()))
        })?;

    Ok(())
}

fn walkdir(path: &Path) -> Result<Vec<PathBuf>> {
    fn walk(dir: &Path, prefix: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        // std::fs rather than sh.read_dir, since we need each entry's file_type
        std::fs::read_dir(dir)
            .with_context(|| format!("read directory {}", dir.display()))?
            .try_for_each(|entry| -> Result<()> {
                let entry = entry.with_context(|| format!("read entry in {}", dir.display()))?;
                let path = entry.path();
                let file_type = entry
                    .file_type()
                    .with_context(|| format!("read file type of {}", path.display()))?;
                let rel = prefix.join(entry.file_name());

                if file_type.is_symlink() {
                    // `file_type()` uses d_type from readdir(), which
                    // categorises symlinks as symlinks rather than their
                    // target. Follow the symlink to classify the target.
                    if path.is_file() {
                        files.push(rel);
                    } else if path.is_dir() {
                        walk(&path, &rel, files)?;
                    } else {
                        bail!("broken symlink in config home: {}", path.display());
                    }
                } else if file_type.is_file() {
                    files.push(rel);
                } else if file_type.is_dir() {
                    walk(&path, &rel, files)?;
                } else {
                    // ignore other types
                }

                Ok(())
            })
    }

    let mut files = Vec::new();
    // yield paths relative to the root, so callers never need strip_prefix
    walk(path, Path::new(""), &mut files)?;
    Ok(files)
}

fn clean(sh: &Shell, force: bool) -> Result<()> {
    let home = get_home_dir(sh)?;
    let config_home = get_config_dir(sh)?.join("home");

    // `clean`'s scope rests on the convention that config/home only holds
    // dot-prefixed entries; anything else gets linked by `setup` but would
    // never be inspected here, so surface it.
    std::fs::read_dir(&config_home)
        .with_context(|| format!("read directory {}", config_home.display()))?
        .try_for_each(|entry| -> Result<()> {
            let entry =
                entry.with_context(|| format!("read entry in {}", config_home.display()))?;
            let name = entry.file_name();
            if !name.to_string_lossy().starts_with('.') {
                eprintln!(
                    "{RED}warning: config/home/{} does not start with '.'; \
                     stale links for it are outside clean's scope{RESET}",
                    name.to_string_lossy()
                );
            }
            Ok(())
        })?;

    let stale = find_stale_links(&home, &config_home)?;
    for link in &stale {
        if force {
            eprintln!("{RED}removing {}{RESET}", link.display());
            // not sh.remove_path: it stats the link's target and treats
            // NotFound as "already gone", so it can never remove a dangling
            // symlink — the exact case here
            std::fs::remove_file(link)
                .with_context(|| format!("remove stale symlink {}", link.display()))?;
        } else {
            eprintln!("{RED}would remove {}{RESET}", link.display());
        }
    }

    eprintln!("{} stale link(s) found", stale.len());
    if !force && !stale.is_empty() {
        eprintln!("re-run with --force to remove them");
    }
    // TODO: also prune parent directories left empty after removals.
    Ok(())
}

/// Collect symlinks under `home` that point into `config_home` and dangle.
///
/// The scan is scoped to depth-1 entries starting with '.', recursing into
/// real directories only. That covers every link `setup` can create (all of
/// config/home is dot-prefixed) while structurally skipping the likes of
/// ~/Library. Symlinked directories are never followed, so the scan cannot
/// escape this scope. A link counts as stale only when its target lies
/// inside config_home and no longer exists; foreign links are left alone,
/// which keeps the sweep through .cache and friends harmless.
///
/// The sweep crosses third-party territory, so unreadable directories
/// (e.g. TCC-protected ~/.Trash on macOS) are skipped with a warning rather
/// than aborting the run — a directory we cannot read cannot be cleaned
/// anyway. This is deliberately laxer than `walkdir`, which stays strict
/// because the config side is fully ours.
fn find_stale_links(home: &Path, config_home: &Path) -> Result<Vec<PathBuf>> {
    fn walk(
        dir: &Path,
        top_level: bool,
        config_home: &Path,
        stale: &mut Vec<PathBuf>,
    ) -> Result<()> {
        let mut entries = match std::fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!("{RED}warning: skipping {}: {}{RESET}", dir.display(), e);
                return Ok(());
            }
        };
        entries.try_for_each(|entry| -> Result<()> {
            let entry = entry.with_context(|| format!("read entry in {}", dir.display()))?;
            let path = entry.path();

            if top_level && !entry.file_name().to_string_lossy().starts_with('.') {
                return Ok(());
            }

            // `file_type()` does not follow symlinks, matching walkdir
            let file_type = entry
                .file_type()
                .with_context(|| format!("read file type of {}", path.display()))?;

            if file_type.is_symlink() {
                let target = std::fs::read_link(&path)
                    .with_context(|| format!("read symlink {}", path.display()))?;
                // relative targets resolve against the link's parent
                let target = if target.is_relative() {
                    dir.join(target)
                } else {
                    target
                };
                // do not canonicalize: that fails on dangling targets,
                // which are exactly what we are looking for. `starts_with`
                // is component-wise, so sibling prefixes can't match.
                let dangling = matches!(
                    std::fs::metadata(&target),
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound
                );
                if dangling && target.starts_with(config_home) {
                    stale.push(path);
                }
            } else if file_type.is_dir() {
                walk(&path, false, config_home, stale)?;
            }
            // ignore other types

            Ok(())
        })
    }

    let mut stale = Vec::new();
    walk(home, true, config_home, &mut stale)?;
    Ok(stale)
}
