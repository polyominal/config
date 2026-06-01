use std::path::PathBuf;

use anyhow::{Result, anyhow, bail};
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

            /// Open the config directory in the editor.
            cmd edit {
                /// Specify the editor to use; defaults to 'code'.
                optional -e, --editor editor: String
            }
        }
    }
}

fn main() -> Result<()> {
    let flags = flags::Config::from_env()?;
    let sh = Shell::new()?;

    match flags.subcommand {
        flags::ConfigCmd::Edit(edit) => {
            let editor = edit.editor.unwrap_or_else(|| "code".to_string());
            let config_dir = get_config_dir(&sh)?;
            cmd!(sh, "{editor} {config_dir}").run()?;
        }
        flags::ConfigCmd::Bundle(bundle) => {
            let brewfile = get_config_dir(&sh)?.join("Brewfile");
            let cleanup_flag = bundle.cleanup.then_some("--cleanup");
            cmd!(sh, "brew bundle --file={brewfile} {cleanup_flag...}").run()?;
        }
        flags::ConfigCmd::Setup(_) => symlink(&sh)?,
    }

    Ok(())
}

fn get_home_dir(sh: &Shell) -> Result<PathBuf> {
    sh.var("HOME")
        .map(PathBuf::from)
        .map_err(|e| anyhow::anyhow!("failed to get HOME directory from env: {e}"))
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
    if !config_home.exists() {
        bail!(
            "config home directory not found at {}",
            config_home.display()
        );
    }

    walkdir(&config_home)?.into_iter().try_for_each(|entry| {
        let rel_path = entry.strip_prefix(&config_home).unwrap_or_else(|e| {
            panic!(
                "failed to determine relative path for {}: {e}",
                entry.display()
            )
        });
        let dest = home.join(rel_path);

        // create parent directory if needed
        if let Some(parent) = dest.parent() {
            sh.create_dir(parent)?;
        }

        // remove existing file/symlink if exists
        if dest.exists() {
            eprintln!("{RED}removing existing {}{RESET}", rel_path.display());
            sh.remove_path(&dest)?;
        }

        // create symlink
        eprintln!("{GREEN}creating symlink for {}{RESET}", rel_path.display());
        std::os::unix::fs::symlink(&entry, &dest).map_err(|e| {
            anyhow!(
                "failed to symlink {} to {}: {e}",
                entry.display(),
                dest.display()
            )
        })
    })?;

    Ok(())
}

fn walkdir(path: &PathBuf) -> Result<Vec<PathBuf>> {
    fn walk(dir: &PathBuf, files: &mut Vec<PathBuf>) -> Result<()> {
        std::fs::read_dir(dir)
            .map_err(|e| anyhow!("failed to read directory {}: {e}", dir.display()))?
            .try_for_each(|entry| -> Result<()> {
                let entry = entry?;
                let path = entry.path();
                let file_type = entry.file_type()?;

                if file_type.is_symlink() {
                    // `file_type()` uses d_type from readdir(), which
                    // categorises symlinks as symlinks rather than their
                    // target. Follow the symlink to classify the target.
                    if path.is_file() {
                        files.push(path);
                    } else if path.is_dir() {
                        walk(&path, files)?;
                    }
                } else if file_type.is_file() {
                    files.push(path);
                } else if file_type.is_dir() {
                    walk(&path, files)?;
                } else {
                    // ignore other types
                }

                Ok(())
            })
    }

    let mut files = Vec::new();
    walk(path, &mut files)?;
    Ok(files)
}
