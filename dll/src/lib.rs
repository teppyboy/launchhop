use process_path::get_dylib_path;
use serde::Deserialize;
use std::ffi::c_void;
use std::fs;
use std::process::Command;
use std::{thread, time};
use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::System::LibraryLoader::{DisableThreadLibraryCalls, FreeLibraryAndExitThread},
    Win32::System::{Console::*, SystemServices::*, Threading::*},
    Win32::UI::WindowsAndMessaging::*,
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
    debug: bool,
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
extern "system" fn DllMain(mut dll_module: HMODULE, call_reason: u32, _: *mut ()) -> i32 {
    unsafe {
        match call_reason {
            // https://www.unknowncheats.me/forum/rust-language-/330583-pure-rust-injectable-dll.html
            // but ported over windows-rs
            DLL_PROCESS_ATTACH => {
                DisableThreadLibraryCalls(dll_module);
                let dll_module_ptr: *mut c_void = &mut dll_module as *mut _ as *mut c_void;
                match CreateThread(
                    None,
                    0,
                    Some(attach),
                    Some(dll_module_ptr),
                    THREAD_CREATION_FLAGS(0),
                    None,
                ) {
                    Ok(_) => (),
                    Err(_) => {
                        MessageBoxA(
                            HWND(0),
                            s!("Failed to create thread."),
                            s!("LaunchHop"),
                            MB_ICONERROR | MB_OK,
                        );
                    }
                }
            }
            DLL_PROCESS_DETACH => (),
            _ => (),
        }
        true as i32
    }
}

unsafe extern "system" fn attach(base: *mut c_void) -> u32 {
    use std::panic; // local import

    // make sure that when attached, it doesn't fuck us over somehow (thru panicking)
    match panic::catch_unwind(|| main()) {
        Err(e) => {
            eprintln!("[DLL]: `main` has panicked: {:#?}", e);
        }
        Ok(r) => match r {
            _ => (),
        },
    }

    // free the lib and exit the thread.
    // the thread should just stop working now
    let a = base as *mut HMODULE;
    let b = *a;
    FreeLibraryAndExitThread(b, 1);
}

unsafe fn cleanup(config: Option<Config>) -> u32 {
    if config.is_none() || !config.unwrap().debug {
        FreeConsole();
    }
    0
}

// unsafe extern "system" fn main(_: *mut c_void) -> u32 {
unsafe fn main() -> u32 {
    let cur_proc_id = GetCurrentProcessId();
    AllocConsole();
    AttachConsole(cur_proc_id);
    SetConsoleTitleA(s!("LaunchHop"));
    println!(
        "
    ██╗      █████╗ ██╗   ██╗███╗   ██╗ ██████╗██╗  ██╗██╗  ██╗ ██████╗ ██████╗ 
    ██║     ██╔══██╗██║   ██║████╗  ██║██╔════╝██║  ██║██║  ██║██╔═══██╗██╔══██╗
    ██║     ███████║██║   ██║██╔██╗ ██║██║     ███████║███████║██║   ██║██████╔╝
    ██║     ██╔══██║██║   ██║██║╚██╗██║██║     ██╔══██║██╔══██║██║   ██║██╔═══╝ 
    ███████╗██║  ██║╚██████╔╝██║ ╚████║╚██████╗██║  ██║██║  ██║╚██████╔╝██║     
    ╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═══╝ ╚═════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚═╝     
                                                                                "
    );
    println!("[DLL]: Attached to process [{}]", cur_proc_id);
    let pathbuf = match get_dylib_path() {
        None => {
            MessageBoxA(
                HWND(0),
                s!("Failed to get current DLL location."),
                s!("LaunchHop"),
                MB_ICONERROR | MB_OK,
            );
            return cleanup(None);
        }
        Some(pathbuf) => pathbuf,
    };
    let path: &std::path::Path = pathbuf.parent().unwrap();
    println!("[DLL]: DLL Path: {:?}", path);
    let config_path = path.clone().join("config.toml");
    println!("[DLL]: Config path: {:?}", config_path);
    println!("[DLL]: Reading config.toml...");
    let config = match fs::read_to_string(config_path) {
        Ok(config) => match toml::from_str::<Config>(&config) {
            Ok(config) => config,
            Err(_) => {
                println!("[DLL]: Failed to parse config.toml");
                MessageBoxA(
                    HWND(0),
                    s!("Failed to parse config.toml"),
                    s!("LaunchHop"),
                    MB_ICONERROR | MB_OK,
                );
                return cleanup(None);
            }
        },
        Err(_) => {
            println!("[DLL]: Failed to read config.toml");
            MessageBoxA(
                HWND(0),
                s!("Failed to read config.toml"),
                s!("LaunchHop"),
                MB_ICONERROR | MB_OK,
            );
            return cleanup(None);
        }
    };
    println!("[DLL]: Target path: {:?}", config.target.path);
    println!("[DLL]: Target args: {:?}", config.target.args);
    let mut proc = Command::new(config.target.path.clone());
    for arg in config.target.args.clone() {
        proc.arg(arg);
    }
    println!("[DLL]: Waiting {}ms to spawn process...", config.target.delay);
    thread::sleep(time::Duration::from_millis(config.target.delay));
    println!("[DLL]: Launching process...");
    let mut child_proc = match proc.spawn() {
        Ok(proc) => {
            println!("[DLL]: Process launched.");
            proc
        }
        Err(_) => {
            println!("[DLL]: Failed to launch process.");
            MessageBoxA(
                HWND(0),
                s!("Failed to launch target."),
                s!("LaunchHop"),
                MB_ICONERROR | MB_OK,
            );
            return cleanup(Some(config));
        }
    };
    if !config.debug {
        FreeConsole();
    }
    if config.launcher.kill_after_launch {
        println!("[DLL]: Killing launcher process...");
        let kill_result = TerminateProcess(GetCurrentProcess(), 0);
        if kill_result == false {
            println!("[DLL]: Failed to kill launcher process.");
            MessageBoxA(
                HWND(0),
                s!("Failed to kill launcher process."),
                s!("LaunchHop"),
                MB_ICONERROR | MB_OK,
            );
            return cleanup(Some(config));
        }
        println!("[DLL]: Launcher process killed.");
        return 0;
    }
    if config.launcher.kill_after_target_exit {
        println!("[DLL]: Waiting for target process to exit...");
        child_proc.wait().unwrap();
        println!("[DLL]: Killing launcher process...");
        let kill_result = TerminateProcess(GetCurrentProcess(), 0);
        if kill_result == false {
            println!("[DLL]: Failed to kill launcher process.");
            MessageBoxA(
                HWND(0),
                s!("Failed to kill launcher process."),
                s!("LaunchHop"),
                MB_ICONERROR | MB_OK,
            );
            return cleanup(Some(config));
        }
        println!("[DLL]: Launcher process killed.");
    }
    println!("[DLL]: The logs below are from the target process...");
    return 0;
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
