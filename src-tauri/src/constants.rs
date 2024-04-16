use std::sync::OnceLock;

pub const APKTOOL_JAR_BYTES: &[u8] = include_bytes!("../apktool.jar");
pub const APKTOOL_NAME: &str = "apktool.jar";

pub const VASDOLLY_JAR_BYTES: &[u8] = include_bytes!("../vasdolly.jar");
pub const VASDOLLY_NAME: &str = "vasdolly.jar";

pub const APKSIGNER_JAR_BYTES: &[u8] = include_bytes!("../uber-apk-signer.jar");
pub const APKSIGNER_NAME: &str = "uber-apk-signer.jar";

pub const KEYSTORE_BYTES: &[u8] = include_bytes!("../key.keystore");
pub const KEYSTORE_NAME: &str = "key.keystore";
pub const KEYSTORE_PWD: &str = "123456";
pub const KEYSTORE_ALIAS_NAME: &str = "key";

pub fn exe_folder() -> &'static std::path::PathBuf {
    static EXE_FOLDER: OnceLock<std::path::PathBuf> = OnceLock::new();
    EXE_FOLDER.get_or_init(|| {
        let mut path = std::env::current_exe().unwrap();
        path.pop();
        if path.ends_with("deps") {
            path.pop();
        }
        path
    })
}

pub fn get_apktool_jar_path() -> std::path::PathBuf {
    exe_folder().join(APKTOOL_NAME)
}

pub fn get_vasdolly_jar_path() -> std::path::PathBuf {
    exe_folder().join(VASDOLLY_NAME)
}

pub fn get_apksigner_jar_path() -> std::path::PathBuf {
    exe_folder().join(APKSIGNER_NAME)
}

pub fn get_key_store_path() -> std::path::PathBuf {
    exe_folder().join(KEYSTORE_NAME)
}

pub fn get_cache_folder() -> &'static std::path::PathBuf {
    static CACHE_FOLDER: OnceLock<std::path::PathBuf> = OnceLock::new();
    CACHE_FOLDER.get_or_init(|| {
        let mut path = exe_folder().clone();
        let time_str = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
        path.push(time_str);
        if !path.exists() {
            std::fs::create_dir_all(&path).unwrap();
        }
        path
    })
}

pub fn get_java_exe() -> anyhow::Result<String> {
    let java_home = std::env::var("JAVA_HOME").unwrap_or("".to_string());
    if java_home.is_empty() {
        return Err(anyhow::anyhow!("JAVA_HOME not found"));
    }

    let java_path = if cfg!(target_os = "windows") {
        std::path::Path::new(&java_home)
            .join("bin")
            .join("javaw.exe")
    } else {
        std::path::Path::new(&java_home).join("bin").join("java")
    };

    if !java_path.exists() {
        return Err(anyhow::anyhow!("java not found"));
    }

    Ok(java_path.to_str().unwrap().to_string())
}
