#![feature(proc_macro_hygiene)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_variables)]

mod momentum_transfer;
mod sys_line;
mod init_settings;
mod edge_slipoffs;
mod status_script_hooks;
mod get_command_flag_cat;
mod get_param;

mod utils;
mod moveset_utils;
mod vars;

// nro hooks
fn nro_main(nro: &skyline::nro::NroInfo) {
    match nro.name {
        "common" => {
            skyline::install_hooks!(
                momentum_transfer::status_jump_sub_hook,
                momentum_transfer::status_attack_air_hook,
                status_script_hooks::status_turndash_sub_hook
            );
        }
        _ => (),
    }
}

#[skyline::main(name = "KetchupMod")]
pub fn main() {

    // Check if an update is available
    println!("[KetchupMod] Checking update server...");
    if skyline_update::check_update("3.17.96.120".parse().unwrap(), "KetchupMod", env!("CARGO_PKG_VERSION"), false) {
        println!("[KetchupMod] Installed new update!");
        skyline::nn::oe::RestartProgramNoArgs();
    } else { println!("[KetchupMod] Local and server versions match. No update needed"); }
    
    println!("========= KetchupMod Ver. {} =========", env!("CARGO_PKG_VERSION"));



    skyline::nro::add_hook(nro_main).unwrap();

    // main hooks
    skyline::install_hooks!(
        momentum_transfer::change_kinetic_hook,
        init_settings::init_settings_hook,
        edge_slipoffs::correct_hook,
        get_param::get_param_float_middle,
        get_command_flag_cat::get_command_flag_cat_hook
    );

    sys_line::install();
}
