#![windows_subsystem = "console"]

mod clicker;
mod config;
mod error;
mod hook;
mod input;
mod timer;
mod vk;

use std::{hint, io::{self, Write}, process, time::Duration};

use clicker::{ButtonState, ClickParams};
use config::Config;
use error::ClixorError;
use hook::MouseHook;
use input::{send_click, Button};
use timer::SleepBackend;
use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;
use windows::Win32::System::Console::{
    GetConsoleMode, GetStdHandle, SetConsoleMode, SetConsoleTitleW,
    CONSOLE_MODE, ENABLE_VIRTUAL_TERMINAL_PROCESSING, STD_OUTPUT_HANDLE,
};
use windows::core::w;

const DIM: &str = "\x1b[38;5;238m";
const ACCENT: &str = "\x1b[38;5;214m";
const LABEL: &str = "\x1b[38;5;245m";
const VALUE: &str = "\x1b[38;5;252m";
const RESET: &str = "\x1b[0m";

struct ClixorApp {
    cfg: Config,
    sleep: SleepBackend,
    hook: MouseHook,
}

impl ClixorApp {
    fn new(cfg: Config) -> Result<Self, ClixorError> {
        let sleep = SleepBackend::init();
        let hook = MouseHook::install()?;
        Ok(Self { cfg, sleep, hook })
    }

    fn run(&mut self, lmb_vk: VIRTUAL_KEY, rmb_vk: VIRTUAL_KEY) {
        let mut lmb = ButtonState::default();
        let mut rmb = ButtonState::default();
        let lmb_params = ClickParams::new(
            self.cfg.lmb_cps,
            self.cfg.ramp_start_ratio,
            self.cfg.ramp_duration_ms,
        );
        let rmb_params = ClickParams::new(
            self.cfg.rmb_cps,
            self.cfg.ramp_start_ratio,
            self.cfg.ramp_duration_ms,
        );
        let spin_threshold = Duration::from_micros(self.cfg.spin_threshold_us);
        let sleep_undershoot = Duration::from_micros(self.cfg.sleep_undershoot_us);

        print_status(lmb.enabled, rmb.enabled);

        loop {
            let lmb_changed = lmb.process_toggle(vk::is_held(lmb_vk));
            let rmb_changed = rmb.process_toggle(vk::is_held(rmb_vk));

            if lmb_changed || rmb_changed {
                print_status(lmb.enabled, rmb.enabled);
            }

            let lmb_held = self.hook.lmb_held();
            let rmb_held = self.hook.rmb_held();

            if lmb.tick(lmb_held, &lmb_params) {
                send_click(Button::Left);
            }
            if rmb.tick(rmb_held, &rmb_params) {
                send_click(Button::Right);
            }

            if !(lmb.enabled && lmb_held) && !(rmb.enabled && rmb_held) {
                self.sleep.sleep(Duration::from_millis(1));
                continue;
            }

            let earliest = lmb
                .time_until_next(&lmb_params)
                .min(rmb.time_until_next(&rmb_params));

            if earliest > spin_threshold {
                self.sleep.sleep(earliest.saturating_sub(sleep_undershoot));
            } else {
                hint::spin_loop();
            }
        }
    }
}

impl Drop for ClixorApp {
    fn drop(&mut self) {
        input::release_all_buttons();
    }
}

fn main() {
    set_console_title();
    enable_ansi_console();
    install_panic_hook();
    install_ctrlc_handler();
    if let Err(e) = run() {
        eprintln!("\n  {DIM}error:{RESET} {e}");
        process::exit(1);
    }
}

fn run() -> Result<(), ClixorError> {
    let cfg = config::load();
    let lmb_vk = vk::from_str(&cfg.lmb_toggle_key)?;
    let rmb_vk = vk::from_str(&cfg.rmb_toggle_key)?;
    print_banner(&cfg);
    let mut app = ClixorApp::new(cfg)?;
    app.run(lmb_vk, rmb_vk);
    Ok(())
}

fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        eprintln!("\n  {DIM}panic:{RESET} {info}");
        input::release_all_buttons();
    }));
}

fn install_ctrlc_handler() {
    ctrlc::set_handler(|| {
        println!();
        input::release_all_buttons();
        process::exit(0);
    })
    .unwrap_or_else(|e| {
        eprintln!("  {DIM}warning:{RESET} failed to install Ctrl+C handler: {e}");
    });
}

fn enable_ansi_console() {
    unsafe {
        if let Ok(handle) = GetStdHandle(STD_OUTPUT_HANDLE) {
            let mut mode = CONSOLE_MODE::default();
            if GetConsoleMode(handle, &mut mode).is_ok() {
                let _ = SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
            }
        }
    }
}

fn set_console_title() {
    unsafe {
        let _ = SetConsoleTitleW(w!("Clixor"));
    }
}

fn print_status(lmb_enabled: bool, rmb_enabled: bool) {
    let lmb_color = if lmb_enabled { ACCENT } else { DIM };
    let lmb_icon = if lmb_enabled { "● ON " } else { "○ OFF" };
    let rmb_color = if rmb_enabled { ACCENT } else { DIM };
    let rmb_icon = if rmb_enabled { "● ON " } else { "○ OFF" };
    let status = format!(
        "\r  {LABEL}LMB{RESET} {lmb_color}{lmb_icon}{RESET} | \
{LABEL}RMB{RESET} {rmb_color}{rmb_icon}{RESET}          "
    );
    print!("{status}");
    io::stdout().flush().ok();
}

fn print_banner(cfg: &Config) {
    let lmb_key = cfg.lmb_toggle_key.to_uppercase();
    let rmb_key = cfg.rmb_toggle_key.to_uppercase();
    let ramp_ms = cfg.ramp_duration_ms.get();
    let ramp_pct = (cfg.ramp_start_ratio * 100.0) as u32;

    println!();
    println!(
        "{ACCENT}{}{RESET}",
        r#"   ██████╗██╗     ██╗██╗  ██╗ ██████╗ ██████╗ 
  ██╔════╝██║     ██║╚██╗██╔╝██╔═══██╗██╔══██╗
  ██║     ██║     ██║ ╚███╔╝ ██║   ██║██████╔╝
  ██║     ██║     ██║ ██╔██╗ ██║   ██║██╔══██╗
  ╚██████╗███████╗██║██╔╝ ██╗╚██████╔╝██║  ██║
   ╚═════╝╚══════╝╚═╝╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝"#
    );
    println!("  {DIM}by Nimbique · v1.0{RESET}\n");
    println!("  {DIM}────────────────────────────────────{RESET}\n");
    println!(
        "  {LABEL}LMB{RESET} → {VALUE}{:.0} CPS{RESET} [{ACCENT}{}{RESET}] | \
{LABEL}RMB{RESET} → {VALUE}{:.0} CPS{RESET} [{ACCENT}{}{RESET}]",
        cfg.lmb_cps, lmb_key, cfg.rmb_cps, rmb_key
    );
    println!("  {DIM}ramp {VALUE}{}ms{DIM} · start {VALUE}{}%{RESET}\n", ramp_ms, ramp_pct);
    println!("  {DIM}hold button to activate · Ctrl+C to exit{RESET}\n");
}
