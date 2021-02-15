use smash::app::BattleObjectModuleAccessor;
use smash::app::lua_bind::*;
use smash::lib::lua_const::*;
use smash::hash40;
use smash::app::{self, lua_bind::*, sv_kinetic_energy, sv_animcmd};
use smash::lib::{lua_const::*, L2CValue, L2CAgent};
use smash::lua2cpp::L2CFighterCommon;
use smash::app::utility::*;

use crate::utils::*;
use crate::vars::custom_vars::*;

//===================================================================
//== MOMENTUM TRANSFER
//== The chonky meat of the code; includes some status script hooks
//===================================================================

//Jump (runs once at the beginning of the status)
#[skyline::hook(replace = smash::lua2cpp::L2CFighterCommon_status_Jump_sub)]
pub unsafe fn status_jump_sub_hook(fighter: &mut L2CFighterCommon, param_2: L2CValue, param_3: L2CValue) -> L2CValue {
    let boma = app::sv_system::battle_object_module_accessor(fighter.lua_state_agent);
    let mut l2c_agent = L2CAgent::new(fighter.lua_state_agent);
    let fighter_kind = get_kind(boma);

    l2c_agent.clear_lua_stack();
    l2c_agent.push_lua_stack(&mut L2CValue::new_int(*FIGHTER_KINETIC_ENERGY_ID_CONTROL as u64));
    l2c_agent.push_lua_stack(&mut L2CValue::new_num(calc_melee_momentum(boma, false, false, false)));
    sv_kinetic_energy::set_speed(fighter.lua_state_agent);
    l2c_agent.clear_lua_stack();
    curr_momentum[get_player_number(boma)] = KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN); // Set the current momentum to what was just calculated

    original!()(fighter, param_2, param_3)

}

//Aerials (runs once at the beginning of the status)
#[skyline::hook(replace = smash::lua2cpp::L2CFighterCommon_sub_attack_air_common)]
pub unsafe fn status_attack_air_hook(fighter: &mut L2CFighterCommon, param_1: L2CValue){
    let lua_state = fighter.lua_state_agent;
    let boma = smash::app::sv_system::battle_object_module_accessor(lua_state);
    let fighter_kind = get_kind(boma);
    let jump_speed_x_max = WorkModule::get_param_float(boma, hash40("jump_speed_x_max"), 0);
    let boma = app::sv_system::battle_object_module_accessor(fighter.lua_state_agent);

    let mut l2c_agent = L2CAgent::new(fighter.lua_state_agent);
    let is_speed_backward = KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN) * PostureModule::lr(boma) < 0.0;
    let prev_status_check = [*FIGHTER_STATUS_KIND_JUMP, *FIGHTER_STATUS_KIND_JUMP_SQUAT].contains(&StatusModule::prev_status_kind(boma, 0));
    let mut new_speed = clamp(curr_momentum[get_player_number(boma)], -jump_speed_x_max, jump_speed_x_max);

    if sh_macro[get_player_number(boma)] && js_vel[get_player_number(boma)].abs() > 0.0 && prev_status_check {
        new_speed = calc_melee_momentum(boma, false, true, false);
    }

    if prev_status_check {
        l2c_agent.clear_lua_stack();
        l2c_agent.push_lua_stack(&mut L2CValue::new_int(*FIGHTER_KINETIC_ENERGY_ID_CONTROL as u64));
        l2c_agent.push_lua_stack(&mut L2CValue::new_num(new_speed));
        sv_kinetic_energy::set_speed(fighter.lua_state_agent);
        l2c_agent.clear_lua_stack();
    }

    original!()(fighter, param_1)
}

                /*      SPACIE LASER MOMENTUM   */

//called in double_jump_cancels.rs in the change_kinetic hook
#[skyline::hook(replace = KineticModule::change_kinetic)]
pub unsafe fn change_kinetic_hook(boma: &mut app::BattleObjectModuleAccessor, kinetic_type: i32) -> Option<i32> { //spacie laser momentum conservation
    let mut kinetic_type_new = kinetic_type;
    let status_kind = StatusModule::status_kind(boma);
    let fighter_kind = app::utility::get_kind(boma);
    let mut should_change_kinetic = false;

    if [*FIGHTER_KIND_FALCO, *FIGHTER_KIND_FOX].contains(&fighter_kind) && status_kind == 446 /* laser status */ { 
        should_change_kinetic = true;
    }

    if [*FIGHTER_KINETIC_TYPE_FALL].contains(&kinetic_type_new) && should_change_kinetic {
        kinetic_type_new = -1;
    }

    original!()(boma, kinetic_type_new)
}

unsafe fn calc_melee_momentum(boma: &mut BattleObjectModuleAccessor, aerial_attack: bool, attack_cancel: bool, walking: bool) -> f32 {
    let id = get_player_number(boma);
    let jump_speed_x = WorkModule::get_param_float(boma, hash40("jump_speed_x"), 0);
    let jump_speed_x_mul = WorkModule::get_param_float(boma, hash40("jump_speed_x_mul"), 0);
    let air_speed_x_stable = WorkModule::get_param_float(boma, hash40("air_speed_x_stable"), 0);
    let dash_speed = WorkModule::get_param_float(boma, hash40("dash_speed"), 0);
    let run_speed_max = WorkModule::get_param_float(boma, hash40("run_speed_max"), 0);
    let walk_speed_max = WorkModule::get_param_float(boma, hash40("walk_speed_max"), 0);
    let traction = WorkModule::get_param_float(boma, hash40("ground_brake"), 0);
    let stick_x = ControlModule::get_stick_x(boma);
    //let x_vel = KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN);
    let jump_speed_x_max = WorkModule::get_param_float(boma, hash40("jump_speed_x_max"), 0);// * (run_speed_max / base_run_speed_max[id]); // To account for characters whose stats can change mid-match
    let is_speed_backward = KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN) * PostureModule::lr(boma) < 0.0;

    let mut x_vel = KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN);

    if StatusModule::prev_status_kind(boma, 0) == *FIGHTER_STATUS_KIND_JUMP_SQUAT {
        x_vel = js_vel[id];
    }

    let mut calcJumpSpeed = (x_vel * jump_speed_x_mul) + (jump_speed_x * stick_x);  // Calculate jump momentum based on the momentum you had on the last frame of jumpsquat

    if irar_jumpsquat[get_player_number(boma)] && stick_x * PostureModule::lr(boma) < 0.0 { // If within the initial dash RAR window and your stick is backwards during jumpsquat
        calcJumpSpeed = ( (jump_speed_x_mul * (dash_speed - (4.0 * traction)) * PostureModule::lr(boma) * -1.0) + (stick_x * jump_speed_x) ) * 0.8; // Subtract four frames' worth of traction from dash speed to simulation jumpsquat
    }

    let jumpSpeedClamped = clamp(calcJumpSpeed, -jump_speed_x_max, jump_speed_x_max);  //melee jump speed calculation... courtesey of Brawltendo

    jumpSpeedClamped
}