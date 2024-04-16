use anyhow::Result;
use std::io::Write;

use crate::constants::{
    get_apksigner_jar_path, get_apktool_jar_path, get_cache_folder, get_key_store_path,
    get_vasdolly_jar_path, APKSIGNER_JAR_BYTES, APKTOOL_JAR_BYTES, KEYSTORE_BYTES,
    VASDOLLY_JAR_BYTES,
};

pub fn extract_tools() -> Result<()> {
    log::info!("extract_tools");

    // check if apktool.jar exists
    let apktool_jar_path = get_apktool_jar_path();
    if !apktool_jar_path.exists() {
        // extract apktool.jar
        let mut apktool_jar_file = std::fs::File::create(&apktool_jar_path)?;
        apktool_jar_file.write_all(APKTOOL_JAR_BYTES)?;
        apktool_jar_file.flush()?;
        log::info!("apktool.jar extracted");
    }

    // check if vasdolly.jar exists
    let vasdolly_jar_path = get_vasdolly_jar_path();
    if !vasdolly_jar_path.exists() {
        // extract vasdolly.jar
        let mut vasdolly_jar_file = std::fs::File::create(&vasdolly_jar_path)?;
        vasdolly_jar_file.write_all(VASDOLLY_JAR_BYTES)?;
        vasdolly_jar_file.flush()?;
        log::info!("vasdolly.jar extracted");
    }

    // check if uber-apk-signer.jar exists
    let apksigner_jar_path = get_apksigner_jar_path();
    if !apksigner_jar_path.exists() {
        // extract uber-apk-signer.jar
        let mut apksigner_jar_file = std::fs::File::create(apksigner_jar_path)?;
        apksigner_jar_file.write_all(APKSIGNER_JAR_BYTES)?;
        apksigner_jar_file.flush()?;
        log::info!("uber-apk-signer.jar extracted");
    }

    // check if key.keystore exists
    let key_store_path = get_key_store_path();
    if !key_store_path.exists() {
        // extract key.keystore
        let mut key_store_file = std::fs::File::create(key_store_path)?;
        key_store_file.write_all(KEYSTORE_BYTES)?;
        key_store_file.flush()?;
        log::info!("key.keystore extracted");
    }

    // create log folder
    let cache_folder = get_cache_folder();

    if !cache_folder.exists() {
        std::fs::create_dir_all(cache_folder)?;
        log::info!("cache_folder {} created", cache_folder.to_str().unwrap());
    }

    Ok(())
}
