//this folder is for status script hooks relating to more general mechanics
use smash::lib::{lua_const::*, L2CValue, L2CAgent};
use smash::lua2cpp::L2CFighterCommon;

use smash::app::BattleObjectModuleAccessor;
use smash::app::lua_bind::*;
use smash::lib::lua_const::*;
use smash::hash40;
use smash::phx::*;
use smash::app::{self, lua_bind::*, sv_kinetic_energy, sv_animcmd};
use smash::app::utility::*;

use crate::utils::*;

use crate::vars::*;

use smash_script::{self, *, macros::*};
use smashline::*;

use hdr_modules::consts::{*, globals::*};
use hdr_modules::*;

//Turn dash (runs once at the beginning of the status)
//fixes "loose" feeling that ult dashdances had... still not perfect but with this its definitely better
#[skyline::hook(replace = smash::lua2cpp::L2CFighterCommon_status_TurnDash_Sub)]
pub unsafe fn status_turndash_sub_hook(fighter: &mut L2CFighterCommon) -> L2CValue {
    let ret = original!()(fighter);

    let mut l2c_agent = L2CAgent::new(fighter.lua_state_agent);
    l2c_agent.clear_lua_stack();
    l2c_agent.push_lua_stack(&mut L2CValue::new_int(*FIGHTER_KINETIC_ENERGY_ID_MOTION as u64));
    l2c_agent.push_lua_stack(&mut L2CValue::new_num(0.0));
    app::sv_kinetic_energy::set_speed(fighter.lua_state_agent);

    ret
}

#[skyline::hook(replace = smash::lua2cpp::L2CFighterCommon_status_TurnDash_Main)]
pub unsafe fn status_turndash_main_hook(fighter: &mut L2CFighterCommon) -> L2CValue {
    let ret = original!()(fighter);
    let mut l2c_agent = L2CAgent::new(fighter.lua_state_agent);
	let mut f_agent = fighter.agent;
	let boma = app::sv_system::battle_object_module_accessor(fighter.lua_state_agent);
	let dash_speed = WorkModule::get_param_float(boma, hash40("dash_speed"), 0) * PostureModule::lr(boma);
	let stick_x = fighter.global_table[STICK_X].get_f32();
	//println!("Current dash speed: {}", dash_speed);

	if MotionModule::frame(boma) >= 1.0 && MotionModule::frame(boma) <= 2.0 && stick_x != 0.0 {
		//println!("Changing current dash speed");
		f_agent.clear_lua_stack();
		f_agent.push_lua_stack(&mut L2CValue::new_int(*FIGHTER_KINETIC_ENERGY_ID_MOTION as u64));
		f_agent.push_lua_stack(&mut L2CValue::new_num(dash_speed));
		sv_kinetic_energy::set_speed(fighter.lua_state_agent);
		f_agent.clear_lua_stack();
	}
	//println!("Current turn dash velocity: {}", KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN));
	//println!("Turn Dash status script main");

	ret
}

#[skyline::hook(replace = smash::lua2cpp::L2CFighterCommon_status_Jump_Main)]
pub unsafe fn status_Jump_Main(fighter: &mut L2CFighterCommon) -> L2CValue {
    sub_air_check_escape_air_snap(fighter, false);

    original!()(fighter)
}


pub unsafe fn sub_air_check_escape_air_snap(fighter: &mut L2CFighterCommon, immediate: bool) {
    if immediate {
        VarModule::set_int(fighter.module_accessor, common::AIR_ESCAPE_MAGNET_FRAME, 0);
        VarModule::on_flag(fighter.module_accessor, common::ENABLE_AIR_ESCAPE_MAGNET);
    } else {
        if !VarModule::is_flag(fighter.module_accessor, common::ENABLE_AIR_ESCAPE_MAGNET)
            && VarModule::countdown_int(fighter.module_accessor, common::AIR_ESCAPE_MAGNET_FRAME, 0) {
            VarModule::on_flag(fighter.module_accessor, common::ENABLE_AIR_ESCAPE_MAGNET);
        }
    }
}
