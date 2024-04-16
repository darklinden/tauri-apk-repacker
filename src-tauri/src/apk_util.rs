use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use crate::{
    constants::{
        get_apksigner_jar_path, get_apktool_jar_path, get_cache_folder, get_java_exe,
        get_key_store_path, get_vasdolly_jar_path, KEYSTORE_ALIAS_NAME, KEYSTORE_PWD,
    },
    run_command::run_command,
    xml_util::{xml_exchange_value, xml_find_value},
};

pub async fn unpack_apk(apk_file_path: &str) -> Result<PathBuf> {
    log::info!("unpack_apk");

    let java_exe = get_java_exe()?;
    log::info!("apk_file_path: {}", apk_file_path);

    let cache_folder = get_cache_folder();
    let original = cache_folder.join("original");
    if original.exists() {
        std::fs::remove_dir_all(&original)?;
    }

    // get exe folder
    let apktool_jar_path = get_apktool_jar_path();

    // exec command
    let _ = run_command(
        &java_exe,
        &[
            "-jar",
            "-Xms512m",
            "-Xmx1024m",
            apktool_jar_path.to_str().unwrap(),
            "--only-main-classes",
            "d",
            "-b",
            "-f",
            apk_file_path,
            "-o",
            original.to_str().unwrap(),
        ],
    )
    .await?;

    Ok(original)
}

pub async fn pack_apk(apk_folder: &Path) -> Result<PathBuf> {
    log::info!("pack_apk");

    let java_exe = get_java_exe()?;
    log::info!("apk_folder: {}", apk_folder.to_str().unwrap());

    let cache_folder = get_cache_folder();
    let apk_base_file_name = apk_folder.file_name().unwrap().to_str().unwrap();
    let original_apk = cache_folder.join(format!("{}.repacked.apk", apk_base_file_name));
    if original_apk.exists() {
        std::fs::remove_file(&original_apk)?;
    }

    // get exe folder
    let apktool_jar_path = get_apktool_jar_path();

    // exec command
    let _ = run_command(
        &java_exe,
        &[
            "-jar",
            "-Xms512m",
            "-Xmx1024m",
            apktool_jar_path.to_str().unwrap(),
            "--only-main-classes",
            "b",
            "-f",
            apk_folder.to_str().unwrap(),
            "-o",
            original_apk.to_str().unwrap(),
        ],
    )
    .await?;

    Ok(original_apk)
}

pub async fn sign_apk(apk_file_path: &Path) -> Result<()> {
    log::info!("sign_apk");

    // sign apk
    let key_store_path = get_key_store_path();
    let java_exe = get_java_exe()?;
    let apksigner_jar_path = get_apksigner_jar_path();
    let _ = run_command(
        &java_exe,
        &[
            "-jar",
            "-Xms512m",
            "-Xmx1024m",
            apksigner_jar_path.to_str().unwrap(),
            "--allowResign",
            "--overwrite",
            "-ks",
            key_store_path.to_str().unwrap(),
            "--ksPass",
            KEYSTORE_PWD,
            "--ksAlias",
            KEYSTORE_ALIAS_NAME,
            "--ksKeyPass",
            KEYSTORE_PWD,
            "-a",
            apk_file_path.to_str().unwrap(),
        ],
    )
    .await?;

    Ok(())
}

pub async fn get_apk_vasdolly_channel(apk_file: &str) -> Result<String> {
    log::info!("get_apk_vasdolly_channel");

    let java_exe = get_java_exe()?;
    let vasdolly_jar_path = get_vasdolly_jar_path();

    let output = run_command(
        &java_exe,
        &[
            "-jar",
            "-Xms512m",
            "-Xmx1024m",
            vasdolly_jar_path.to_str().unwrap(),
            "get",
            "-c",
            apk_file,
        ],
    )
    .await?;

    let channel_index = output.rfind("Channel: ").unwrap();
    let channel = String::from(&output[channel_index + 9..])
        .trim()
        .to_string();

    Ok(channel)
}

pub async fn set_apk_vasdolly_channel(apk_file: &Path, channel: &str) -> Result<()> {
    log::info!("set_apk_vasdolly_channel");

    let apk_file_path = apk_file.to_str().unwrap();
    let java_exe = get_java_exe()?;
    let vasdolly_jar_path = get_vasdolly_jar_path();

    // remove channel first
    let _ = run_command(
        &java_exe,
        &[
            "-jar",
            "-Xms512m",
            "-Xmx1024m",
            vasdolly_jar_path.to_str().unwrap(),
            "remove",
            "-c",
            apk_file_path,
        ],
    )
    .await?;

    let _ = run_command(
        &java_exe,
        &[
            "-jar",
            "-Xms512m",
            "-Xmx1024m",
            vasdolly_jar_path.to_str().unwrap(),
            "put",
            "-c",
            channel,
            "-f",
            apk_file_path,
            apk_file_path,
        ],
    )
    .await?;

    Ok(())
}

pub fn get_apk_package_name(apk_folder: &Path) -> Result<String> {
    let manifest_file_path = apk_folder.join("AndroidManifest.xml");
    let manifest_file_content = std::fs::read_to_string(manifest_file_path)?;

    let package_name = xml_find_value(&manifest_file_content, &["manifest"], "package")?;

    if package_name.is_empty() {
        return Err(anyhow::anyhow!("error find package name"));
    }

    Ok(package_name[0].to_owned())
}

pub fn get_apk_display_name(apk_folder: &Path) -> Result<String> {
    let manifest_file_path = apk_folder.join("AndroidManifest.xml");
    let manifest_file_content = std::fs::read_to_string(manifest_file_path)?;

    let display_name = xml_find_value(&manifest_file_content, &["application"], "android:label")?;
    if display_name.is_empty() {
        return Err(anyhow::anyhow!("error find display name"));
    }
    let mut display_name = display_name[0].to_owned();

    if display_name.starts_with("@string/") {
        let string_name = display_name[8..].to_string();
        let string_name = string_name.replace('@', "");
        let string_name = string_name.replace('/', "_");
        log::info!("string_name: {}", string_name);

        let mut values_dirs = vec![];
        let res_dir = std::fs::read_dir(apk_folder.join("res")).unwrap();
        for entry in res_dir {
            let entry = entry.unwrap();
            let path = entry.path();
            let folder_name = path.file_name().unwrap().to_string_lossy().to_string();
            // log::info!("path: {} {}", path.to_str().unwrap(), folder_name);
            if folder_name.starts_with("values") && path.is_dir() {
                values_dirs.push(path);
            }
        }

        if values_dirs.is_empty() {
            return Err(anyhow::anyhow!("error find values dirs"));
        }

        let mut str_display_name = "".to_string();

        let re_str = format!(r##"<string name="{}">(.+)</string>"##, string_name);
        let re = regex::Regex::new(re_str.as_str()).unwrap();

        for values_dir in values_dirs {
            let string_file_path = values_dir.join("strings.xml");
            if !string_file_path.exists() {
                continue;
            }

            let string_file_content = std::fs::read_to_string(&string_file_path);
            if string_file_content.is_err() {
                continue;
            }

            log::info!("string_file_path: {}", string_file_path.to_str().unwrap());

            let string_file_content = string_file_content.unwrap();
            for (_, [name]) in re.captures_iter(&string_file_content).map(|c| c.extract()) {
                log::info!("found name {}", name);
                str_display_name.push_str(name);
                str_display_name.push(',');
            }
        }

        if str_display_name.is_empty() {
            return Ok("error find display name".to_string());
        }

        display_name = str_display_name.trim_end_matches(',').to_string();
        log::info!("display_name: {}", display_name);
    }

    Ok(display_name)
}

pub fn get_apk_icon_names(apk_folder: &Path) -> Result<Vec<String>> {
    let mut name_list: Vec<String> = vec![];

    // read manifest android:icon
    let manifest_file_path = apk_folder.join("AndroidManifest.xml");
    let manifest_file_content = std::fs::read_to_string(manifest_file_path)?;

    let mut manifest_icon_names =
        xml_find_value(&manifest_file_content, &["application"], "android:icon")?
            .iter()
            .map(|icon_name| {
                log::info!("application android:icon: {}", icon_name);
                let icon_name_index = icon_name.rfind('/').unwrap();
                let icon_name = format!("{}.png", &icon_name[icon_name_index + 1..]);
                icon_name
            })
            .collect::<Vec<String>>();

    name_list.append(&mut manifest_icon_names);

    // read res/mipmap-anydpi-v26 foreground android:drawable
    let v26_folder = apk_folder.join("res").join("mipmap-anydpi-v26");
    if v26_folder.exists() {
        let v26_dir = std::fs::read_dir(v26_folder)?;
        for entry in v26_dir {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let file_name = path.file_name().unwrap().to_str().unwrap();
                if file_name.starts_with("ic_launcher") {
                    let xml_content = std::fs::read_to_string(&path)?;

                    let mut foreground_icon_names =
                        xml_find_value(&xml_content, &["foreground"], "android:drawable")?
                            .iter()
                            .map(|icon_name| {
                                log::info!("foreground android:drawable: {}", icon_name);
                                let icon_name_index = icon_name.rfind('/').unwrap();
                                let icon_name =
                                    format!("{}.png", &icon_name[icon_name_index + 1..]);
                                icon_name
                            })
                            .collect::<Vec<String>>();

                    name_list.append(&mut foreground_icon_names);
                }
            }
        }
    }

    // unique
    Ok(name_list
        .into_iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApkInfo {
    pub package_name: String,
    pub display_name: String,
    pub icon_path: String,
}

pub async fn get_apk_info(apk_folder: &Path) -> Result<ApkInfo> {
    let package_name = get_apk_package_name(apk_folder)?;
    let display_name = get_apk_display_name(apk_folder)?;

    let icon_names = get_apk_icon_names(apk_folder)?;

    // list dir of res
    let res_path = apk_folder.join("res");
    let res_dir = std::fs::read_dir(res_path).unwrap();
    let mut max_w: u32 = 0;
    let mut max_icon_path = "".to_string();
    for entry in res_dir {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            for icon_name in &icon_names {
                let icon_path = path.join(icon_name);
                log::info!("icon_path: {}", icon_path.to_str().unwrap());
                if icon_path.exists() {
                    let old_image = image::open(&icon_path).unwrap();
                    let w = old_image.width();
                    if w > max_w {
                        max_w = w;
                        max_icon_path = icon_path.to_str().unwrap().to_string();
                    }
                }
            }
        }
    }

    Ok(ApkInfo {
        package_name,
        display_name,
        icon_path: max_icon_path,
    })
}

pub fn exchange_apk_package_name(apk_folder: &Path, new_package_name: &str) -> Result<()> {
    let manifest_file_path = apk_folder.join("AndroidManifest.xml");
    let manifest_file_content = std::fs::read_to_string(&manifest_file_path)?;

    let result = xml_exchange_value(
        &manifest_file_content,
        &["manifest"],
        "package",
        new_package_name,
    )?;

    std::fs::write(manifest_file_path, result)?;

    Ok(())
}

pub fn exchange_apk_display_name(apk_folder: &Path, new_display_name: &str) -> Result<()> {
    let manifest_file_path = apk_folder.join("AndroidManifest.xml");
    let manifest_file_content = std::fs::read_to_string(&manifest_file_path)?;

    let result = xml_exchange_value(
        &manifest_file_content,
        &["application"],
        "android:label",
        new_display_name,
    )?;

    std::fs::write(manifest_file_path, result)?;

    Ok(())
}

pub fn exchange_apk_icon(apk_folder: &Path, new_icon_path: &str) -> Result<()> {
    let icon_names = get_apk_icon_names(apk_folder)?;

    let icon_image = image::open(new_icon_path).unwrap();

    // list dir of res
    let res_path = apk_folder.join("res");
    let res_dir = std::fs::read_dir(res_path).unwrap();
    for entry in res_dir {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            for icon_name in &icon_names {
                let icon_path = path.join(icon_name);
                log::info!("icon_path: {}", icon_path.to_str().unwrap());
                if icon_path.exists() {
                    let old_image = image::open(&icon_path).unwrap();
                    let w = old_image.width();
                    let h = old_image.height();
                    std::fs::remove_file(&icon_path).unwrap();

                    let new_image =
                        icon_image.resize_exact(w, h, image::imageops::FilterType::Nearest);
                    new_image.save(&icon_path).unwrap();
                }
            }
        }
    }

    Ok(())
}
