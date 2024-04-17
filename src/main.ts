import { invoke } from "@tauri-apps/api/tauri";
import { message, open } from '@tauri-apps/api/dialog';
import { exists, BaseDirectory, copyFile } from '@tauri-apps/api/fs';
import { sep, join } from '@tauri-apps/api/path'
import { info } from "tauri-plugin-log-api";
import { convertFileSrc } from '@tauri-apps/api/tauri';


let lb_apk_local_path: HTMLLabelElement | null;
let btn_apk_load_info: HTMLButtonElement | null;
let btn_apk_load_by_path: HTMLButtonElement | null;

let lb_package_old: HTMLLabelElement | null;
let it_package_new: HTMLInputElement | null;

let lb_display_name_old: HTMLLabelElement | null;
let it_display_name_new: HTMLInputElement | null;

let icon_old: HTMLImageElement | null;
let icon_new: HTMLImageElement | null;
let btn_load_icon: HTMLButtonElement | null;
let des_icon_path: string | null = null;

let lb_jdk_path: HTMLLabelElement | null;
let btn_load_jdk_path: HTMLButtonElement | null;

let btn_start_work: HTMLButtonElement | null;

async function cacheDir() {

    let cache_dir = await invoke<string>("get_cache_dir");
    info("cache_dir: " + cache_dir);

    return cache_dir;
}

async function alert(content: string) {
    await message(content, {
        title: 'Tip',
        okLabel: 'Ok'
    });
}

async function bind_apk_elements() {
    lb_apk_local_path = document.querySelector("#lb_apk_local_path");
    btn_apk_load_info = document.querySelector("#btn_apk_load_info");
    btn_apk_load_by_path = document.querySelector("#btn_apk_load_by_path");

    btn_apk_load_info!.hidden = true;

    btn_apk_load_by_path?.addEventListener("click", (e) => {
        e.preventDefault();
        load_apk();
    });

    btn_apk_load_info?.addEventListener("click", (e) => {
        e.preventDefault();
        load_apk_info();
    });
}

async function load_apk() {

    // Open a selection dialog for directories
    let selected = await open({
        directory: false,
        multiple: false,
        filters: [{
            name: '*.apk',
            extensions: ['apk']
        }],
    });

    if (selected === null) {
        if (!lb_apk_local_path!.textContent)
            // user cancelled the selection
            await alert('Please select an apk file');
        return;
    }

    if (Array.isArray(selected)) {
        selected = selected[0];
    }

    let fileExist = await exists(selected, { dir: BaseDirectory.AppData });

    if (!fileExist) {
        await alert('Apk file not exist');
        return;
    }

    info(selected);

    lb_apk_local_path!.textContent = selected;
    btn_apk_load_info!.hidden = false;
}

async function load_apk_info() {
    info("load_apk_info");

    let apk_path = lb_apk_local_path!.textContent;

    if (!apk_path || !await exists(apk_path!, { dir: BaseDirectory.AppData })) {
        await alert('Please select an apk file');
        return;
    }

    btn_apk_load_info!.hidden = true;

    let result = await invoke<string>("unpack_and_get_apk_info", {
        apkPath: apk_path,
    });

    info("" + result);

    if (result.startsWith("error")) {
        await alert('Load apk info failed');
    }
    else {
        let apk_info = JSON.parse(result);
        info("apk_info: " + apk_info);

        lb_package_old!.textContent = apk_info['package_name'];
        if (it_package_new!.value == "")
            it_package_new!.value = apk_info['package_name'];
        lb_display_name_old!.textContent = apk_info['display_name'];
        if (it_display_name_new!.value == "")
            it_display_name_new!.value = apk_info['display_name'];

        let src_icon_path = apk_info['icon_path'];
        let icon_path = convertFileSrc(src_icon_path);
        info("icon_path: " + icon_path);
        icon_old!.src = icon_path;
        if (!des_icon_path) {
            icon_new!.src = icon_path;
            des_icon_path = src_icon_path;
        }

        await alert('Load apk info success');
    }

    btn_apk_load_info!.hidden = false;
}

function bind_package_elements() {
    lb_package_old = document.querySelector("#lb_package_old");
    it_package_new = document.querySelector("#it_package_new");
    lb_display_name_old = document.querySelector("#lb_display_name_old");
    it_display_name_new = document.querySelector("#it_display_name_new");
}

async function bind_app_icon_elements() {

    icon_old = document.querySelector("#icon_old");
    icon_new = document.querySelector("#icon_new");
    btn_load_icon = document.querySelector("#btn_load_icon");

    btn_load_icon?.addEventListener("click", (e) => {
        e.preventDefault();
        load_icon();
    });
}

async function load_icon() {

    info("load_icon");
    // Open a selection dialog for directories
    let selected = await open({
        directory: false,
        multiple: false,
        filters: [{
            name: '*.png',
            extensions: ['png']
        }],
    });

    if (selected === null) {
        // user cancelled the selection
        await alert('Please select an icon file');
        return;
    }

    if (Array.isArray(selected)) {
        selected = selected[0];
    }

    let fileExist = await exists(selected, { dir: BaseDirectory.AppData });

    if (!fileExist) {
        await alert('Icon file not exist');
        return;
    }

    info(selected);

    let cached = await cacheDir();
    let icon_cached = await join(cached, 'des-icon_' + Date.now() + '.png');
    await copyFile(selected, icon_cached);

    info("icon_cached: " + icon_cached);
    let icon_path = convertFileSrc(icon_cached);
    info("icon_path: " + icon_path);
    icon_new!.src = icon_path;

    des_icon_path = icon_cached;
}

async function bind_environments() {
    lb_jdk_path = document.querySelector("#lb_jdk_path");
    btn_load_jdk_path = document.querySelector("#btn_load_jdk_path");

    btn_load_jdk_path?.addEventListener("click", (e) => {
        e.preventDefault();
        load_jdk_path();
    });

    let result = await invoke<string>("get_env", { name: "JAVA_HOME" });

    lb_jdk_path!.textContent = result;
}

async function load_jdk_path() {

    // Open a selection dialog for directories
    let selected = await open({
        directory: true,
        multiple: false,
        filters: [{
            name: 'bin',
            extensions: ['']
        }],
    });

    info("selected: " + selected);

    if (!selected) {
        if (!lb_jdk_path!.textContent) {
            // user cancelled the selection
            await alert('Please select java path');
            return;
        }
        else {
            selected = lb_jdk_path!.textContent;
        }
    }

    if (Array.isArray(selected)) {
        selected = selected[0];
    }

    let fileExist = await exists(selected, { dir: BaseDirectory.AppData });

    if (!fileExist) {
        await alert('Java path not exist');
        return;
    }

    // remove bin/java
    if (selected.endsWith(".exe"))
        selected = selected.substring(0, selected.length - 9);
    if (selected.endsWith("java"))
        selected = selected.substring(0, selected.length - 5);
    if (selected.endsWith(sep))
        selected = selected.substring(0, selected.length - 1);
    if (selected.endsWith("bin"))
        selected = selected.substring(0, selected.length - 4);

    info(selected);


    lb_jdk_path!.textContent = selected;
    await invoke("set_env", { name: "JAVA_HOME", value: selected });
}

async function repack(): Promise<boolean> {
    let apk_file_path = lb_apk_local_path!.textContent;
    let apk_package_name = it_package_new!.value;
    let apk_display_name = it_display_name_new!.value;
    let apk_icon_file_path = des_icon_path;

    if (!apk_file_path || !await exists(apk_file_path, { dir: BaseDirectory.AppData })) {
        await alert('Please select an apk file');
        return false;
    }

    if (!apk_package_name) {
        await alert('Please fill in the package name');
        return false;
    }

    if (!apk_display_name) {
        await alert('Please fill in the display name');
        return false;
    }

    if (!apk_icon_file_path) {
        await alert('Please select an icon file');
        return false;
    }

    let result = await invoke<string>("change_content_and_repack_apk", {
        apkFilePath: apk_file_path,
        apkPackageName: apk_package_name,
        apkDisplayName: apk_display_name,
        apkIconFilePath: apk_icon_file_path
    });

    if (result != "success") {
        await alert('Repack failed');
        return false;
    }

    return true;
}

async function start_work() {

    btn_start_work!.hidden = true;

    let success: boolean = false;

    do {
        success = await repack();
        if (!success) break;

    } while (false);

    btn_start_work!.hidden = false;

    if (success) {
        await alert('Repack success');
    }
}

function bind_works() {
    btn_start_work = document.querySelector("#btn_start_work");
    btn_start_work?.addEventListener("click", (e) => {
        e.preventDefault();
        start_work();
    });

    btn_start_work!.hidden = false;
}

window.addEventListener("DOMContentLoaded", () => {

    info("DOMContentLoaded");
    bind_apk_elements();
    bind_package_elements();
    bind_app_icon_elements();
    bind_works();
    bind_environments();
});




