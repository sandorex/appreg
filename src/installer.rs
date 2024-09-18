use std::fs;
use std::{os::unix::fs::PermissionsExt, path::PathBuf};
use std::path::Path;
use std::process::Command;
use anyhow::{anyhow, Context, Result};
use super::cli::CmdInstallArgs;

#[derive(PartialEq, Eq, Default, Debug)]
pub struct InstallInfo {
    pub exec: PathBuf,
    pub icon: PathBuf,
    pub desktop_file: Option<PathBuf>,
    pub env: Option<Vec<String>>,
    pub args: Option<Vec<String>>,
}

fn is_appimage(file: &Path) -> Result<bool> {
    use std::fs::File;
    use std::os::unix::fs::FileExt;

    let f = File::open(file).with_context(|| anyhow!("Failed to open file {:?}", file))?;
    let mut buffer = [0u8; 4];
    f.read_exact_at(&mut buffer, 8)?;

    // NOTE: the magic number is 3 bytes long
    let number = u32::from_be_bytes(buffer) >> 8;

    // appimage magic number for type 2
    Ok(number == 0x414902)
}

// fn is_file_executable(file: &str) -> Result<bool, Box<dyn Error>> {
//     let perm = std::path::Path::new(file).metadata()?.permissions();
//     Ok(perm.mode() & 0o111 != 0)
// }

// fn generate_desktop_file() -> String {
//     "".to_string()
// }
//
// fn install_from_config(dry_run: bool, path: String) {
//     todo!()
// }
//
// fn install_from_appimage(dry_run: bool, path: String) {
//     todo!()
// }
//
// /// Install for an AppDir, may contain desktop files and icons
// fn install_from_appdir(dry_run: bool, path: String) {
//     todo!()
// }
//
// /// Install for a binary, so its probably an app so possibly have an icon on internet?
// fn install_from_binary(dry_run: bool, path: String) {
//     todo!()
// }
//
// /// Install for a generic executable (its probably a script)
// fn install_from_generic_executable(dry_run: bool, path: &str) {
//     println!("got:\n{}", format!(r#"[Desktop Entry]
// Name={}
// Type=Application
// Exec={} %u"#, "aa", "/usr/bin/firefox"));
// }

pub fn install_appimage(file: &Path) -> Result<InstallInfo> {
    let dir = match file.parent() {
        Some(x) if x.file_name().is_none() => Path::new("."),
        Some(x) => x,
        None => Path::new("."),
    };
    dbg!(&dir);

    // make sure the file is executable
    {
        let mut perm = file.metadata()
            .with_context(|| format!("Failed to get permission for file {:?}", file))?
            .permissions();

        if perm.mode() & 0o111 != 0 {
            perm.set_mode(perm.mode() | 0o111);
            std::fs::set_permissions(file, perm)
                .with_context(|| format!("Failed to set permission for file {:?}", file))?;
        }
    }

    let extracted_path = {
        // remove extension and use the rest as the dir name
        let mut dir_path = dir.join(file.to_path_buf());
        dir_path.set_extension("");
        dir_path
    };

    // // extract app image
    // let cmd = Command::new(path)
    //     .arg("--appimage-extract")
    //     .output()
    //     .expect("Could not execute appimage");
    //
    // if !cmd.status.success() {
    //     eprintln!("Error while extracting appimage, is it a valid appimage?");
    //     return Ok(1);
    // }

    // rename squashfs-root
    // std::fs::rename("squashfs-root", &dir_path)?;

    // NOTE: removed for testing
    // remove original appimage
    // std::fs::remove_file(path)?;

    // find icon using the link
    let icon_link = extracted_path.join(".DirIcon");
    dbg!(&icon_link);
    let icon_path = match Path::new(&icon_link).read_link() {
        Ok(x) => {
            let icon_path = extracted_path.join(x);
            if !icon_path.exists() {
                return Err(anyhow!("Invalid appimage, icon {:?} does not exist", icon_path));
            }

            icon_path
        },
        Err(_) => return Err(anyhow!("Invalid appimage, '.DirIcon' does not exist")),
    };

    // the script to execute the app
    let apprun_path = {
        let apprun_path = extracted_path.join("AppRun");
        if !apprun_path.exists() {
            return Err(anyhow!("Invalid appimage, 'AppRun' does not exist"));
        }

        apprun_path
    };

    // find the desktop file, according to spec there must be only one
    let desktop_file_path = {
        let mut desktop_file_path: Option<PathBuf> = None;
        for entry in extracted_path.read_dir()
            .with_context(|| anyhow!("Unable to read directory {:?}", extracted_path))?
            .flatten() {
            // ignore all non-file entries
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_file() {
                    let file_name_raw = entry.file_name();
                    let file_name = file_name_raw.to_string_lossy();
                    if file_name.ends_with(".desktop") {
                        desktop_file_path = Some(entry.path());
                        break
                    }
                }
            }
        }

        match desktop_file_path {
            // if symlink get path of the real file
            Some(x) if x.is_symlink() => {
                let link_target = x.read_link()
                    .with_context(|| format!("Invalid appimage, failed to read desktop file link {:?}", x))?;

                if link_target.is_file() {
                    link_target
                } else {
                    return Err(anyhow!("Invalid appimage, desktop file link is invalid"));
                }
            },
            Some(x) => x,
            None => return Err(anyhow!("Invalid appimage, no desktop files found")),
        }
    };

    {
        let i = ini::Ini::load_from_file("./steam.desktop").unwrap();
        // i.section(name)
        // let i = ini::Ini::load_from_file(&desktop_file_path).unwrap();
        for (sec, prop) in i.iter() {
            println!("Section: {:?}", sec);
            for (k, v) in prop.iter() {
                println!("{}:{}", k, v);
            }
        }
    }
    // TODO copy icon to ~/.local/share/icons and make sure to change the name
    // TODO
    // 1. copy the image
    // 2. read desktop file and modify it
    //      2.1. modify exec
    //      2.2. modify icon path

    Ok(InstallInfo {
        icon: icon_path,
        exec: apprun_path,
        desktop_file: Some(desktop_file_path),
        ..Default::default()
    })
}

pub fn cmd_install(dry_run: bool, cli_args: CmdInstallArgs) -> Result<u8> {
    // if there is a yaml file load it and it should point to proper files
    //
    // no yaml file:
    //  appimage:
    //   1. extract .desktop, and icon if possible
    //   1b. if not then try finding icon on internet by name (in future versions)
    //

    // TODO test if path exists
    let path = Path::new(&cli_args.file);
    if !path.exists() {
        return Err(anyhow!("File {:?} does not exist", path));
    }

    if path.is_dir() {
        return Err(anyhow!("Path {:?} is a directory not a file", path));
    }

    if is_appimage(path)? {
        println!("Installing appimage {:?}", path);
        let info = install_appimage(path)?;
        println!("got: {:#?}", info);
    } else {
        return Err(anyhow!("File {:?} cannot be installed", path))
    }

    Ok(0)
}
