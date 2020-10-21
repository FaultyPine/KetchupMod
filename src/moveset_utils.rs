use smash::lib::lua_const::*;
use smash::app::{self, lua_bind::*};
use crate::utils::*;



/*
----------------------------------------------------------------------------------------------------
                                            ENABLE CANCELS
----------------------------------------------------------------------------------------------------
*/

pub unsafe fn enable_jump_cancel(boma: &mut app::BattleObjectModuleAccessor, situation_kind: i32, cat1: i32, begin_frame_window: i32, end_frame_window: i32) {
    if jump_checker_buffer(boma, cat1) && MotionModule::frame(boma) >= begin_frame_window as f32 && MotionModule::frame(boma) <= end_frame_window as f32 {
        if situation_kind == *SITUATION_KIND_AIR {
            if WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_JUMP_COUNT) < WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_JUMP_COUNT_MAX) {
                StatusModule::change_status_request_from_script(boma, *FIGHTER_STATUS_KIND_JUMP_AERIAL, true);
            }
        }
        else if situation_kind == *SITUATION_KIND_GROUND {
            StatusModule::change_status_request_from_script(boma, *FIGHTER_STATUS_KIND_JUMP_SQUAT, true);
        }
    }
}

pub unsafe fn enable_jump_cancel_dir_reversable(boma: &mut app::BattleObjectModuleAccessor, situation_kind: i32, cat1: i32, begin_frame_window: i32, end_frame_window: i32, reversable_ground: bool, reversable_air: bool) {
    if jump_checker_buffer(boma, cat1) && MotionModule::frame(boma) >= begin_frame_window as f32 && MotionModule::frame(boma) <= end_frame_window as f32 {
        if situation_kind == *SITUATION_KIND_AIR {
            if WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_JUMP_COUNT) < WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_JUMP_COUNT_MAX) {
                if reversable_air && PostureModule::lr(boma) * ControlModule::get_stick_x(boma) < 0.0 {
                    PostureModule::reverse_lr(boma);
                    PostureModule::update_rot_y_lr(boma);
                }
                StatusModule::change_status_request_from_script(boma, *FIGHTER_STATUS_KIND_JUMP_AERIAL, true);
            }
        }
        else if situation_kind == *SITUATION_KIND_GROUND {
            if reversable_ground && PostureModule::lr(boma) * ControlModule::get_stick_x(boma) < 0.0 {
                PostureModule::reverse_lr(boma);
                PostureModule::update_rot_y_lr(boma);
            }
            StatusModule::change_status_request_from_script(boma, *FIGHTER_STATUS_KIND_JUMP_SQUAT, true);
        }
    }
}

pub unsafe fn enable_fast_fall(boma: &mut app::BattleObjectModuleAccessor, situation_kind: i32, cat2: i32) {
    if situation_kind == *SITUATION_KIND_AIR {
        if ControlModule::get_stick_y(boma) < 0. && compare_cat(cat2, *FIGHTER_PAD_CMD_CAT2_FLAG_FALL_JUMP) && KineticModule::get_sum_speed_y(boma, *FIGHTER_KINETIC_ENERGY_ID_GRAVITY) < 0. {
            WorkModule::set_flag(boma, true, *FIGHTER_STATUS_WORK_ID_FLAG_RESERVE_DIVE);
        }
    }
}

pub unsafe fn enable_land_cancel(boma: &mut app::BattleObjectModuleAccessor, situation_kind: i32, land_status: i32) {
    if situation_kind == *SITUATION_KIND_GROUND && StatusModule::prev_situation_kind(boma) == *SITUATION_KIND_AIR {
        StatusModule::change_status_request_from_script(boma, land_status, true);
    }
}

pub unsafe fn enable_dash_cancel(boma: &mut app::BattleObjectModuleAccessor, cat1: i32, situation_kind: i32, begin_frame_window: i32, end_frame_window: i32) {
    if MotionModule::frame(boma) as i32 >= begin_frame_window && MotionModule::frame(boma) as i32 <= end_frame_window && situation_kind == SITUATION_KIND_GROUND {
        if compare_cat(cat1,*FIGHTER_PAD_CMD_CAT1_FLAG_DASH) {
            StatusModule::change_status_request_from_script(boma, *FIGHTER_STATUS_KIND_DASH,true);
        }
        else if compare_cat(cat1, *FIGHTER_PAD_CMD_CAT1_FLAG_TURN_DASH) {
            StatusModule::change_status_request_from_script(boma, *FIGHTER_STATUS_KIND_TURN_DASH,true);
        }
    }
}




pub unsafe fn jump_checker_buffer(boma: &mut app::BattleObjectModuleAccessor, cat1: i32) -> bool{
    ( compare_cat(cat1, *FIGHTER_PAD_CMD_CAT1_FLAG_JUMP) && ControlModule::is_enable_flick_jump(boma) ) 
    || 
    compare_cat(cat1, *FIGHTER_PAD_CMD_CAT1_FLAG_JUMP_BUTTON)
}