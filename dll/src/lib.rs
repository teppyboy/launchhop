use process_path::get_dylib_path;
use serde::Deserialize;
use std::fs;
use std::process::Command;
use std::{thread, time};
use windows::{
    core::*, Win32::UI::WindowsAndMessaging::*,
    Win32::Foundation::*, Win32::System::LibraryLoader::FreeLibraryAndExitThread,
    Win32::System::{
        SystemServices::*,
        Console::*, Threading::*,
    }
};

#[derive(Deserialize)]
struct Program {
    path: String,
    args: Vec<String>,
    delay: u64,
}

#[derive(Deserialize)]
struct Launcher {
    kill_after_launch: bool,
    kill_after_target_exit: bool,
}

#[derive(Deserialize)]
struct Config {
    target: Program,
    launcher: Launcher,
}

#[no_mangle]
pub extern "C" fn add(left: usize, right: usize) -> usize {
    left + right
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
// HINSTANCE is not available anymore, and according to the internet,
// it's the same as HMODULE
extern "system" fn DllMain(dll_module: HMODULE, call_reason: u32, _: *mut ()) -> u32 {
    unsafe {
        match call_reason {
            DLL_PROCESS_ATTACH => attach(),
            DLL_PROCESS_DETACH => (),
            _ => (),
        }
        // Unload the DLL immediately after initialization.
        FreeConsole();
        FreeLibraryAndExitThread(dll_module, 0);
    }
}

unsafe fn attach() {
    let cur_proc_id = GetCurrentProcessId();
    AllocConsole();
    AttachConsole(cur_proc_id);
    SetConsoleTitleA(s!("LaunchHop"));
    println!("===/LaunchHop/===");
    println!("Attached to process [{}]", cur_proc_id);
    let pathbuf = match get_dylib_path() {
        None => {
            MessageBoxA(
                HWND(0),
                s!("Failed to get current DLL location."),
                s!("LaunchHop"),
                MB_ICONERROR | MB_OK,
            );
            return;
        }
        Some(pathbuf) => pathbuf,
    };
    let path: &std::path::Path = pathbuf.parent().unwrap();
    println!("DLL Path: {:?}", path);
    let config_path = path.clone().join("config.toml");
    println!("Config path: {:?}", config_path);
    println!("Reading config.toml...");
    let config = match fs::read_to_string(config_path) {
        Ok(config) => match toml::from_str::<Config>(&config) {
            Ok(config) => config,
            Err(_) => {
                println!("Failed to parse config.toml");
                MessageBoxA(
                    HWND(0),
                    s!("Failed to parse config.toml"),
                    s!("LaunchHop"),
                    MB_ICONERROR | MB_OK,
                );
                return;
            }
        },
        Err(_) => {
            println!("Failed to read config.toml");
            MessageBoxA(
                HWND(0),
                s!("Failed to read config.toml"),
                s!("LaunchHop"),
                MB_ICONERROR | MB_OK,
            );
            return;
        }
    };
    println!("Target path: {:?}", config.target.path);
    println!("Target args: {:?}", config.target.args);
    let mut proc = Command::new(config.target.path);
    for arg in config.target.args {
        proc.arg(arg);
    }
    println!("Waiting {}ms to spawn process...", config.target.delay);
    thread::sleep(time::Duration::from_millis(config.target.delay));
    println!("Launching process...");
    let mut child_proc = match proc.spawn() {
        Ok(proc) => {
            println!("Process launched.");
            proc
        }
        Err(_) => {
            println!("Failed to launch process.");
            MessageBoxA(
                HWND(0),
                s!("Failed to launch target."),
                s!("LaunchHop"),
                MB_ICONERROR | MB_OK,
            );
            return;
        }
    };
    if config.launcher.kill_after_launch {
        println!("Killing launcher process...");
        let kill_result = TerminateProcess(GetCurrentProcess(), 0);
        if kill_result == false {
            println!("Failed to kill launcher process.");
            MessageBoxA(
                HWND(0),
                s!("Failed to kill launcher process."),
                s!("LaunchHop"),
                MB_ICONERROR | MB_OK,
            );
            return;
        }
        println!("Launcher process killed.");
        return;
    }
    if config.launcher.kill_after_target_exit {
        println!("Waiting for target process to exit...");
        child_proc.wait().unwrap();
        println!("Killing launcher process...");
        let kill_result = TerminateProcess(GetCurrentProcess(), 0);
        if kill_result == false {
            println!("Failed to kill launcher process.");
            MessageBoxA(
                HWND(0),
                s!("Failed to kill launcher process."),
                s!("LaunchHop"),
                MB_ICONERROR | MB_OK,
            );
            return;
        }
        println!("Launcher process killed.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}