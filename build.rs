/*
use std::{
    ops::Deref,
    path::{Path, PathBuf},
};

use async_zip::{base::read::mem::ZipFileReader, StoredZipEntry};
use reqwest::Response;
use tar::Archive;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let _ = color_eyre::install();

    let platform = match std::env::var("CARGO_CFG_TARGET_OS")?.as_str() {
        "linux" => "ubuntu",
        "windows" => "windows",
        "android" => "android",
        "macos" => "macos",
        _ => todo!(),
    };

    let arch = match std::env::var("CARGO_CFG_TARGET_ARCH")?.as_str() {
        "x86_64" => "x86-64",
        "armv7" => "armv7",
        _ => todo!(),
    };

    let feature_set = std::env::var("CARGO_CFG_TARGET_FEATURE")?;
    let feature_set = feature_set.split(',').collect::<Vec<_>>();

    let features = match arch {
        "x86_64" => {
            if feature_set.contains(&"avx2") {
                "-avx2"
            } else if feature_set.contains(&"sse4.1") {
                "-modern"
            } else {
                ""
            }
        }
        "armv7" => {
            if feature_set.contains(&"neon") {
                "-neon"
            } else {
                ""
            }
        }
        _ => "",
    };

    let asset_tag = format!("stockfish-{platform}-{arch}{features}.");
    let out_path = PathBuf::from(std::env::var("OUT_DIR")?);

    if std::fs::exists(dirs::cache_dir().unwrap().join())

    let octocrab = octocrab::instance();
    let latest_release = octocrab
        .repos("official-stockfish", "Stockfish")
        .releases()
        .get_latest()
        .await?;

    for asset in latest_release.assets {
        if asset.name.starts_with(&asset_tag) {
            let resp = reqwest::get(asset.browser_download_url).await?;
            let binary_path = if asset.name.ends_with(".zip") {
                read_from_zip(resp, &out_path).await?
            } else {
                read_from_tar(resp, &asset_tag[..asset_tag.len() - 1], &out_path).await?
            };

            println!(
                "cargo:rustc-env=STOCKFISH_PATH={}",
                binary_path.canonicalize()?.display()
            );
            return Ok(());
        }
    }

    return Err(color_eyre::Report::msg("No sutable release found"));
}

async fn read_from_tar(
    resp: Response,
    asset_name: &str,
    out_path: &Path,
) -> color_eyre::Result<PathBuf> {
    let bytes = resp.bytes().await?;
    let mut ar = Archive::new(bytes.deref());

    let mut entries = ar.entries()?;
    while let Some(mut entry) = entries.next().transpose()? {
        let path = entry.path()?.into_owned();
        if path.file_name().is_some_and(|x| x == asset_name) {
            entry.unpack_in(out_path)?;
            return Ok(out_path.join(path));
        }
    }

    return Err(color_eyre::Report::msg("No valid entry found"));
}

async fn read_from_zip(resp: Response, out_path: &Path) -> color_eyre::Result<PathBuf> {
    let bytes = resp.bytes().await?;
    for entry in ZipFileReader::new(Vec::from(bytes))
        .await?
        .file()
        .entries()
        .iter()
        .map(StoredZipEntry::entry)
    {
        panic!("{entry:#?}")
    }

    return Err(color_eyre::Report::msg("No valid entry found"));
}
*/

pub fn main() {}
