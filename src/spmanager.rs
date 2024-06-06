use bevy::prelude::*;
use bevy_console::{ConsoleCommandEntered, ConsolePlugin, ConsoleSet, PrintConsoleLine, Color32};
use colored::Colorize;
use paprika::*;
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, RwLock},
};
/// Spice Manager to facillitate interaction with NgSpice
#[derive(Debug, Default)]
struct SpManager {
    sharedres: Arc<RwLock<VecDeque<(String, Color32)>>>,
    vecvals: Mutex<Vec<PkVecvaluesall>>,
    vecinfo: Option<PkVecinfoall>,
}

impl SpManager {
    fn new() -> Self {
        SpManager::default()
    }
}

#[allow(unused_variables)]
impl paprika::PkSpiceManager for SpManager {
    fn cb_send_char(&mut self, msg: String, id: i32) {
        let opt = msg.split_once(' ');
        let (token, msgs) = match opt {
            Some(tup) => (tup.0, tup.1),
            None => (msg.as_str(), ""),
        };
        let color = match token {
            "stdout" => Color32::GREEN,
            "stderr" => Color32::RED,
            _ => Color32::LIGHT_RED, // some unknown channel
        };
        let mut arvs = self.sharedres.write().unwrap();
        (*arvs).push_back((msgs.to_owned(), color));
    }
    fn cb_send_stat(&mut self, msg: String, id: i32) {
        let mut arvs = self.sharedres.write().unwrap();
        (*arvs).push_back((msg, Color32::BLUE));
    }
    fn cb_ctrldexit(&mut self, status: i32, is_immediate: bool, is_quit: bool, id: i32) {}
    fn cb_send_init(&mut self, pkvecinfoall: PkVecinfoall, id: i32) {
        self.vecinfo = Some(pkvecinfoall);
    }
    fn cb_send_data(&mut self, pkvecvaluesall: PkVecvaluesall, count: i32, id: i32) {
        // this is called every simulation step when running tran
        self.vecvals.try_lock().unwrap().push(pkvecvaluesall);
    }
    fn cb_bgt_state(&mut self, is_fin: bool, id: i32) {}
}

#[derive(Resource)]
struct SPRes {
    /// spice manager
    spm: Arc<SpManager>,
    /// ngspice library
    lib: PkSpice<SpManager>,
}

impl Default for SPRes {
    fn default() -> Self {
        let spm = Arc::new(SpManager::new());
        let mut lib;
        #[cfg(target_family = "windows")]
        {
            lib = PkSpice::<SpManager>::new(std::ffi::OsStr::new("ngspice.dll")).unwrap();
        }
        #[cfg(target_os = "macos")]
        {
            // retrieve libngspice.dylib from the following possible directories
            let ret = std::process::Command::new("find")
                .args(&["/usr/lib", "/usr/local/lib"])
                .arg("-name")
                .arg("*libngspice.dylib")
                .stdout(std::process::Stdio::piped())
                .output()
                .unwrap_or_else(|_| {
                    eprintln!("Error: Could not find libngspice.dylib. Make sure it is installed.");
                    std::process::exit(1);
                });
            let path = String::from_utf8(ret.stdout).unwrap();
            lib = PkSpice::<SpManager>::new(&std::ffi::OsString::from(path.trim())).unwrap();
        }
        #[cfg(target_os = "linux")]
        {
            // dynamically retrieves libngspice from system
            let ret = std::process::Command::new("sh")
                .arg("-c")
                .arg("ldconfig -p | grep ngspice | awk '/.*libngspice.so$/{print $4}'")
                .stdout(std::process::Stdio::piped())
                .output()
                .unwrap_or_else(|_| {
                    eprintln!("Error: Could not find libngspice. Make sure it is installed.");
                    std::process::exit(1);
                });

            let path = String::from_utf8(ret.stdout).unwrap();
            lib = PkSpice::<SpManager>::new(&std::ffi::OsString::from(path.trim())).unwrap();
        }
        lib.init(Some(spm.clone()));
        SPRes { spm, lib }
    }
}

pub struct SPManagerPlugin;

impl Plugin for SPManagerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ConsolePlugin);
        app.init_resource::<SPRes>();
        app.add_systems(Update, write_to_console.after(ConsoleSet::ConsoleUI));
        app.add_systems(Update, pass_to_ngspice.after(ConsoleSet::Commands));
    }
}

fn write_to_console(mut console_line: EventWriter<PrintConsoleLine>, sres: Res<SPRes>) {
    for s in sres.spm.sharedres.write().unwrap().drain(..) {
        console_line.send(PrintConsoleLine::new(s.0, s.1));
    }
}

fn pass_to_ngspice(mut console_line: EventReader<ConsoleCommandEntered>, sres: Res<SPRes>) {
    for ConsoleCommandEntered { command } in console_line.read() {
        sres.lib.command(command);
    }
}
