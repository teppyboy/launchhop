use dll_syringe::{process::OwnedProcess, Syringe};
use serde::Deserialize;
use std::env;
use std::fs;
use std::process::Command;
use std::{thread, time};

#[derive(Deserialize)]
struct Program {
    path: String,
    args: Vec<String>,
    delay: u64,
    inject_delay: u64,
}

#[derive(Deserialize)]
struct Config {
    dll_name: String,
    launcher: Program,
}

fn main() {
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
    println!("Reading config.toml...");
    let cwd = env::current_dir().unwrap();
    let config_path = cwd.join("config.toml");
    let config = match fs::read_to_string(config_path) {
        Ok(config) => match toml::from_str::<Config>(&config) {
            Ok(config) => config,
            Err(_) => {
                println!("Failed to parse config.toml");
                return;
            }
        },
        Err(_) => {
            println!("Failed to read config.toml");
            return;
        }
    };
    println!("Launcher path: {:?}", config.launcher.path);
    println!("Launcher args: {:?}", config.launcher.args);
    let mut proc = Command::new(config.launcher.path);
    for arg in config.launcher.args {
        proc.arg(arg);
    }
    println!("Waiting {}ms to spawn process...", config.launcher.delay);
    thread::sleep(time::Duration::from_millis(config.launcher.delay));
    println!("Launching process...");
    let child_proc = match proc.spawn() {
        Ok(proc) => {
            println!("Process launched.");
            proc
        }
        Err(_) => {
            println!("Failed to launch process.");
            return;
        }
    };
    println!(
        "Waiting {}ms to before injecting...",
        config.launcher.inject_delay
    );
    thread::sleep(time::Duration::from_millis(config.launcher.inject_delay));
    println!("Injecting DLL...");
    let dll_path = cwd.join(config.dll_name);
    let owned_proc = OwnedProcess::from_pid(child_proc.id()).unwrap();
    let syringe = Syringe::for_process(owned_proc);
    match syringe.inject(dll_path) {
        Ok(_) => {
            println!("DLL injected.");
        }
        Err(_) => {
            println!("Failed to inject DLL.");
            return;
        }
    }
    thread::sleep(time::Duration::from_millis(1000));
}
