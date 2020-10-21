use smash::app::{self, lua_bind::*, sv_kinetic_energy};
use smash::lib::{lua_const::*, L2CValue, L2CAgent};
use smash::lua2cpp::L2CFighterCommon;
use smash::hash40;
use crate::utils::*;

//Jump (runs once at the beginning of the status)
#[skyline::hook(replace = smash::lua2cpp::L2CFighterCommon_status_Jump_sub)]
pub unsafe fn status_jump_sub_hook(fighter: &mut L2CFighterCommon, param_2: L2CValue, param_3: L2CValue) -> L2CValue {
    let boma = app::sv_system::battle_object_module_accessor(fighter.lua_state_agent);
    let mut l2c_agent = L2CAgent::new(fighter.lua_state_agent);

    l2c_agent.clear_lua_stack();
    l2c_agent.push_lua_stack(&mut L2CValue::new_int(*FIGHTER_KINETIC_ENERGY_ID_CONTROL as u64));
    l2c_agent.push_lua_stack(&mut L2CValue::new_num(calc_melee_momentum(boma, false)));
    sv_kinetic_energy::set_speed(fighter.lua_state_agent);


    original!()(fighter, param_2, param_3)
}


//Aerials (runs once at the beginning of the status)
#[skyline::hook(replace = smash::lua2cpp::L2CFighterCommon_sub_attack_air_common)]
pub unsafe fn status_attack_air_hook(fighter: &mut L2CFighterCommon, param_1: L2CValue){
    let boma = app::sv_system::battle_object_module_accessor(fighter.lua_state_agent);
    let mut l2c_agent = L2CAgent::new(fighter.lua_state_agent);
    let is_speed_backward = KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN) * PostureModule::lr(boma) < 0.0;
    let prev_status_check = [*FIGHTER_STATUS_KIND_FALL, *FIGHTER_STATUS_KIND_JUMP, *FIGHTER_STATUS_KIND_JUMP_SQUAT].contains(&StatusModule::prev_status_kind(boma, 0));    
    let mut new_speed = CURR_MOMENTUM[get_player_number(boma)];


        /*      Shorthop aerial macro and "bair stick flick" fix     */
    if WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_FRAME_IN_AIR) <= 1 && 
        StatusModule::prev_status_kind(boma, 1) == *FIGHTER_STATUS_KIND_JUMP_SQUAT && !is_speed_backward { //if you used the shorthop aerial macro
        new_speed = calc_melee_momentum(boma, true);
    }

    if prev_status_check {
        l2c_agent.clear_lua_stack();
        l2c_agent.push_lua_stack(&mut L2CValue::new_int(*FIGHTER_KINETIC_ENERGY_ID_CONTROL as u64));
        l2c_agent.push_lua_stack(&mut L2CValue::new_num(new_speed));
        sv_kinetic_energy::set_speed(fighter.lua_state_agent);
    }

    original!()(fighter, param_1)
}


//called in moveset_edits in sys_line_system_control_fighter.rs
use crate::vars::custom_vars::{CURR_MOMENTUM, RAR_LENIENCY};
pub unsafe fn momentum_transfer_helper(lua_state: u64, l2c_agent: &mut L2CAgent, boma: &mut app::BattleObjectModuleAccessor, status_kind: i32, situation_kind: i32, curr_frame: f32, fighter_kind: i32) {
    if [*FIGHTER_STATUS_KIND_JUMP_SQUAT, *FIGHTER_STATUS_KIND_JUMP, *FIGHTER_STATUS_KIND_FALL].contains(&status_kind) {
        CURR_MOMENTUM[get_player_number(boma)] = KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN); 
    }

    if [*FIGHTER_STATUS_KIND_TURN_RUN, *FIGHTER_STATUS_KIND_RUN_BRAKE, *FIGHTER_STATUS_KIND_TURN_RUN_BRAKE].contains(&status_kind){
		RAR_LENIENCY[get_player_number(boma)] = clamp(0.8*(MotionModule::end_frame(boma) - MotionModule::frame(boma)*2.0 + 6.0)/MotionModule::end_frame(boma), 0.1, 0.8); // You have a limited amount of time to get full RAR momentum from turn brake or run brake, with a 3F leniency
	}

            /*      ADDITIONAL MOVES THAT SHOULD CONSERVE MOMENTUM       */
    let mut should_conserve_momentum = false;
    
    if situation_kind == *SITUATION_KIND_AIR && curr_frame <= 1.0 {

        if [*FIGHTER_KIND_CAPTAIN, *FIGHTER_KIND_MARIO, *FIGHTER_KIND_LUIGI]
            .contains(&fighter_kind) && status_kind == *FIGHTER_STATUS_KIND_SPECIAL_N { //put any fighter here whose neutral special should conserve momentum
                should_conserve_momentum = true; //spacie lasers, falcon punch, 
        }

        if should_conserve_momentum && KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN).abs() > 0.1 {
            l2c_agent.clear_lua_stack();
            l2c_agent.push_lua_stack(&mut L2CValue::new_int(*FIGHTER_KINETIC_ENERGY_ID_CONTROL as u64));
            l2c_agent.push_lua_stack(&mut L2CValue::new_num(CURR_MOMENTUM[get_player_number(boma)]));
            sv_kinetic_energy::set_speed(lua_state);
        }

    }

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


unsafe fn calc_melee_momentum(boma: &mut app::BattleObjectModuleAccessor, is_aerial_attack: bool) -> f32 {
    let jump_speed_x = WorkModule::get_param_float(boma, hash40("jump_speed_x"), 0);
    let jump_speed_x_mul = WorkModule::get_param_float(boma, hash40("jump_speed_x_mul"), 0);
	let air_speed_x_stable = WorkModule::get_param_float(boma, hash40("air_speed_x_stable"), 0);
    let stick_x = ControlModule::get_stick_x(boma);
    let x_vel = KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN);
    let jump_speed_x_max = WorkModule::get_param_float(boma, hash40("jump_speed_x_max"), 0);

	let mut airSpeedCalc = 0.0;
	if stick_x != 0.0 {
        airSpeedCalc = air_speed_x_stable*(stick_x.signum());
    }
	let jumpSpeedRemainder = ((x_vel*jump_speed_x_mul + jump_speed_x*x_vel.signum()) - airSpeedCalc)*stick_x;

	let mut jumpSpeedRemainderAdd = 0.0;
	if jumpSpeedRemainder.abs() >= 0.0 {
		jumpSpeedRemainderAdd = jumpSpeedRemainder.abs();
	}

	let mut calcJumpSpeed;
	if    [*FIGHTER_STATUS_KIND_DASH, *FIGHTER_STATUS_KIND_RUN, *FIGHTER_STATUS_KIND_TURN_DASH].contains(&StatusModule::prev_status_kind(boma, 1)) && !is_aerial_attack
	   || [*FIGHTER_STATUS_KIND_DASH, *FIGHTER_STATUS_KIND_RUN, *FIGHTER_STATUS_KIND_TURN_DASH].contains(&StatusModule::prev_status_kind(boma, 2)) && is_aerial_attack { // Full RAR momentum calculation with full leniency only if you are in a dash/run state
		calcJumpSpeed = (jumpSpeedRemainderAdd * stick_x) + (x_vel);
		if stick_x * PostureModule::lr(boma) < 0.0 {
			calcJumpSpeed = (jumpSpeedRemainderAdd * stick_x * 0.1) + (x_vel);
		}
	}
	else if [*FIGHTER_STATUS_KIND_TURN_RUN, *FIGHTER_STATUS_KIND_TURN_RUN_BRAKE, *FIGHTER_STATUS_KIND_RUN_BRAKE].contains(&StatusModule::prev_status_kind(boma, 1)) && !is_aerial_attack
			|| [*FIGHTER_STATUS_KIND_TURN_RUN, *FIGHTER_STATUS_KIND_TURN_RUN_BRAKE, *FIGHTER_STATUS_KIND_RUN_BRAKE].contains(&StatusModule::prev_status_kind(boma, 2)) && is_aerial_attack {
		calcJumpSpeed = (jumpSpeedRemainderAdd * stick_x * RAR_LENIENCY[get_player_number(boma)]) + (x_vel);
	}
	else if [*FIGHTER_STATUS_KIND_RUN_BRAKE].contains(&StatusModule::prev_status_kind(boma, 1)) && !is_aerial_attack
			|| [*FIGHTER_STATUS_KIND_RUN_BRAKE].contains(&StatusModule::prev_status_kind(boma, 2)) && is_aerial_attack{
		calcJumpSpeed = (jumpSpeedRemainderAdd * stick_x) + (x_vel);
		if stick_x * PostureModule::lr(boma) < 0.0 {
			calcJumpSpeed = (jumpSpeedRemainderAdd * stick_x * 0.1 * RAR_LENIENCY[get_player_number(boma)]) + (x_vel);
		}
	}
	else{ // In all other states, only a bit of extra stick momentum is added
		calcJumpSpeed =(jumpSpeedRemainderAdd * stick_x * 0.2) + (x_vel);
	}

    let jumpSpeedClamped = clamp(calcJumpSpeed, -jump_speed_x_max, jump_speed_x_max);  //melee jump speed calculation... courtesey of Brawltendo
    jumpSpeedClamped
}