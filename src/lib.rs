#![feature(proc_macro_hygiene)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_variables)]

mod momentum_transfer;
mod sys_line;
mod ecbs;
mod init_settings;
mod edge_slipoffs;
mod status_script_hooks;
mod get_command_flag_cat;

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
    skyline::nro::add_hook(nro_main).unwrap();

    // main hooks
    skyline::install_hooks!(
        momentum_transfer::change_kinetic_hook,
        ecbs::get_ground_correct_kind_air_trans_hook,
        ecbs::get_rhombus_hook,
        init_settings::init_settings_hook,
        edge_slipoffs::correct_hook,
        get_command_flag_cat::get_command_flag_cat_hook
    );

    sys_line::install();
}
