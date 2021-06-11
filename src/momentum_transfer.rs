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

use smash_script::*;

use hdr_modules::consts::{*, globals::*};
use hdr_modules::VarModule;

/* Moves that should bypass the momentum logic (in terms of the jump status script) */
const MOMENTUM_EXCEPTION_MOVES: [smash::lib::LuaConst ; 1] = [
    FIGHTER_SONIC_STATUS_KIND_SPIN_JUMP
];

//Jump (runs once at the beginning of the status)
#[skyline::hook(replace = smash::lua2cpp::L2CFighterCommon_status_Jump_sub)]
pub unsafe fn status_jump_sub_hook(fighter: &mut L2CFighterCommon, param_2: L2CValue, param_3: L2CValue) -> L2CValue {
    let boma = app::sv_system::battle_object_module_accessor(fighter.lua_state_agent);
    let mut l2c_agent = L2CAgent::new(fighter.lua_state_agent);
    let fighter_kind = get_kind(boma);
    //println!("Pre-jump horizontal velocity: {}", KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN));

    if !MOMENTUM_EXCEPTION_MOVES.iter().any(|x| *x == StatusModule::status_kind(boma) ) {
        l2c_agent.clear_lua_stack();
        l2c_agent.push_lua_stack(&mut L2CValue::new_int(*FIGHTER_KINETIC_ENERGY_ID_CONTROL as u64));
        l2c_agent.push_lua_stack(&mut L2CValue::new_num(calc_melee_momentum(boma, false, false, false)));
        sv_kinetic_energy::set_speed(fighter.lua_state_agent);
        l2c_agent.clear_lua_stack();
        //println!("Post-jump horizontal velocity: {}", KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN));
        VarModule::set_float(boma, common::CURRENT_MOMENTUM, KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN)); // Set the current momentum to what was just calculated
    }

    original!()(fighter, param_2, param_3)

}


//Aerials (runs once at the beginning of the status)
#[skyline::hook(replace = smash::lua2cpp::L2CFighterCommon_sub_attack_air_common)]
pub unsafe fn status_attack_air_hook(fighter: &mut L2CFighterCommon, param_1: L2CValue){
    let boma = app::sv_system::battle_object_module_accessor(fighter.lua_state_agent);
    let fighter_kind = get_kind(boma);
    let jump_speed_x_max = WorkModule::get_param_float(boma, hash40("run_speed_max"), 0) * jump_speed_ratio[get_player_number(boma)];

    let mut l2c_agent = L2CAgent::new(fighter.lua_state_agent);
    let is_speed_backward = KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN) * PostureModule::lr(boma) < 0.0;
    let prev_status_check = [*FIGHTER_STATUS_KIND_JUMP, *FIGHTER_STATUS_KIND_JUMP_SQUAT].contains(&StatusModule::prev_status_kind(boma, 0));
    let mut new_speed = clamp(VarModule::get_float(fighter.module_accessor, common::CURRENT_MOMENTUM), -jump_speed_x_max, jump_speed_x_max);

    if sh_macro[get_player_number(boma)] && VarModule::get_float(fighter.module_accessor, common::JUMPSQUAT_VELOCITY).abs() > 0.0 && prev_status_check {
        new_speed = calc_melee_momentum(boma, false, true, false);
    }

    if prev_status_check {
        fighter.clear_lua_stack();
        lua_args!(fighter, FIGHTER_KINETIC_ENERGY_ID_CONTROL, new_speed);
        app::sv_kinetic_energy::set_speed(fighter.lua_state_agent);
        fighter.clear_lua_stack();
    }

    original!()(fighter, param_1)
}

//=================================================================
//== JUMPSQUAT MOMENTUM CORRECTION
//== Set flags during jumpsquat for various conditions
//=================================================================
pub unsafe fn jumpsquat_correction(fighter: &mut L2CFighterCommon, boma: &mut BattleObjectModuleAccessor, status_kind: i32, situation_kind: i32, curr_frame: f32) {
		// Turn Dash RAR window and Attack Cancel Jumpsquat stuff
		if(status_kind == *FIGHTER_STATUS_KIND_TURN_DASH && [*FIGHTER_STATUS_KIND_DASH, *FIGHTER_STATUS_KIND_TURN_DASH].contains(&StatusModule::prev_status_kind(boma, 0))){
			// RAR momentum help for initial dash RAR
			if curr_frame < 2.0 {
				// println!("Within RAR window; turn_dash frame = {}", curr_frame);
				// println!("Current H Speed = {}", KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN));
				VarModule::set_flag(fighter.module_accessor, common::IRAR_WINDOW, true);
			}
			else{
				VarModule::set_flag(fighter.module_accessor, common::IRAR_WINDOW, false);
			}
		}
		else if status_kind == *FIGHTER_STATUS_KIND_JUMP_SQUAT {
			// Don't reset the IRAR window flag if in jumpsquat
			if VarModule::is_flag(fighter.module_accessor, common::IRAR_WINDOW) {
				// println!("IRAR jumpsquat");
				VarModule::set_flag(fighter.module_accessor, common::IRAR_JUMPSQUAT, true); // We are in irar jumpsquat
			}
			// Clear speed if in IRAR jumpsquat
			if VarModule::is_flag(fighter.module_accessor, common::IRAR_JUMPSQUAT) {
			    use smash_script::*;
			    lua_args!(fighter, FIGHTER_KINETIC_ENERGY_ID_CONTROL);
			    let speed_y = smash::app::sv_kinetic_energy::get_speed_y(fighter.lua_state_agent);
			    smash_script::sv_kinetic_energy!(set_speed, fighter, FIGHTER_KINETIC_ENERGY_ID_CONTROL, 0, speed_y);
			}
		}
		else{
			if situation_kind == SITUATION_KIND_GROUND{
				VarModule::set_flag(fighter.module_accessor, common::IRAR_JUMPSQUAT, false);  // No longer in IRAR jumpsquat
			}
			VarModule::set_flag(fighter.module_accessor, common::IRAR_WINDOW, false);  // The IRAR window has been exited
		}

		// Track short hop aerial shenanigans
		if WorkModule::is_flag(boma, *FIGHTER_INSTANCE_WORK_ID_FLAG_JUMP_MINI_ATTACK) {
			// println!("Short hop macro tracked");
			sh_macro[get_player_number(boma)] = true;
		}
		if situation_kind == SITUATION_KIND_GROUND && status_kind != *FIGHTER_STATUS_KIND_JUMP_SQUAT /*FIGHTER_STATUS_KIND_JUMP_SQUAT*/ {
			sh_macro[get_player_number(boma)] = false;
		}
}

//called in moveset_edits in sys_line_system_control_fighter.rs
pub unsafe fn momentum_transfer_helper(fighter: &mut L2CFighterCommon, lua_state: u64, l2c_agent: &mut L2CAgent, boma: &mut BattleObjectModuleAccessor, status_kind: i32, situation_kind: i32, fighter_kind: i32, curr_frame: f32) {

	if situation_kind == *SITUATION_KIND_AIR && ![*FIGHTER_STATUS_KIND_JUMP_SQUAT, *FIGHTER_STATUS_KIND_JUMP, *FIGHTER_STATUS_KIND_ESCAPE_AIR].contains(&status_kind) {
		VarModule::on_flag(fighter.module_accessor, common::ENABLE_AIR_ESCAPE_MAGNET);
	}

	if (fighter_kind == *FIGHTER_KIND_PFUSHIGISOU && status_kind == *FIGHTER_PFUSHIGISOU_STATUS_KIND_SPECIAL_LW_STANDBY)
		|| (fighter_kind == *FIGHTER_KIND_PLIZARDON && status_kind == *FIGHTER_PLIZARDON_STATUS_KIND_SPECIAL_LW_STANDBY)
		|| (fighter_kind == *FIGHTER_KIND_PZENIGAME && status_kind == *FIGHTER_PZENIGAME_STATUS_KIND_SPECIAL_LW_STANDBY)
		|| (fighter_kind == *FIGHTER_KIND_EFLAME && status_kind == *FIGHTER_EFLAME_STATUS_KIND_SPECIAL_LW_STANDBY)
		|| (fighter_kind == *FIGHTER_KIND_ELIGHT && status_kind == *FIGHTER_ELIGHT_STATUS_KIND_SPECIAL_LW_STANDBY) {
		jump_speed_ratio[get_player_number(boma)] = (WorkModule::get_param_float(boma, hash40("jump_speed_x_max"), 0) / WorkModule::get_param_float(boma, hash40("run_speed_max"), 0));
	}

	if [*FIGHTER_STATUS_KIND_TURN_RUN, *FIGHTER_STATUS_KIND_TURN_RUN_BRAKE].contains(&status_kind) {
        rar_leniency[get_player_number(boma)] = clamp(0.8*(MotionModule::end_frame(boma) - MotionModule::frame(boma)*2.0 + 6.0)/MotionModule::end_frame(boma), 0.1, 0.8); // You have a limited amount of time to get full RAR momentum from turn brake or run brake, with a 3F leniency
    }

    if situation_kind == *SITUATION_KIND_GROUND {
        VarModule::set_float(boma, common::GROUND_VELOCITY, KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN));
    }

    if [*FIGHTER_STATUS_KIND_JUMP, *FIGHTER_STATUS_KIND_PASS, *FIGHTER_STATUS_KIND_JUMP_AERIAL].contains(&status_kind) {
        VarModule::set_float(boma, common::CURRENT_MOMENTUM, KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN));
    }

    if [*FIGHTER_STATUS_KIND_JUMP].contains(&status_kind) {
        VarModule::set_float(boma, common::CURRENT_MOMENTUM_SPECIALS, KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN));
    }

}

pub unsafe fn additional_momentum_transfer_moves(fighter: &mut L2CFighterCommon, lua_state: u64, l2c_agent: &mut L2CAgent, boma: &mut BattleObjectModuleAccessor, status_kind: i32, situation_kind: i32, fighter_kind: i32, curr_frame: f32) {

    /*      ADDITIONAL MOVES THAT SHOULD CONSERVE MOMENTUM       */

    if situation_kind == *SITUATION_KIND_AIR && curr_frame <= 1.0 {

        //characters whose neutral special should conserve momentum
        let should_conserve_special_momentum =
        ( [*FIGHTER_KIND_MARIO, *FIGHTER_KIND_LUIGI, *FIGHTER_KIND_MARIOD, *FIGHTER_KIND_DIDDY, *FIGHTER_KIND_PIKACHU, *FIGHTER_KIND_PICHU, *FIGHTER_KIND_GANON]
          .contains(&fighter_kind) && status_kind == *FIGHTER_STATUS_KIND_SPECIAL_N )
        || ( fighter_kind == *FIGHTER_KIND_DIDDY && [*FIGHTER_DIDDY_STATUS_KIND_SPECIAL_N_CHARGE, *FIGHTER_DIDDY_STATUS_KIND_SPECIAL_N_SHOOT].contains(&status_kind) )
        || ( fighter_kind == *FIGHTER_KIND_KIRBY && [*FIGHTER_STATUS_KIND_SPECIAL_S, *FIGHTER_KIRBY_STATUS_KIND_SPECIAL_S_ATTACK].contains(&status_kind) );

		if should_conserve_special_momentum && KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN).abs() > 0.1 {
            if StatusModule::prev_status_kind(boma, 0) != *FIGHTER_STATUS_KIND_JUMP {
                VarModule::set_float(boma, common::CURRENT_MOMENTUM_SPECIALS, KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN));
            }
			let new_speed = VarModule::get_float(fighter.module_accessor, common::CURRENT_MOMENTUM_SPECIALS);
			fighter.clear_lua_stack();
	        lua_args!(fighter, FIGHTER_KINETIC_ENERGY_ID_CONTROL, new_speed);
	        app::sv_kinetic_energy::set_speed(fighter.lua_state_agent);
	        fighter.clear_lua_stack();
        }

    }
}

                /*      SPACIE LASER MOMENTUM   */

//called in double_jump_cancels.rs in the change_kinetic hook
pub unsafe fn change_kinetic_momentum_related(boma: &mut smash::app::BattleObjectModuleAccessor, kinetic_type: i32) -> Option<i32> { //spacie laser momentum conservation
    let status_kind = StatusModule::status_kind(boma);
    let prev_status_kind = StatusModule::prev_status_kind(boma, 0);
    let situation_kind = StatusModule::situation_kind(boma);
    let fighter_kind = smash::app::utility::get_kind(boma);
    if [*FIGHTER_KIND_CAPTAIN, *FIGHTER_KIND_FALCO, *FIGHTER_KIND_FOX, *FIGHTER_KIND_GAMEWATCH, *FIGHTER_KIND_WOLF].contains(&fighter_kind) && status_kind == *FIGHTER_STATUS_KIND_SPECIAL_N
        && situation_kind == *SITUATION_KIND_AIR && [*FIGHTER_STATUS_KIND_JUMP, *FIGHTER_STATUS_KIND_JUMP_SQUAT].contains(&prev_status_kind) {
        return Some(-1);
    }
    None
}

#[skyline::hook(replace = KineticModule::change_kinetic)]
pub unsafe fn change_kinetic_hook(boma: &mut app::BattleObjectModuleAccessor, kinetic_type: i32) -> Option<i32> { //spacie laser momentum conservation
    let mut kinetic_type_new = kinetic_type;
    let status_kind = StatusModule::status_kind(boma);
    let fighter_kind = app::utility::get_kind(boma);
    let mut should_change_kinetic = false;

    match change_kinetic_momentum_related(boma, kinetic_type_new) {
        Some(x) => kinetic_type_new = x,
        None => ()
    }

    original!()(boma, kinetic_type_new)
}


unsafe fn calc_melee_momentum(boma: &mut BattleObjectModuleAccessor, aerial_attack: bool, attack_cancel: bool, walking: bool) -> f32 {
    let fighter_kind = get_kind(boma);
	let id = get_player_number(boma);
    let jump_speed_x = WorkModule::get_param_float(boma, hash40("jump_speed_x"), 0);
    let jump_speed_x_mul = WorkModule::get_param_float(boma, hash40("jump_speed_x_mul"), 0);
    let air_speed_x_stable = WorkModule::get_param_float(boma, hash40("air_speed_x_stable"), 0);
    let dash_speed = WorkModule::get_param_float(boma, hash40("dash_speed"), 0);
    let run_speed_max = WorkModule::get_param_int(boma, hash40("run_speed_max"), 0);
    let walk_speed_max = WorkModule::get_param_float(boma, hash40("walk_speed_max"), 0);
	let js_frames = WorkModule::get_param_float(boma, hash40("jump_squat_frame"), 0);
    let traction = WorkModule::get_param_float(boma, hash40("ground_brake"), 0);
    let stick_x = ControlModule::get_stick_x(boma);
    //let x_vel = KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN);

	let jump_speed_x_max = WorkModule::get_param_float(boma, hash40("run_speed_max"), 0) * jump_speed_ratio[id];

	/*
	if [*FIGHTER_KIND_BRAVE, *FIGHTER_KIND_WIIFIT, *FIGHTER_KIND_SHULK].contains(&fighter_kind){
		jump_speed_x_max = WorkModule::get_param_float(boma, hash40("run_speed_max"), 0) * jump_speed_ratio[get_player_number(boma)];
	}
	*/

    let is_speed_backward = KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN) * PostureModule::lr(boma) < 0.0;

    let mut x_vel = KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN);

    if StatusModule::prev_status_kind(boma, 0) == *FIGHTER_STATUS_KIND_JUMP_SQUAT {
        //println!("Jumpsquat momentum...");
        x_vel = VarModule::get_float(boma, common::JUMPSQUAT_VELOCITY);
    }


    //println!("Current run speed: {}", run_speed_max);

    //println!("jumpsquat velocity: {}", x_vel);
    //println!("current velocity: {}", KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN));

    let mut calcJumpSpeed = (x_vel * jump_speed_x_mul) + (jump_speed_x * stick_x);  // Calculate jump momentum based on the momentum you had on the last frame of jumpsquat

    if VarModule::is_flag(boma, common::IRAR_JUMPSQUAT) && stick_x * PostureModule::lr(boma) < 0.0{ // If within the initial dash RAR window and your stick is backwards during jumpsquat
        calcJumpSpeed = ( (jump_speed_x_mul * (dash_speed - (js_frames as f32 * traction)) * PostureModule::lr(boma) * -1.0) + (stick_x * jump_speed_x) ) * 0.8; // Subtract four frames' worth of traction from dash speed to simulate jumpsquat
    }

    // Helper momentum for attack cancel aerials
    if attack_cancel {
        //println!("Attack Cancel! Calculated jump speed so far: {}", calcJumpSpeed);
        // If Attack Cancel RAR
        if    [*FIGHTER_STATUS_KIND_DASH, *FIGHTER_STATUS_KIND_TURN_DASH].contains(&StatusModule::prev_status_kind(boma, 2)) {
            //println!("Initial Dash Attack Cancel");
            /*
            if dash_speed > run_speed_max{
                calcJumpSpeed = ((jump_speed_x * x_vel/dash_speed) + (jump_speed_x_mul * x_vel));
            }
            else{
                calcJumpSpeed = ((jump_speed_x * x_vel/run_speed_max) + (jump_speed_x_mul * x_vel));
            }
            */

        }
        /*
        if walking{
            calcJumpSpeed = ((jump_speed_x * x_vel/walk_speed_max) + (jump_speed_x_mul * x_vel));
        }
        */
    }

    let jumpSpeedClamped = clamp(calcJumpSpeed, -jump_speed_x_max, jump_speed_x_max);  //melee jump speed calculation... courtesey of Brawltendo

    jumpSpeedClamped
}
