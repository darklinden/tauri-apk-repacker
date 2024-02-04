// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use std::io::Cursor;
use std::io::Write;
use tauri_plugin_log::LogTarget;

const APKTOOL_JAR_BYTES: &[u8] = include_bytes!("../apktool.jar");
const VASDOLLY_JAR_BYTES: &[u8] = include_bytes!("../vasdolly.jar");
const KEYSTORE_BYTES: &[u8] = include_bytes!("../key.keystore");
const KEYSTORE_PWD: &str = "123456";
const KEYSTORE_ALIAS_NAME: &str = "key";

#[tauri::command]
async fn extract_tools() -> Result<String, ()> {
    log::info!("extract_tools");

    // get exe folder
    let exe_path = std::env::current_exe().unwrap();
    let exe_folder = exe_path.parent().unwrap();
    let apktool_jar_path = exe_folder.join("apktool.jar");

    // check if apktool.jar exists
    if !apktool_jar_path.exists() {
        // extract apktool.jar
        let mut apktool_jar_file = std::fs::File::create(apktool_jar_path.clone()).unwrap();
        apktool_jar_file.write_all(APKTOOL_JAR_BYTES).unwrap();
        apktool_jar_file.flush().unwrap();
        log::info!("apktool.jar extracted");
    }

    // check if vasdolly.jar exists
    let vasdolly_jar_path = exe_folder.join("vasdolly.jar");

    if !vasdolly_jar_path.exists() {
        // extract vasdolly.jar
        let mut vasdolly_jar_file = std::fs::File::create(vasdolly_jar_path.clone()).unwrap();
        vasdolly_jar_file.write_all(VASDOLLY_JAR_BYTES).unwrap();
        vasdolly_jar_file.flush().unwrap();
        log::info!("vasdolly.jar extracted");
    }

    // check if key.keystore exists
    let key_store_path = exe_folder.join("key.keystore");

    if !key_store_path.exists() {
        // extract key.keystore
        let mut key_store_file = std::fs::File::create(key_store_path.clone()).unwrap();
        key_store_file.write_all(KEYSTORE_BYTES).unwrap();
        key_store_file.flush().unwrap();
        log::info!("key.keystore extracted");
    }

    let result = if apktool_jar_path.exists() && key_store_path.exists() {
        "Ok"
    } else {
        "Error"
    };
    Ok(String::from(result))
}

#[tauri::command]
fn get_env(name: &str) -> String {
    std::env::var(String::from(name)).unwrap_or(String::from(""))
}

#[tauri::command]
fn set_env(name: &str, value: &str) {
    std::env::set_var(String::from(name), String::from(value));
}

fn try_parse_output(output: std::process::Output) -> Result<String, String> {
    if !output.stderr.is_empty() {
        let err_str = String::from_utf8_lossy(&output.stderr);
        return Err(err_str.to_string());
    }

    let out_str = String::from_utf8_lossy(&output.stdout);
    log::info!("output: {}", out_str);

    if output.status.success() {
        return Ok(out_str.to_string());
    } else {
        return Err("unknown error".to_string());
    }
}

#[tauri::command]
async fn change_content_and_repack_apk(
    javapath: String,
    apkfilepath: String,
    apkpackagename: String,
    apkdisplayname: String,
    apkiconfilepath: String,
) -> Result<String, ()> {
    log::info!("change_content_and_repack_apk");
    let java_path = std::path::Path::new(&javapath).join("bin").join("java");
    let java_path = java_path.to_str().unwrap();
    let java_path = if cfg!(target_os = "windows") {
        format!("{}.exe", java_path)
    } else {
        java_path.to_string()
    };

    log::info!("java_path: {}", java_path);
    let jarsigner_path = std::path::Path::new(&javapath)
        .join("bin")
        .join("jarsigner");
    let jarsigner_path = jarsigner_path.to_str().unwrap();
    let jarsigner_path = if cfg!(target_os = "windows") {
        format!("{}.exe", jarsigner_path)
    } else {
        jarsigner_path.to_string()
    };

    log::info!("jarsigner_path: {}", jarsigner_path);
    let apk_file_path = apkfilepath.clone();
    log::info!("apk_file_path: {}", apk_file_path);
    let apk_package_name = apkpackagename.clone();
    log::info!("apk_package_name: {}", apk_package_name);
    let apk_display_name = apkdisplayname.clone();
    log::info!("apk_display_name: {}", apk_display_name);
    let apk_icon_file_path = apkiconfilepath.clone();
    log::info!("apk_icon_file_path: {}", apk_icon_file_path);

    let temp_dir = apk_file_path.to_string() + ".tmp";
    let temp_dir = std::path::Path::new(&temp_dir);
    if temp_dir.exists() {
        std::fs::remove_dir_all(temp_dir).unwrap();
    }
    std::fs::create_dir(temp_dir).unwrap();
    log::info!("temp_dir: {}", temp_dir.to_str().unwrap());

    let original = std::path::Path::new(temp_dir).join("original");

    // get exe folder
    let exe_path = std::env::current_exe().unwrap();
    let exe_folder = exe_path.parent().unwrap();
    let apktool_jar_path = exe_folder.join("apktool.jar");

    // exec command
    log::info!(
        "run {} -jar -Xms512m -Xmx1024m {} --only-main-classes d -b -f {} -o {}",
        java_path,
        apktool_jar_path.to_str().unwrap(),
        apk_file_path,
        original.to_str().unwrap()
    );
    let output = std::process::Command::new(&java_path)
        .args(&[
            "-jar",
            "-Xms512m",
            "-Xmx1024m",
            apktool_jar_path.to_str().unwrap(),
            "--only-main-classes",
            "d",
            "-b",
            "-f",
            apk_file_path.as_str(),
            "-o",
            original.to_str().unwrap(),
        ])
        .output()
        .expect("failed to execute process");

    let output_result = try_parse_output(output);
    if output_result.is_err() {
        return Ok(output_result.err().unwrap());
    }

    let manifest_file_path = original.join("AndroidManifest.xml");
    let manifest_file_content = std::fs::read_to_string(manifest_file_path.clone()).unwrap();

    log::info!("src manifest_file_content: {}", manifest_file_content);

    let manifest_file_content =
        manifest_exchange_package_name(manifest_file_content, apk_package_name.clone());

    log::info!(
        "package name changed manifest_file_content: {}",
        manifest_file_content
    );

    let manifest_file_content =
        manifest_exchange_display_name(manifest_file_content, apk_display_name.clone());

    log::info!(
        "display name changed manifest_file_content: {}",
        manifest_file_content
    );

    std::fs::write(manifest_file_path, &manifest_file_content).unwrap();

    let icon_name = read_apk_launch_icon_name(&manifest_file_content);
    let icon_name_index = icon_name.rfind("/").unwrap();
    let icon_name = format!("{}.png", &icon_name[icon_name_index + 1..]);
    log::info!("icon_name: {}", icon_name);

    // remove mipmap-anydpi-v26
    let mipmap_anydpi_v26_path = original.join("res").join("mipmap-anydpi-v26");
    if mipmap_anydpi_v26_path.exists() {
        std::fs::remove_dir_all(mipmap_anydpi_v26_path).unwrap();
    }

    let icon_image = image::open(&apk_icon_file_path).unwrap();

    // list dir of res
    let res_path = original.join("res");
    let res_dir = std::fs::read_dir(res_path).unwrap();
    for entry in res_dir {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            let icon_path = path.join(&icon_name);
            log::info!("icon_path: {}", icon_path.to_str().unwrap());
            if icon_path.exists() {
                let old_image = image::open(&icon_path).unwrap();
                let w = old_image.width();
                let h = old_image.height();
                std::fs::remove_file(&icon_path).unwrap();

                let new_image = icon_image.resize_exact(w, h, image::imageops::FilterType::Nearest);
                new_image.save(&icon_path).unwrap();
            }
        }
    }

    // repack apk
    let des_apk_file_path = format!("{}.repacked.apk", apk_file_path);

    log::info!(
        "run {} -jar -Xms512m -Xmx1024m {} --only-main-classes b -f {} -o {}",
        java_path,
        apktool_jar_path.to_str().unwrap(),
        original.to_str().unwrap(),
        des_apk_file_path
    );
    let output = std::process::Command::new(&java_path)
        .args(&[
            "-jar",
            "-Xms512m",
            "-Xmx1024m",
            apktool_jar_path.to_str().unwrap(),
            "--only-main-classes",
            "b",
            "-f",
            original.to_str().unwrap(),
            "-o",
            des_apk_file_path.as_str(),
        ])
        .output()
        .expect("failed to execute process");

    let output_result = try_parse_output(output);
    if output_result.is_err() {
        return Ok(output_result.err().unwrap());
    }

    // sign apk
    let key_store_path = exe_folder.join("key.keystore");
    log::info!(
            "run {} -verbose -digestalg SHA1 -sigalg MD5withRSA -keystore {} -storepass {} -keypass {} -signedjar {} {} {}",
            jarsigner_path,
            key_store_path.to_str().unwrap(),
            KEYSTORE_PWD,
            KEYSTORE_PWD,
            des_apk_file_path,
            des_apk_file_path,
            KEYSTORE_ALIAS_NAME,
        );
    let output = std::process::Command::new(&jarsigner_path)
        .args(&[
            "-verbose",
            "-digestalg",
            "SHA1",
            "-sigalg",
            "MD5withRSA",
            "-keystore",
            key_store_path.to_str().unwrap(),
            "-storepass",
            KEYSTORE_PWD,
            "-keypass",
            KEYSTORE_PWD,
            "-signedjar",
            des_apk_file_path.as_str(),
            des_apk_file_path.as_str(),
            KEYSTORE_ALIAS_NAME,
        ])
        .output()
        .expect("failed to execute process");

    let output_result = try_parse_output(output);
    if output_result.is_err() {
        return Ok(output_result.err().unwrap());
    }

    // copy channel if exist
    let vasdolly_jar_path = exe_folder.join("vasdolly.jar");

    log::info!(
        "run {} -jar {} get -c {}",
        java_path,
        vasdolly_jar_path.to_str().unwrap(),
        apk_file_path,
    );
    let output = std::process::Command::new(&java_path)
        .args(&[
            "-jar",
            vasdolly_jar_path.to_str().unwrap(),
            "get",
            "-c",
            apk_file_path.as_str(),
        ])
        .output()
        .expect("failed to execute process");

    let output_result = try_parse_output(output);
    if output_result.is_err() {
        return Ok(output_result.err().unwrap());
    }

    let output = output_result.unwrap();
    let channel_index = output.rfind("Channel: ").unwrap();
    let channel = String::from(&output[channel_index + 9..])
        .trim()
        .to_string();

    log::info!("channel: {}", channel);

    if channel == "" || channel == "null" {
        log::info!("channel not found");
    } else {
        // copy channel to new apk
        log::info!(
            "run {} -jar {} put -c '{}' -f {} {}",
            java_path,
            vasdolly_jar_path.to_str().unwrap(),
            channel,
            des_apk_file_path,
            des_apk_file_path,
        );

        let output = std::process::Command::new(&java_path)
            .args(&[
                "-jar",
                vasdolly_jar_path.to_str().unwrap(),
                "put",
                "-c",
                channel.as_str(),
                "-f",
                des_apk_file_path.as_str(),
                des_apk_file_path.as_str(),
            ])
            .output()
            .expect("failed to execute process");

        let output_result = try_parse_output(output);
        if output_result.is_err() {
            return Ok(output_result.err().unwrap());
        }
    }

    // remove temp dir
    std::fs::remove_dir_all(temp_dir).unwrap();

    Ok(String::from("success"))
}

fn manifest_exchange_package_name(manifest_content: String, new_package_name: String) -> String {
    let mut reader = Reader::from_str(manifest_content.as_str());
    reader.trim_text(true);
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    let mut old_package_name: String = "".to_string();
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                // exchange package name
                if e.name().as_ref() == b"manifest" {
                    // crates a new element ... alternatively we could reuse `e` by calling
                    // `e.into_owned()`
                    let mut elem = BytesStart::new("manifest");

                    old_package_name = String::from_utf8(
                        e.attributes()
                            .map(|attr| attr.unwrap())
                            .find(|attr| attr.key.as_ref() == b"package")
                            .unwrap()
                            .value
                            .to_vec(),
                    )
                    .unwrap();

                    log::info!("old_package_name: {}", old_package_name);

                    // collect existing attributes except for the "package" one
                    elem.extend_attributes(
                        e.attributes()
                            .map(|attr| attr.unwrap())
                            .filter(|attr| attr.key.as_ref() != b"package"),
                    );

                    // copy existing attributes, adds a new my-key="some value" attribute
                    elem.push_attribute(("package", new_package_name.as_str()));

                    // writes the event to the writer
                    assert!(writer.write_event(Event::Start(elem)).is_ok());
                } else if e.name().as_ref() == b"provider" {
                    // crates a new element ... alternatively we could reuse `e` by calling
                    // `e.into_owned()`
                    let mut elem = BytesStart::new("provider");

                    let mut authority = String::from_utf8(
                        e.attributes()
                            .map(|attr| attr.unwrap())
                            .find(|attr| attr.key.as_ref() == b"android:authorities")
                            .unwrap()
                            .value
                            .to_vec(),
                    )
                    .unwrap();

                    log::info!("old authority: {}", authority);

                    authority =
                        authority.replace(old_package_name.as_str(), new_package_name.as_str());

                    // collect existing attributes
                    elem.extend_attributes(
                        e.attributes()
                            .map(|attr| attr.unwrap())
                            .filter(|attr| attr.key.as_ref() != b"android:authorities"),
                    );

                    // copy existing attributes, adds a new my-key="some value" attribute
                    elem.push_attribute(("android:authorities", authority.as_str()));

                    // writes the event to the writer
                    assert!(writer.write_event(Event::Start(elem)).is_ok());
                } else {
                    // writes the event to the writer as is
                    assert!(writer.write_event(Event::Start(e)).is_ok());
                }
            }
            Ok(Event::End(e)) => {
                assert!(writer.write_event(Event::End(e)).is_ok());
            }
            Ok(Event::Eof) => break,
            // we can either move or borrow the event to write, depending on your use-case
            Ok(e) => assert!(writer.write_event(e).is_ok()),
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
        }
    }

    String::from_utf8(writer.into_inner().into_inner()).unwrap()
}

fn manifest_exchange_display_name(manifest_content: String, new_display_name: String) -> String {
    let mut reader = Reader::from_str(manifest_content.as_str());
    reader.trim_text(true);
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                // exchange package name
                if e.name().as_ref() == b"application" {
                    // crates a new element ... alternatively we could reuse `e` by calling
                    // `e.into_owned()`
                    let mut elem = BytesStart::new("application");

                    let old_display_name = String::from_utf8(
                        e.attributes()
                            .map(|attr| attr.unwrap())
                            .find(|attr| attr.key.as_ref() == b"android:label")
                            .unwrap()
                            .value
                            .to_vec(),
                    )
                    .unwrap();

                    log::info!("old_display_name: {}", old_display_name);

                    // collect existing attributes except for the "package" one
                    elem.extend_attributes(
                        e.attributes()
                            .map(|attr| attr.unwrap())
                            .filter(|attr| attr.key.as_ref() != b"android:label"),
                    );

                    // copy existing attributes, adds a new my-key="some value" attribute
                    elem.push_attribute(("android:label", new_display_name.as_str()));

                    // writes the event to the writer
                    assert!(writer.write_event(Event::Start(elem)).is_ok());
                } else {
                    // writes the event to the writer as is
                    assert!(writer.write_event(Event::Start(e)).is_ok());
                }
            }
            Ok(Event::End(e)) => {
                assert!(writer.write_event(Event::End(e)).is_ok());
            }
            Ok(Event::Eof) => break,
            // we can either move or borrow the event to write, depending on your use-case
            Ok(e) => assert!(writer.write_event(e).is_ok()),
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
        }
    }

    String::from_utf8(writer.into_inner().into_inner()).unwrap()
}

fn read_apk_launch_icon_name(manifest_file_content: &String) -> String {
    let mut reader = Reader::from_str(manifest_file_content.as_str());
    reader.trim_text(true);
    let mut icon_name: String = "".to_string();
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                // exchange package name
                if e.name().as_ref() == b"application" {
                    icon_name = String::from_utf8(
                        e.attributes()
                            .map(|attr| attr.unwrap())
                            .find(|attr| attr.key.as_ref() == b"android:icon")
                            .unwrap()
                            .value
                            .to_vec(),
                    )
                    .unwrap();

                    log::info!("icon_name: {}", icon_name);

                    break;
                }
            }
            Ok(Event::End(_e)) => {}
            Ok(Event::Eof) => break,
            // we can either move or borrow the event to write, depending on your use-case
            Ok(_e) => {}
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
        }
    }

    if icon_name == "" {
        icon_name = "@drawable/ic_launcher".to_string();
    }

    icon_name
}

fn main() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .targets([LogTarget::LogDir, LogTarget::Stdout, LogTarget::Webview])
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            extract_tools,
            get_env,
            set_env,
            change_content_and_repack_apk
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
