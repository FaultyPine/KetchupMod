use smash::app::BattleObjectModuleAccessor;
use smash::phx::{Vector2f, Vector3f};
use smash::app::lua_bind::*;
use smash::lib::lua_const::*;
use smash::lib::{lua_const::*, L2CValue, L2CAgent};
use smash::lua2cpp::L2CFighterCommon;
use smash::hash40;

use crate::utils::*;
use crate::vars::custom_vars::*;

//=================================================================
//== JUMPSQUAT MOMENTUM CORRECTION
//== Set flags during jumpsquat for various conditions
//=================================================================
unsafe fn jumpsquat_correction(boma: &mut BattleObjectModuleAccessor, status_kind: i32, situation_kind: i32, curr_frame: f32) {
		// Turn Dash RAR window and Attack Cancel Jumpsquat stuff
		if(status_kind == *FIGHTER_STATUS_KIND_TURN_DASH && [*FIGHTER_STATUS_KIND_DASH, *FIGHTER_STATUS_KIND_TURN_DASH].contains(&StatusModule::prev_status_kind(boma, 0))){
			
			// RAR momentum help for initial dash RAR
			if curr_frame < 2.0 {
				// println!("Within RAR window; turn_dash frame = {}", curr_frame);
				// println!("Current H Speed = {}", KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN));
				irar_window[get_player_number(boma)] = true;
			}
			else{
				irar_window[get_player_number(boma)] = false;
			}
		}
		else if status_kind == *FIGHTER_STATUS_KIND_JUMP_SQUAT {
			// Don't reset the IRAR window flag if in jumpsquat
			if irar_window[get_player_number(boma)] {
				// println!("IRAR jumpsquat");
				irar_jumpsquat[get_player_number(boma)] = true; // We are in irar jumpsquat
			}
			// Clear speed if in IRAR jumpsquat
			if irar_jumpsquat[get_player_number(boma)] {
				KineticModule::clear_speed_all(boma);
			}
		}
		else{
			if situation_kind == SITUATION_KIND_GROUND{
				irar_jumpsquat[get_player_number(boma)] = false;  // No longer in IRAR jumpsquat
			}
			irar_window[get_player_number(boma)] = false;  // The IRAR window has been exited
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

//===================================================================
//== MOMENTUM TRANSFER HELPER
//== Performs some extra calculations to help with momentum handling
//===================================================================
pub unsafe fn momentum_transfer_helper(lua_state: u64, l2c_agent: &mut L2CAgent, boma: &mut BattleObjectModuleAccessor, status_kind: i32, situation_kind: i32, fighter_kind: i32, curr_frame: f32) {

    if [*FIGHTER_STATUS_KIND_TURN_RUN, *FIGHTER_STATUS_KIND_TURN_RUN_BRAKE].contains(&status_kind) {
        rar_leniency[get_player_number(boma)] = clamp(0.8*(MotionModule::end_frame(boma) - MotionModule::frame(boma)*2.0 + 6.0)/MotionModule::end_frame(boma), 0.1, 0.8); // You have a limited amount of time to get full RAR momentum from turn brake or run brake, with a 3F leniency
    }

    if status_kind == *FIGHTER_STATUS_KIND_JUMP_SQUAT /*&& curr_frame <= 1.0*/ {
        js_vel[get_player_number(boma)] = KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN);
    }

    if situation_kind == *SITUATION_KIND_GROUND {
        ground_vel[get_player_number(boma)] = KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN);
    }

    if [*FIGHTER_STATUS_KIND_JUMP_SQUAT, *FIGHTER_STATUS_KIND_JUMP, *FIGHTER_STATUS_KIND_PASS, *FIGHTER_STATUS_KIND_JUMP_AERIAL].contains(&status_kind) {
        curr_momentum[get_player_number(boma)] = KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN);
    }

    if [*FIGHTER_STATUS_KIND_JUMP_SQUAT, *FIGHTER_STATUS_KIND_JUMP].contains(&status_kind) {
        curr_momentum_specials[get_player_number(boma)] = KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN);
    }

}

unsafe fn additional_momentum_transfer_moves(lua_state: u64, l2c_agent: &mut L2CAgent, boma: &mut BattleObjectModuleAccessor, status_kind: i32, situation_kind: i32, fighter_kind: i32, curr_frame: f32) {

    /*      ADDITIONAL MOVES THAT SHOULD CONSERVE MOMENTUM       */

    if situation_kind == *SITUATION_KIND_AIR && curr_frame <= 1.0 {

        //characters whose neutral special should conserve momentum
        let should_conserve_special_momentum =
        ( [*FIGHTER_KIND_CAPTAIN, *FIGHTER_KIND_MARIO, *FIGHTER_KIND_LUIGI, *FIGHTER_KIND_MARIOD, *FIGHTER_KIND_DIDDY, *FIGHTER_KIND_PIKACHU, *FIGHTER_KIND_PICHU, *FIGHTER_KIND_GANON]
          .contains(&fighter_kind) && status_kind == *FIGHTER_STATUS_KIND_SPECIAL_N )
        || ( fighter_kind == *FIGHTER_KIND_DIDDY && [*FIGHTER_DIDDY_STATUS_KIND_SPECIAL_N_CHARGE, *FIGHTER_DIDDY_STATUS_KIND_SPECIAL_N_SHOOT].contains(&status_kind) )
        || ( fighter_kind == *FIGHTER_KIND_KIRBY && [*FIGHTER_STATUS_KIND_SPECIAL_S, *FIGHTER_KIRBY_STATUS_KIND_SPECIAL_S_ATTACK].contains(&status_kind) );

        if should_conserve_special_momentum && KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN).abs() > 0.1 {
            l2c_agent.clear_lua_stack();
            l2c_agent.push_lua_stack(&mut L2CValue::new_int(*FIGHTER_KINETIC_ENERGY_ID_CONTROL as u64));
            l2c_agent.push_lua_stack(&mut L2CValue::new_num(curr_momentum[get_player_number(boma)]));
            smash::app::sv_kinetic_energy::set_speed(lua_state);
            l2c_agent.clear_lua_stack();
        }

    }
}


pub unsafe fn run(lua_state: u64, l2c_agent: &mut L2CAgent, boma: &mut BattleObjectModuleAccessor, status_kind: i32, situation_kind: i32, fighter_kind: i32) {
    let curr_frame = MotionModule::frame(boma);
    jumpsquat_correction(boma, status_kind, situation_kind, curr_frame);
    momentum_transfer_helper(lua_state, l2c_agent, boma, status_kind, situation_kind, fighter_kind, curr_frame);
    additional_momentum_transfer_moves(lua_state, l2c_agent, boma, status_kind, situation_kind, fighter_kind, curr_frame);
}
