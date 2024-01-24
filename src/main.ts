import { invoke } from "@tauri-apps/api/tauri";
import { open } from '@tauri-apps/api/dialog';
import { confirm } from '@tauri-apps/api/dialog';
import { exists, BaseDirectory, writeTextFile } from '@tauri-apps/api/fs';
import { sep } from '@tauri-apps/api/path';
import { info } from "tauri-plugin-log-api";

let element_java_path: HTMLInputElement | null;
let element_apk_file_path: HTMLInputElement | null;
let element_apk_package_name: HTMLInputElement | null;
let element_apk_display_name: HTMLInputElement | null;
let element_apk_icon_file_path: HTMLInputElement | null;

let element_apk_repack_in_progress: HTMLLabelElement | null;
let element_apk_repack: HTMLButtonElement | null;


async function initialize() {

    element_apk_repack_in_progress!.hidden = false;
    element_apk_repack_in_progress!.textContent = "Initializing...";
    element_apk_repack!.hidden = true;

    info("Call extract_tools ");
    let result = await invoke<string>("extract_tools", {});
    info("extract_tools result: " + result);

    info("Call get_java_path ");
    result = await invoke<string>("get_env", { name: "JAVA_HOME" });

    element_java_path!.value = result;

    element_apk_repack_in_progress!.hidden = true;
    element_apk_repack!.hidden = false;
}

async function java_path_select() {

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
        if (!element_java_path!.value) {
            // user cancelled the selection
            await confirm('Please select java path', 'Ok');
            return;
        }
        else {
            selected = element_java_path!.value;
        }
    }

    if (Array.isArray(selected)) {
        selected = selected[0];
    }

    let fileExist = await exists(selected, { dir: BaseDirectory.AppData });

    if (!fileExist) {
        await confirm('Java path not exist', 'Ok');
        return;
    }

    // remove bing/java
    if (selected.endsWith("java.exe"))
        selected = selected.substring(0, selected.length - 9);
    if (selected.endsWith("java"))
        selected = selected.substring(0, selected.length - 5);
    if (selected.endsWith(sep))
        selected = selected.substring(0, selected.length - 1);
    if (selected.endsWith("bin"))
        selected = selected.substring(0, selected.length - 4);

    info(selected);

    if (element_java_path) {
        element_java_path!.value = selected;
        info("Call set_java_path ");
        let result = await invoke("set_env", { name: "JAVA_HOME", value: selected });
        info("set_java_path result: " + result);
    }

}

async function apk_file_select() {

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
        if (!element_apk_file_path!.value)
            // user cancelled the selection
            await confirm('Please select apk file', 'Ok');
        return;
    }

    if (Array.isArray(selected)) {
        selected = selected[0];
    }

    let fileExist = await exists(selected, { dir: BaseDirectory.AppData });

    if (!fileExist) {
        await confirm('Apk file not exist', 'Ok');
        return;
    }

    info(selected);
    if (element_apk_file_path)
        element_apk_file_path!.value = selected;

}

async function apk_icon_file_select() {

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
        await confirm('Please select icon file', 'Ok');
        return;
    }

    if (Array.isArray(selected)) {
        selected = selected[0];
    }

    let fileExist = await exists(selected, { dir: BaseDirectory.AppData });

    if (!fileExist) {
        await confirm('Icon file not exist', 'Ok');
        return;
    }

    info(selected);
    if (element_apk_icon_file_path)
        element_apk_icon_file_path!.value = selected;
}

async function do_apk_repack(
    java_path: string | null | undefined,
    apk_file_path: string | null | undefined,
    apk_package_name: string | null | undefined,
    apk_display_name: string | null | undefined,
    apk_icon_file_path: string | null | undefined) {

    element_apk_repack_in_progress!.hidden = false;
    element_apk_repack_in_progress!.textContent = "Repacking...";
    element_apk_repack!.hidden = true;

    if (!apk_file_path) {
        await confirm('Please select apk file', 'Ok');
        element_apk_repack_in_progress!.hidden = true;
        element_apk_repack!.hidden = false;
        return;
    }

    if (!apk_package_name) {
        apk_package_name = '';
    }

    if (!apk_display_name) {
        apk_display_name = '';
    }

    if (!apk_icon_file_path) {
        apk_icon_file_path = '';
    }

    info("Call change_content_and_repack_apk ");
    let result = await invoke<string>("change_content_and_repack_apk", {
        javapath: java_path,
        apkfilepath: apk_file_path,
        apkpackagename: apk_package_name,
        apkdisplayname: apk_display_name,
        apkiconfilepath: apk_icon_file_path
    });

    info("" + result);

    if (result === "success") {
        await confirm('Success', 'Ok');
    }
    else {
        await confirm('Failed, View log for details', 'Ok');
        let fileName = "failed_" + Date.now() + ".txt";
        await writeTextFile(fileName, result, { dir: BaseDirectory.App });
    }

    element_apk_repack_in_progress!.hidden = true;
    element_apk_repack!.hidden = false;
}

window.addEventListener("DOMContentLoaded", () => {
    element_java_path = document.querySelector("#java-path");
    element_apk_file_path = document.querySelector("#apk-file-path");
    element_apk_package_name = document.querySelector("#apk-package-name");
    element_apk_display_name = document.querySelector("#apk-display-name");
    element_apk_icon_file_path = document.querySelector("#apk-icon-file-path");
    element_apk_repack_in_progress = document.querySelector("#apk-repack-in-progress");
    element_apk_repack = document.querySelector("#apk-repack");

    document.querySelector("#java-path-select")?.addEventListener("click", (e) => {
        e.preventDefault();
        info("java-path-select");
        java_path_select();
    });

    document.querySelector("#apk-file-select")?.addEventListener("click", (e) => {
        e.preventDefault();
        info("apk-file-select");
        apk_file_select();
    });

    document.querySelector("#apk-icon-file-select")?.addEventListener("click", (e) => {
        e.preventDefault();
        info("apk-icon-file-select");
        apk_icon_file_select();
    });

    element_apk_repack?.addEventListener("click", (e) => {
        e.preventDefault();
        let java_path = element_java_path?.value;
        let apk_file_path = element_apk_file_path?.value;
        let apk_package_name = element_apk_package_name?.value
        let apk_display_name = element_apk_display_name?.value;
        let apk_icon_file_path = element_apk_icon_file_path?.value;
        do_apk_repack(java_path, apk_file_path, apk_package_name, apk_display_name, apk_icon_file_path);
    });

    initialize();
});


