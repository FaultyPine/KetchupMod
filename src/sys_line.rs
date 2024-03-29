use smash::app::{self, lua_bind::*};
use smash::lib::{L2CAgent, lua_const::*};
use smash::hash40;
use smash::lua2cpp::L2CFighterCommon;
use crate::utils::*;

use hdr_modules::consts::{*, globals::*};
use hdr_modules::*;

use crate::vars::custom_vars::*;

pub fn install() {
    smashline::install_agent_frames!(sys_line_system_control_fighter_hook);
}


//#[skyline::hook(replace = smash::lua2cpp::L2CFighterCommon_sys_line_system_control_fighter)]
#[smashline::fighter_frame(global)]
pub unsafe fn sys_line_system_control_fighter_hook(fighter: &mut L2CFighterCommon) /*-> L2CValue*/ {
    unsafe {
        let boma = app::sv_system::battle_object_module_accessor(fighter.lua_state_agent);
        let mut l2c_agent = L2CAgent::new(fighter.lua_state_agent);
        let lua_state = fighter.lua_state_agent;
        let battle_object_category = app::utility::get_category(boma);


        if battle_object_category == *BATTLE_OBJECT_CATEGORY_FIGHTER {

            handle_game_resets(boma, fighter);
            fighter_engine_edits(fighter, lua_state, &mut l2c_agent, boma)

        }
    }
}



// Global engine edits

// Note - if character-specific moveset changes are gonna happen... PLEASE use a jumptable instead of silly if/else chaining

unsafe fn fighter_engine_edits(fighter: &mut L2CFighterCommon, lua_state: u64, mut l2c_agent: &mut L2CAgent, boma: &mut app::BattleObjectModuleAccessor) {

    let status_kind = StatusModule::status_kind(boma);
    let situation_kind = StatusModule::situation_kind(boma);
    let curr_frame = MotionModule::frame(boma);
    let fighter_kind = app::utility::get_kind(boma);
    let cat1 = ControlModule::get_command_flag_cat(boma, 0);
    let cat2 = ControlModule::get_command_flag_cat(boma, 1);
    let stick_angle = ControlModule::get_stick_angle(boma);
    let stick_value_x = ControlModule::get_stick_x(boma);
    let entry_id = get_player_number(boma);

    crate::momentum_transfer::momentum_transfer_helper(fighter, lua_state, &mut l2c_agent, boma, status_kind, situation_kind, fighter_kind, curr_frame);
    crate::momentum_transfer::jumpsquat_correction(fighter, boma, status_kind, situation_kind, curr_frame);
    crate::momentum_transfer::additional_momentum_transfer_moves(fighter, lua_state, l2c_agent, boma, status_kind, situation_kind, fighter_kind, curr_frame);

    actions_out_of_js(boma, status_kind, situation_kind, cat1);
    shield_stops(boma, status_kind);
    shield_drops(boma, cat2, status_kind, fighter_kind);
    //single_button_smash_attacks(boma, status_kind, stick_angle, situation_kind);
    pivots(boma, status_kind, curr_frame, stick_value_x);
}




/* Notes on is_ready_go and the logic here

is_ready_go returns true when you (the player) have control over your character.
by creating two statics and comparing them we can determine when the game switches from a state
where you don't have control of the character (menus, loading, even training mode reset, anything that isn't technically "ingame")
we can determine the "start" (or end) of a match/game/gameplay session

In addition, is_ready go returns false for a few frames at the beginning of loading into training mode. It also returns false for the duration of the
Ready.... Go! sequence at the beginning of a match.

*/
unsafe fn handle_game_resets(boma: &mut app::BattleObjectModuleAccessor, fighter: &mut L2CFighterCommon) {
    //static vars don't get re-initialized if they've already been
    static mut LAST_READY_GO: bool = false;
    static mut IS_READY_GO_CURR: bool = true;

    IS_READY_GO_CURR = is_ready_go();

    //THIS BLOCK RUNS WHEN A "SESSION" ENDS
    if !IS_READY_GO_CURR && LAST_READY_GO
    {
        //println!("----------------GAME END--------------");
    }
    //THIS BLOCK RUNS WHEN A "SESSION" BEGINS
    else if IS_READY_GO_CURR && !LAST_READY_GO
    {
        //println!("----------------GAME START--------------");
        crate::vars::custom_var_resets::reset(boma);
        jump_speed_ratio[get_player_number(boma)] = (WorkModule::get_param_float(boma, hash40("jump_speed_x_max"), 0) / WorkModule::get_param_float(boma, hash40("run_speed_max"), 0));

    }
    LAST_READY_GO = IS_READY_GO_CURR;
}




// ------------------------------------------ ENGINE EDITS ---------------------------------------------------------------



unsafe fn actions_out_of_js(boma: &mut app::BattleObjectModuleAccessor, status_kind: i32, situation_kind: i32, cat1: i32) {
    let id = VarModule::get_int(boma, common::COSTUME_SLOT_NUMBER) as usize;

    if status_kind == *FIGHTER_STATUS_KIND_JUMP_SQUAT && situation_kind == *SITUATION_KIND_GROUND
    {

        // if you are in js, input grab, and you weren't previously shielding - transition to grab
        if compare_cat(cat1, *FIGHTER_PAD_CMD_CAT1_FLAG_CATCH)
           && ![*FIGHTER_STATUS_KIND_GUARD, *FIGHTER_STATUS_KIND_GUARD_ON, *FIGHTER_STATUS_KIND_GUARD_OFF].contains(&StatusModule::prev_status_kind(boma, 0))
           && !ItemModule::is_have_item(boma, 0)
        {
            WorkModule::set_flag(boma, true, *FIGHTER_STATUS_WORK_ID_FLAG_RESERVE_ATTACK_DISABLE_MINI_JUMP_ATTACK);
            StatusModule::change_status_request_from_script(boma, *FIGHTER_STATUS_KIND_CATCH, true);
        }

        // if you input airdodge and stick is below the netural y position - transition to airdodge
        /***else if compare_cat(cat1, *FIGHTER_PAD_CMD_CAT1_FLAG_AIR_ESCAPE)
                && ControlModule::get_stick_y(boma) <= 0.0
                && !compare_cat(cat1, *FIGHTER_PAD_CMD_CAT1_FLAG_CATCH)
        {
            StatusModule::change_status_request_from_script(boma, *FIGHTER_STATUS_KIND_ESCAPE_AIR, true);
        }***/



    }
    if [*FIGHTER_STATUS_KIND_ATTACK, *FIGHTER_STATUS_KIND_ATTACK_100, *FIGHTER_STATUS_KIND_ATTACK_DASH, *FIGHTER_STATUS_KIND_ATTACK_HI3, *FIGHTER_STATUS_KIND_ATTACK_LW3, *FIGHTER_STATUS_KIND_ATTACK_S3, *FIGHTER_STATUS_KIND_ATTACK_HI4, *FIGHTER_STATUS_KIND_ATTACK_LW4, *FIGHTER_STATUS_KIND_ATTACK_S4, *FIGHTER_STATUS_KIND_ATTACK_S4_HOLD, *FIGHTER_STATUS_KIND_ATTACK_HI4_HOLD, *FIGHTER_STATUS_KIND_ATTACK_LW4_HOLD, *FIGHTER_STATUS_KIND_ATTACK_S4_START, *FIGHTER_STATUS_KIND_ATTACK_HI4_START, *FIGHTER_STATUS_KIND_ATTACK_LW4_START].contains(&StatusModule::status_kind(boma)) {
        if MotionModule::frame(boma) < 3.0 {
            can_attack_cancel[id] = true;
        }
        else {
            can_attack_cancel[id] = false;
        }
    }
}

unsafe fn shield_stops(boma: &mut app::BattleObjectModuleAccessor, status_kind: i32) {
    if ( status_kind == *FIGHTER_STATUS_KIND_DASH || status_kind == *FIGHTER_STATUS_KIND_TURN_DASH ) &&
        ( ControlModule::check_button_trigger(boma, *CONTROL_PAD_BUTTON_GUARD) && ControlModule::check_button_off(boma, *CONTROL_PAD_BUTTON_CATCH) )
    {
        StatusModule::change_status_request_from_script(boma, *FIGHTER_STATUS_KIND_GUARD_ON, true);
        ControlModule::clear_command(boma, true);
    }
}

const SHIELD_DROP_FLICK_SENS: i32 = 11; // num frames since stick was displaced in the y axis. Lower num = more "intense" flick
const SHIELD_DROP_STICK_THRESHOLD: f32 = -0.3; // "highest" position stick can be (scale of -1 - 1 ... -1 = all the way down, 1 = all the way up)
unsafe fn shield_drops(boma: &mut app::BattleObjectModuleAccessor, cat2: i32, status_kind: i32, fighter_kind: i32) {
    let mut is_no_special_button_pass_char = false;
    if [*FIGHTER_KIND_INKLING, *FIGHTER_KIND_PICKEL].contains(&fighter_kind) { // characters that shouldn't be able to shield drop with shield + special button
        is_no_special_button_pass_char = true;
    }

unsafe fn jump_cancel_grab(boma: &mut app::BattleObjectModuleAccessor, cat1: i32, status_kind: i32, fighter_kind: i32) {
    if status_kind == *FIGHTER_STATUS_KIND_JUMP_SQUAT {
        if compare_cat(cat1, *FIGHTER_PAD_CMD_CAT1_FLAG_CATCH) {
            WorkModule::on_flag(boma, *FIGHTER_STATUS_WORK_ID_FLAG_RESERVE_ATTACK_DISABLE_MINI_JUMP_ATTACK);
            StatusModule::change_status_request_from_script(boma, *FIGHTER_STATUS_KIND_CATCH, true);
        }
    }
}

    if status_kind == *FIGHTER_STATUS_KIND_GUARD || status_kind == *FIGHTER_STATUS_KIND_GUARD_ON {

        let is_input_shield_drop =
        (compare_cat(ControlModule::get_pad_flag(boma), *FIGHTER_PAD_FLAG_SPECIAL_TRIGGER) && !is_no_special_button_pass_char)
        ||
        (
            compare_cat(cat2, *FIGHTER_PAD_CMD_CAT2_FLAG_GUARD_TO_PASS)
            || (ControlModule::get_flick_y(boma) <= SHIELD_DROP_FLICK_SENS && ControlModule::get_stick_y(boma) <= SHIELD_DROP_STICK_THRESHOLD)
        );

        if is_input_shield_drop && GroundModule::is_passable_ground(boma) {
            StatusModule::change_status_request_from_script(boma, *FIGHTER_STATUS_KIND_PASS, true);
        }
    }
}

const PREVENT_SMASH_ATTACK_STATUSES: [smash::lib::LuaConst;10] = [
    FIGHTER_STATUS_KIND_ATTACK_S4,
    FIGHTER_STATUS_KIND_ATTACK_HI4,
    FIGHTER_STATUS_KIND_ATTACK_LW4,
    FIGHTER_STATUS_KIND_ATTACK_S4_HOLD,
    FIGHTER_STATUS_KIND_ATTACK_HI4_HOLD,
    FIGHTER_STATUS_KIND_ATTACK_LW4_HOLD,
    FIGHTER_STATUS_KIND_ATTACK_S4_START,
    FIGHTER_STATUS_KIND_ATTACK_HI4_START,
    FIGHTER_STATUS_KIND_ATTACK_LW4_START,
    FIGHTER_STATUS_KIND_APPEAL
];
const SINGLE_BUTTON_SMASH_ATTACK_PAD_BUTTON: smash::lib::LuaConst = CONTROL_PAD_BUTTON_APPEAL_HI;
const SINGLE_BUTTON_SMASH_ATTACK_STICK_FLICK_THRESHOLD: i32 = 5; // max num of "flick frames" (# of frames since stick was displaced from neutral position) (higher = more lenient, lower = less lenient)
unsafe fn single_button_smash_attacks(boma: &mut app::BattleObjectModuleAccessor, status_kind: i32, stick_angle: f32, situation_kind: i32) {
    if !PREVENT_SMASH_ATTACK_STATUSES.iter().any(|x| *x == status_kind) && ControlModule::check_button_trigger(boma, *SINGLE_BUTTON_SMASH_ATTACK_PAD_BUTTON)
       && (ControlModule::get_flick_x(boma) <= SINGLE_BUTTON_SMASH_ATTACK_STICK_FLICK_THRESHOLD || ControlModule::get_flick_y(boma) <= SINGLE_BUTTON_SMASH_ATTACK_STICK_FLICK_THRESHOLD )
       && situation_kind == *SITUATION_KIND_GROUND && CancelModule::is_enable_cancel(boma)
    {
        match stick_angle {
            x if (-0.75..=0.75).contains(&x) => { //right
                StatusModule::change_status_request(boma, *FIGHTER_STATUS_KIND_ATTACK_S4_START, true);
            }
            x if (0.75..=2.25).contains(&x) => { //up
                StatusModule::change_status_request(boma, *FIGHTER_STATUS_KIND_ATTACK_HI4_START, true);
            }
            x if (2.25..=3.15).contains(&x) || (-3.15..=-2.25).contains(&x) => { //left
                StatusModule::change_status_request(boma, *FIGHTER_STATUS_KIND_ATTACK_S4_START, true);
            }
            x if (-2.25..=-0.75).contains(&x) => { //down
                StatusModule::change_status_request(boma, *FIGHTER_STATUS_KIND_ATTACK_LW4_START, true);
            }
            _ => (),
        }
    }
}


const PIVOT_STICK_SNAPBACK_WINDOW: f32 = 1.0;
const LIL_BOOSTIE: smash::phx::Vector3f = smash::phx::Vector3f {x: 1.6, y: 0.0, z: 0.0};
unsafe fn pivots(boma: &mut app::BattleObjectModuleAccessor, status_kind: i32, curr_frame: f32, stick_value_x: f32){
    if status_kind == *FIGHTER_STATUS_KIND_TURN_DASH
        && curr_frame <= PIVOT_STICK_SNAPBACK_WINDOW && stick_value_x == 0.0
        && [*FIGHTER_STATUS_KIND_TURN_DASH, *FIGHTER_STATUS_KIND_DASH].contains(&StatusModule::prev_status_kind(boma, 0))
        && ![*FIGHTER_STATUS_KIND_WAIT, *FIGHTER_STATUS_KIND_TURN].contains(&StatusModule::prev_status_kind(boma, 1))
    {
        PostureModule::reverse_lr(boma);
        StatusModule::change_status_request_from_script(boma, *FIGHTER_STATUS_KIND_TURN,true);
        KineticModule::clear_speed_all(boma);
        KineticModule::add_speed(boma, &LIL_BOOSTIE);
    }
}
