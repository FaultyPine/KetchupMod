use smash::{self, hash40, app, app::lua_bind::*, lib::*, lib::lua_const::*, lua2cpp::*, phx::*};
use smash_script::{self, *, macros::*};
use smashline::*;

use hdr_modules::consts::{*, globals::*};
use hdr_modules::*;

use crate::vars::custom_vars::*;

use crate::utils::*;
// use hdr_core::modules::VarModule;
// use hdr_modules::consts::*;
// use hdr_core::debugln;

// This file contains code for wavedashing out of jumpsquat, fullhop buffered aerials/attack canceling

pub fn install() {
    install_status_scripts!(
        //status_pre_JumpSquat,
        status_JumpSquat,
        status_end_JumpSquat,
        status_exec_JumpSquat
    );

    install_hooks!(
        //status_pre_JumpSquat_param,
        status_JumpSquat_Main,
        status_JumpSquat_common,
        uniq_process_JumpSquat_exec_status_param,
        sub_jump_squat_uniq_check_sub,
        sub_jump_squat_uniq_check_sub_mini_attack,
        sub_status_JumpSquat_check_stick_lr_update
    );
}
/***
// pre status stuff
#[common_status_script(status = FIGHTER_STATUS_KIND_JUMP_SQUAT, condition = LUA_SCRIPT_STATUS_FUNC_STATUS_PRE,
    symbol = "_ZN7lua2cpp16L2CFighterCommon20status_pre_JumpSquatEv")]
unsafe fn status_pre_JumpSquat(fighter: &mut L2CFighterCommon) -> L2CValue {
    status_pre_JumpSquat_param(
        fighter,
        FIGHTER_STATUS_WORK_KEEP_FLAG_NONE_FLAG.into(),
        FIGHTER_STATUS_WORK_KEEP_FLAG_NONE_INT.into(),
        FIGHTER_STATUS_WORK_KEEP_FLAG_NONE_FLOAT.into(),
        FIGHTER_KINETIC_TYPE_MOTION.into(),
        L2CValue::I32(0)
    )
}

#[hook(module = "common", symbol = "_ZN7lua2cpp16L2CFighterCommon26status_pre_JumpSquat_paramEN3lib8L2CValueES2_S2_S2_S2_")]
unsafe extern "C" fn status_pre_JumpSquat_param(fighter: &mut L2CFighterCommon, flag_keep: L2CValue, int_keep: L2CValue, float_keep: L2CValue, kinetic: L2CValue, arg: L2CValue) -> L2CValue {
    let flag_keep = flag_keep.into();
    let int_keep = int_keep.into();
    let float_keep = float_keep.into();
    let kinetic = kinetic.into();
    let arg = arg.into();
    StatusModule::init_settings(
        fighter.module_accessor,
        app::SituationKind(*SITUATION_KIND_GROUND),
        kinetic,
        *GROUND_CORRECT_KIND_GROUND_CLIFF_STOP as u32,
        app::GroundCliffCheckKind(*GROUND_CLIFF_CHECK_KIND_NONE),
        true,
        flag_keep,
        int_keep,
        float_keep,
        arg
    );
    FighterStatusModuleImpl::set_fighter_status_data(
        fighter.module_accessor,
        true,
        *FIGHTER_TREADED_KIND_ENABLE,
        false,
        false,
        false,
        0,
        *FIGHTER_STATUS_ATTR_INTO_DOOR as u32,
        0,
        0
    );

    0.into()
} ***/

#[common_status_script(status = FIGHTER_STATUS_KIND_JUMP_SQUAT, condition = LUA_SCRIPT_STATUS_FUNC_STATUS_MAIN,
    symbol = "_ZN7lua2cpp16L2CFighterCommon16status_JumpSquatEv")]
unsafe fn status_JumpSquat(fighter: &mut L2CFighterCommon) -> L2CValue {
    let lr_update = fighter.sub_status_JumpSquat_check_stick_lr_update();
    fighter.status_JumpSquat_common(lr_update);
    fighter.sub_shift_status_main(L2CValue::Ptr(status_JumpSquat_Main as *const () as _))
}

#[hook(module = "common", symbol = "_ZN7lua2cpp16L2CFighterCommon21status_JumpSquat_MainEv")]
unsafe fn status_JumpSquat_Main(fighter: &mut L2CFighterCommon) -> L2CValue {
    // Check if a character (like greninja) has a custom subroutine for status checks
    let should_end = if fighter.global_table[CUSTOM_ROUTINE].get_bool() {
        let custom_routine: *const extern "C" fn(&mut L2CFighterCommon) -> L2CValue = fighter.global_table[CUSTOM_ROUTINE].get_ptr() as _;
        if !custom_routine.is_null() {
            let callable: extern "C" fn(&mut L2CFighterCommon) -> L2CValue = std::mem::transmute(custom_routine);
            callable(fighter).get_bool()
        } else {
            false
        }
    } else { false };
    if should_end {
        return L2CValue::I32(1);
    }

    // begin testing for transitions out of jump squat
    let situation_kind = fighter.global_table[SITUATION_KIND].get_i32();
    if WorkModule::is_enable_transition_term(fighter.module_accessor, *FIGHTER_STATUS_TRANSITION_TERM_ID_CONT_ESCAPE_AIR) {
        println!("can airdodge js");
        fighter.change_status(
            L2CValue::I32(*FIGHTER_STATUS_KIND_ESCAPE_AIR), // We don't want to change to ESCAPE_AIR_SLIDE in case they do a nair dodge
            L2CValue::Bool(true)
        );
    } else if WorkModule::is_enable_transition_term(fighter.module_accessor, *FIGHTER_STATUS_TRANSITION_TERM_ID_JUMP_START) {
        fighter.change_status(
            L2CValue::I32(*FIGHTER_STATUS_KIND_JUMP),
            L2CValue::Bool(false)
        );
    } else if WorkModule::is_enable_transition_term(fighter.module_accessor, *FIGHTER_STATUS_TRANSITION_TERM_ID_FALL)
            && situation_kind == *SITUATION_KIND_AIR {
        fighter.change_status(
            L2CValue::I32(*FIGHTER_STATUS_KIND_FALL),
            L2CValue::Bool(false)
        );
    } else if !fighter.sub_transition_group_check_ground_item().get_bool() {
        let cat1 = fighter.global_table[CMD_CAT1].get_i32();
        if WorkModule::is_enable_transition_term(fighter.module_accessor, *FIGHTER_STATUS_TRANSITION_TERM_ID_CONT_SPECIAL_HI)
            && cat1 & *FIGHTER_PAD_CMD_CAT1_FLAG_SPECIAL_HI != 0
            && situation_kind == *SITUATION_KIND_GROUND {
            fighter.change_status(
                L2CValue::I32(*FIGHTER_STATUS_KIND_SPECIAL_HI),
                L2CValue::Bool(true)
            );
        } else if !fighter.sub_transition_specialflag_hoist().get_bool() {
            let cat2 = fighter.global_table[CMD_CAT2].get_i32();
            if WorkModule::is_enable_transition_term(fighter.module_accessor, *FIGHTER_STATUS_TRANSITION_TERM_ID_CONT_ATTACK_HI4_START)
                && !ControlModule::check_button_on(fighter.module_accessor, *CONTROL_PAD_BUTTON_CSTICK_ON)
                && cat2 & *FIGHTER_PAD_CMD_CAT2_FLAG_ATTACK_DASH_ATTACK_HI4 != 0
                && situation_kind == *SITUATION_KIND_GROUND {
                fighter.change_status(
                    L2CValue::I32(*FIGHTER_STATUS_KIND_ATTACK_HI4_START),
                    L2CValue::Bool(true)
                );
            }
        }
    }
    0.into()
}

// end status stuff
// no symbol since you can't call `fighter.status_end_JumpSquat()`, and replacing `bind_call_...` makes no sense here
#[common_status_script(status = FIGHTER_STATUS_KIND_JUMP_SQUAT, condition = LUA_SCRIPT_STATUS_FUNC_STATUS_END)]
unsafe fn status_end_JumpSquat(fighter: &mut L2CFighterCommon) -> L2CValue {
    WorkModule::off_flag(fighter.module_accessor, *FIGHTER_INSTANCE_WORK_ID_FLAG_JUMP_MINI_ATTACK);
    0.into()
}

// exec status stuff
#[common_status_script(status = FIGHTER_STATUS_KIND_JUMP_SQUAT, condition = LUA_SCRIPT_STATUS_FUNC_EXEC_STATUS)]
unsafe fn status_exec_JumpSquat(fighter: &mut L2CFighterCommon) -> L2CValue {
    uniq_process_JumpSquat_exec_status_param(fighter, L2CValue::Ptr(0 as _));
    0.into()
}

// common jumpsquat subroutine -- to be called by each fighter before transitioning to a custom main status
#[hook(module = "common", symbol = "_ZN7lua2cpp16L2CFighterCommon23status_JumpSquat_commonEN3lib8L2CValueE")]
unsafe fn status_JumpSquat_common(fighter: &mut L2CFighterCommon, lr_update: L2CValue) {
    let id = VarModule::get_int(fighter.module_accessor, common::COSTUME_SLOT_NUMBER) as usize;
    let is_button_jump = WorkModule::get_int(fighter.module_accessor, *FIGHTER_INSTANCE_WORK_ID_INT_STICK_JUMP_COMMAND_LIFE) == 0
                                || fighter.global_table[FLICK_Y_DIR].get_i32() <= 0;
    if is_button_jump {
        WorkModule::on_flag(fighter.module_accessor, *FIGHTER_STATUS_JUMP_FLAG_BUTTON);
        // check if we are doing double button shorthop input
        if ControlModule::is_jump_mini_button(fighter.module_accessor) || ([*FIGHTER_STATUS_KIND_ATTACK, *FIGHTER_STATUS_KIND_ATTACK_100, *FIGHTER_STATUS_KIND_ATTACK_DASH, *FIGHTER_STATUS_KIND_ATTACK_HI3, *FIGHTER_STATUS_KIND_ATTACK_LW3, *FIGHTER_STATUS_KIND_ATTACK_S3, *FIGHTER_STATUS_KIND_ATTACK_HI4, *FIGHTER_STATUS_KIND_ATTACK_LW4, *FIGHTER_STATUS_KIND_ATTACK_S4, *FIGHTER_STATUS_KIND_ATTACK_S4_HOLD, *FIGHTER_STATUS_KIND_ATTACK_HI4_HOLD, *FIGHTER_STATUS_KIND_ATTACK_LW4_HOLD, *FIGHTER_STATUS_KIND_ATTACK_S4_START, *FIGHTER_STATUS_KIND_ATTACK_HI4_START, *FIGHTER_STATUS_KIND_ATTACK_LW4_START].contains(&StatusModule::prev_status_kind(fighter.module_accessor, 0)) && can_attack_cancel[id]) {
            WorkModule::on_flag(fighter.module_accessor, *FIGHTER_STATUS_WORK_ID_FLAG_RESERVE_JUMP_MINI);
        }
    }
    // I think this int might be referring to how many frames we check for tap jump?
    WorkModule::set_int(fighter.module_accessor, 0, *FIGHTER_INSTANCE_WORK_ID_INT_STICK_JUMP_COMMAND_LIFE);
    // `lr_update` comes from a dif subroutine
    if lr_update.get_bool() {
        PostureModule::set_stick_lr(fighter.module_accessor, 0.0);
        PostureModule::update_rot_y_lr(fighter.module_accessor);
    }
    ControlModule::reset_flick_y(fighter.module_accessor);
    ControlModule::reset_flick_sub_y(fighter.module_accessor);
    fighter.global_table[FLICK_Y] = 0xFE.into();

    // not a conditional enable, so it's not in potential_enables
    WorkModule::enable_transition_term(fighter.module_accessor, *FIGHTER_STATUS_TRANSITION_TERM_ID_FALL);
    let potential_enables = [
        *FIGHTER_STATUS_TRANSITION_TERM_ID_CONT_SPECIAL_HI,
        *FIGHTER_STATUS_TRANSITION_TERM_ID_CONT_ATTACK_HI4_START,
        *FIGHTER_STATUS_TRANSITION_TERM_ID_CONT_ITEM_THROW_FORCE,
        *FIGHTER_STATUS_TRANSITION_TERM_ID_CONT_ITEM_THROW
    ];
    for x in potential_enables.iter() {
        WorkModule::enable_transition_term(fighter.module_accessor, *x);
    }
    WorkModule::unable_transition_term(fighter.module_accessor, *FIGHTER_STATUS_TRANSITION_TERM_ID_JUMP_START);
    if WorkModule::is_flag(fighter.module_accessor, *FIGHTER_INSTANCE_WORK_ID_FLAG_ABNORMAL_MINIJUMP_SLOWWALK) {
        WorkModule::on_flag(fighter.module_accessor, *FIGHTER_STATUS_WORK_ID_FLAG_RESERVE_JUMP_MINI);
    }
    // if we are doing a buffered aerial we want to disable all of the other transitions
    if WorkModule::is_flag(fighter.module_accessor, *FIGHTER_INSTANCE_WORK_ID_FLAG_JUMP_MINI_ATTACK) {
        WorkModule::on_flag(fighter.module_accessor, *FIGHTER_STATUS_JUMP_FLAG_RESERVE_ATTACK_BUTTON_ON);
        for x in potential_enables.iter() {
            WorkModule::unable_transition_term(fighter.module_accessor, *x);
        }
        MotionAnimcmdModule::enable_skip_delay_update(fighter.module_accessor);
    }
    // same as above, but without the skip stuff
    if WorkModule::is_flag(fighter.module_accessor, *FIGHTER_INSTANCE_WORK_ID_FLAG_RESERVE_JUMP_MINI_ATTACK) {
        for x in potential_enables.iter() {
            WorkModule::unable_transition_term(fighter.module_accessor, *x);
        }
    }

    VarModule::off_flag(fighter.module_accessor, common::ENABLE_AIR_ESCAPE_JUMPSQUAT);
    sub_air_reset_escape_air_snap(fighter);
}

// The main exec block, for some reason it's not found in the exec status
#[hook(module = "common", symbol = "_ZN7lua2cpp16L2CFighterCommon40uniq_process_JumpSquat_exec_status_paramEN3lib8L2CValueE")]
unsafe fn uniq_process_JumpSquat_exec_status_param(fighter: &mut L2CFighterCommon, arg: L2CValue) {
    let should_check = if fighter.global_table[CUSTOM_ROUTINE].get_bool() {
        let custom_routine: *const extern "C" fn(&mut L2CFighterCommon) -> L2CValue = fighter.global_table[CUSTOM_ROUTINE].get_ptr() as _;
        if !custom_routine.is_null() {
            let callable: extern "C" fn(&mut L2CFighterCommon) -> L2CValue = std::mem::transmute(custom_routine);
            callable(fighter);
            true
        } else {
            true
        }
    } else { true };
    if should_check {
        fighter.sub_jump_squat_uniq_check_sub(L2CValue::I32(*FIGHTER_STATUS_JUMP_FLAG_BUTTON));
        fighter.sub_jump_squat_uniq_check_sub_mini_attack();
    }

    let motion_kind = MotionModule::motion_kind(fighter.module_accessor);
    let frame = MotionModule::frame(fighter.module_accessor);
    let update_rate = MotionModule::update_rate(fighter.module_accessor);
    let cat1 = fighter.global_table[CMD_CAT1].get_i32();
    if cat1 & *FIGHTER_PAD_CMD_CAT1_FLAG_AIR_ESCAPE != 0 {
        VarModule::on_flag(fighter.module_accessor, common::ENABLE_AIR_ESCAPE_JUMPSQUAT);
    }
    if cat1 & *FIGHTER_PAD_CMD_CAT1_FLAG_CATCH != 0
        && !WorkModule::is_enable_transition_term(fighter.module_accessor, *FIGHTER_STATUS_TRANSITION_TERM_ID_CONT_CATCH) {
        WorkModule::enable_transition_term(fighter.module_accessor, *FIGHTER_STATUS_TRANSITION_TERM_ID_CONT_CATCH);
    }
    else {
        let end_frame = MotionModule::end_frame_from_hash(fighter.module_accessor, Hash40::new_raw(motion_kind)) as u32 as f32;
        if end_frame <= (frame + update_rate) {
            StatusModule::set_situation_kind(fighter.module_accessor, app::SituationKind(*SITUATION_KIND_AIR), false);
            fighter.global_table[PREV_SITUATION_KIND] = fighter.global_table[SITUATION_KIND].clone();
            if VarModule::is_flag(fighter.module_accessor, common::ENABLE_AIR_ESCAPE_JUMPSQUAT) {
                // check if we are doing directional airdodge
                let stick = app::sv_math::vec2_length(fighter.global_table[STICK_X].get_f32(), fighter.global_table[STICK_Y].get_f32());
                if stick >= WorkModule::get_param_float(fighter.module_accessor, hash40("common"), hash40("escape_air_slide_stick")) {
                    VarModule::on_flag(fighter.module_accessor, common::PERFECT_WAVEDASH);
                    // change kinetic/ground properties for wavedash
                    GroundModule::correct(fighter.module_accessor, app::GroundCorrectKind(*GROUND_CORRECT_KIND_NONE));
                    KineticModule::change_kinetic(fighter.module_accessor, *FIGHTER_KINETIC_TYPE_AIR_ESCAPE);
                } else {
                    VarModule::off_flag(fighter.module_accessor, common::PERFECT_WAVEDASH);
                    // change kinetic properties for rising nairdodge
                    GroundModule::correct(fighter.module_accessor, app::GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
                    KineticModule::change_kinetic(fighter.module_accessor, *FIGHTER_KINETIC_TYPE_JUMP);
                }
                WorkModule::enable_transition_term(fighter.module_accessor, *FIGHTER_STATUS_TRANSITION_TERM_ID_CONT_ESCAPE_AIR);
            } else {
                // change kinetic/ground properties for jump
                VarModule::off_flag(fighter.module_accessor, common::PERFECT_WAVEDASH);
                GroundModule::correct(fighter.module_accessor, app::GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
                WorkModule::set_int(fighter.module_accessor, *FIGHTER_STATUS_JUMP_FROM_SQUAT, *FIGHTER_STATUS_WORK_ID_INT_RESERVE_JUMP_FROM);
                KineticModule::change_kinetic(fighter.module_accessor, *FIGHTER_KINETIC_TYPE_JUMP);
                WorkModule::enable_transition_term(fighter.module_accessor, *FIGHTER_STATUS_TRANSITION_TERM_ID_JUMP_START);
            }
        }
        else {
            //println!("js_vel: {}", KineticModule::get_sum_speed_x(fighter.module_accessor, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN));
            VarModule::set_float(fighter.module_accessor, common::JUMPSQUAT_VELOCITY, KineticModule::get_sum_speed_x(fighter.module_accessor, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN));
            VarModule::set_float(fighter.module_accessor, common::CURRENT_MOMENTUM, KineticModule::get_sum_speed_x(fighter.module_accessor, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN));
            VarModule::set_float(fighter.module_accessor, common::CURRENT_MOMENTUM_SPECIALS, KineticModule::get_sum_speed_x(fighter.module_accessor, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN));
        }
    }

}

// subroutine for checking for aerial macro
#[hook(module = "common", symbol = "_ZN7lua2cpp16L2CFighterCommon29sub_jump_squat_uniq_check_subEN3lib8L2CValueE")]
unsafe fn sub_jump_squat_uniq_check_sub(fighter: &mut L2CFighterCommon, flag: L2CValue) {
    original!()(fighter, flag)
}

#[hook(module = "common", symbol = "_ZN7lua2cpp16L2CFighterCommon41sub_jump_squat_uniq_check_sub_mini_attackEv")]
unsafe fn sub_jump_squat_uniq_check_sub_mini_attack(fighter: &mut L2CFighterCommon) {
    original!()(fighter)
}

#[hook(module = "common", symbol = "_ZN7lua2cpp16L2CFighterCommon42sub_status_JumpSquat_check_stick_lr_updateEv")]
unsafe fn sub_status_JumpSquat_check_stick_lr_update(fighter: &mut L2CFighterCommon) -> L2CValue {
    let prev_status = fighter.global_table[PREV_STATUS_KIND].get_i32();
    L2CValue::Bool(prev_status == *FIGHTER_STATUS_KIND_DASH || prev_status == *FIGHTER_STATUS_KIND_TURN_DASH)
}

pub unsafe fn sub_air_reset_escape_air_snap(fighter: &mut L2CFighterCommon) {
    let magnet_frame = ParamModule::get_int(fighter.module_accessor, ParamType::Common, "air_escape_snap_frame");
    VarModule::set_int(fighter.module_accessor, common::AIR_ESCAPE_MAGNET_FRAME, magnet_frame);
    // println!("magnet_frame {}", magnet_frame);
    VarModule::off_flag(fighter.module_accessor, common::ENABLE_AIR_ESCAPE_MAGNET);
}
