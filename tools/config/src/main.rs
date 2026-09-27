use std::collections::BTreeMap;
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

            /// Link home files and copy missing seed files from config and config-private.
            cmd setup {}

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
            let cleanup_flag = bundle.cleanup.then_some("--force-cleanup");
            cmd!(sh, "brew bundle --file={brewfile} {cleanup_flag...}")
                .run()
                .context("run `brew bundle`")?;
        }
        flags::ConfigCmd::Setup(_) => setup(&sh, &get_home_dir(&sh)?)?,
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

fn setup(sh: &Shell, home: &Path) -> Result<()> {
    let config_home = home.join("config/home");
    let private = home.join("config-private");
    let mut sources = vec![(config_home, false)];
    if entry_exists(&private)? {
        if !private.is_dir() {
            bail!("private config is not a directory: {}", private.display());
        }
        let private_home = private.join("home");
        if entry_exists(&private_home)? {
            sources.push((private_home, false));
        }
    }
    for seed in [home.join("config/seed"), private.join("seed")] {
        if entry_exists(&seed)? {
            sources.push((seed, true));
        }
    }

    // validate source trees and destinations before creating any files
    let mut files = BTreeMap::new();
    for (source, seed) in sources {
        for relative in walkdir(&source)? {
            let entry = source.join(&relative);
            if let Some((previous, _)) = files.insert(relative.clone(), (entry.clone(), seed)) {
                bail!(
                    "conflicting sources for {}: {} and {}",
                    relative.display(),
                    previous.display(),
                    entry.display()
                );
            }
        }
    }
    for (relative, (entry, seed)) in &files {
        for ancestor in relative
            .ancestors()
            .skip(1)
            .filter(|p| !p.as_os_str().is_empty())
        {
            if files.contains_key(ancestor) {
                bail!(
                    "conflicting file and directory destinations: {} and {}",
                    ancestor.display(),
                    relative.display()
                );
            }
            let parent = home.join(ancestor);
            if entry_exists(&parent)? && !parent.is_dir() {
                bail!(
                    "destination parent is not a directory: {}",
                    parent.display()
                );
            }
        }
        let dest = home.join(relative);
        if !seed
            && dest.canonicalize().ok() != Some(entry.canonicalize()?)
            && entry_exists(&dest)?
            && !dest.is_symlink()
        {
            bail!(
                "{} exists and is not a symlink; refusing to remove it",
                dest.display()
            );
        }
    }

    files
        .into_iter()
        .try_for_each(|(rel_path, (entry, seed))| {
            let dest = home.join(&rel_path);

            // existing seed destinations belong to the machine
            if seed && entry_exists(&dest)? {
                return Ok(());
            }

            // create parent directory if needed
            if let Some(parent) = dest.parent() {
                sh.create_dir(parent)
                    .with_context(|| format!("create parent directory {}", parent.display()))?;
            }

            if seed {
                return seed_file(&entry, &dest);
            }

            // already linked, possibly through a symlinked parent directory
            if dest.canonicalize().ok() == Some(entry.canonicalize()?) {
                return Ok(());
            }

            if dest.symlink_metadata().is_ok() {
                if dest.is_symlink() {
                    // replace existing symlinks outright
                    eprintln!(
                        "{RED}replacing existing symlink {}{RESET}",
                        rel_path.display()
                    );
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

fn seed_file(source: &Path, dest: &Path) -> Result<()> {
    let mut input =
        std::fs::File::open(source).with_context(|| format!("open seed {}", source.display()))?;
    // create_new also protects files created after the existence check
    let mut output = match std::fs::File::create_new(dest) {
        Ok(file) => file,
        Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => return Ok(()),
        Err(err) => return Err(err).with_context(|| format!("create seed {}", dest.display())),
    };
    let result = (|| -> std::io::Result<()> {
        std::io::copy(&mut input, &mut output)?;
        output.set_permissions(input.metadata()?.permissions())
    })();
    if let Err(err) = result {
        // remove partial copies so the next setup can retry
        std::fs::remove_file(dest)
            .with_context(|| format!("remove partial seed {}", dest.display()))?;
        return Err(err).with_context(|| format!("copy seed to {}", dest.display()));
    }
    eprintln!("{GREEN}creating seed for {}{RESET}", dest.display());
    Ok(())
}

fn entry_exists(path: &Path) -> Result<bool> {
    match path.symlink_metadata() {
        Ok(_) => Ok(true),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(err).with_context(|| format!("inspect {}", path.display())),
    }
}

fn walkdir(path: &Path) -> Result<Vec<PathBuf>> {
    fn walk(dir: &Path, prefix: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        if !dir.is_dir() {
            bail!("config source is not a directory: {}", dir.display());
        }
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
