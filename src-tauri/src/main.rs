// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use apk_util::get_apk_info;
use constants::get_cache_folder;
use prepare::extract_tools;
use std::path::Path;
use std::vec;

use tauri_plugin_log::LogTarget;

mod apk_util;
mod constants;
mod prepare;
mod run_command;
mod xml_util;

use crate::apk_util::exchange_apk_display_name;
use crate::apk_util::exchange_apk_icon;
use crate::apk_util::exchange_apk_package_name;
use crate::apk_util::get_apk_vasdolly_channel;
use crate::apk_util::pack_apk;
use crate::apk_util::set_apk_vasdolly_channel;
use crate::apk_util::sign_apk;
use crate::apk_util::unpack_apk;

#[tauri::command]
fn get_env(name: &str) -> String {
    std::env::var(String::from(name)).unwrap_or(String::from(""))
}

#[tauri::command]
fn set_env(name: &str, value: &str) {
    log::info!("set_env: {} => {}", name, value);
    std::env::set_var(String::from(name), String::from(value));
}

#[tauri::command]
fn get_cache_dir() -> String {
    get_cache_folder().to_str().unwrap().to_string()
}

#[tauri::command]
async fn unpack_and_get_apk_info(apk_path: String) -> String {
    let apk_folder = unpack_apk(&apk_path).await;
    if apk_folder.is_err() {
        return format!("error unpack apk: {}", apk_folder.err().unwrap());
    }

    let apk_folder = apk_folder.unwrap();
    let apk_info = get_apk_info(&apk_folder).await;
    if apk_info.is_err() {
        return format!("error get apk info: {}", apk_info.err().unwrap());
    }

    let apk_info = apk_info.unwrap();

    serde_json::to_string(&apk_info).unwrap()
}

async fn do_change_content_and_repack(
    apk_file_path: String,
    apk_package_name: String,
    apk_display_name: String,
    apk_icon_file_path: String,
) -> Result<()> {
    let apk_folder = unpack_apk(&apk_file_path).await?;
    exchange_apk_package_name(&apk_folder, &apk_package_name)?;
    exchange_apk_display_name(&apk_folder, &apk_display_name)?;
    exchange_apk_icon(&apk_folder, &apk_icon_file_path)?;

    let repacked_apk = pack_apk(&apk_folder).await?;
    sign_apk(&repacked_apk).await?;

    let channel = get_apk_vasdolly_channel(&apk_file_path).await?;
    log::info!("channel: {}", channel);

    if channel.is_empty() || channel == "null" {
        log::info!("channel not found");
    } else {
        set_apk_vasdolly_channel(&repacked_apk, &channel).await?;
    }

    let des = apk_file_path + ".repacked.apk";
    let des = Path::new(&des);
    if des.exists() {
        std::fs::remove_file(des)?;
    }

    std::fs::rename(&repacked_apk, des)?;

    Ok(())
}

#[tauri::command]
async fn change_content_and_repack_apk(
    apk_file_path: String,
    apk_package_name: String,
    apk_display_name: String,
    apk_icon_file_path: String,
) -> String {
    let result = do_change_content_and_repack(
        apk_file_path,
        apk_package_name,
        apk_display_name,
        apk_icon_file_path,
    )
    .await;

    match result {
        Err(e) => {
            log::error!("{:?}", e);
            "error".to_string()
        }
        Ok(_) => "success".to_string(),
    }
}

fn main() {
    let result = extract_tools();
    if result.is_err() {
        log::error!("{:?}", result.err().unwrap());
        return;
    }

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .targets([
                    LogTarget::Folder(get_cache_folder().to_owned()),
                    LogTarget::Stdout,
                    LogTarget::Webview,
                ])
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            get_env,
            set_env,
            get_cache_dir,
            unpack_and_get_apk_info,
            change_content_and_repack_apk,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
