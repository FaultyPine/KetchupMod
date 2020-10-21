use smash::app::{self, lua_bind::*};
use smash::lib::lua_const::*;
use crate::utils::*;


#[skyline::hook(replace = ControlModule::get_command_flag_cat)]
pub unsafe fn get_command_flag_cat_hook(boma: &mut app::BattleObjectModuleAccessor, category: i32) -> i32 {
    let mut cat = original!()(boma, category);
    let status_kind = StatusModule::status_kind(boma);
    let situation_kind = StatusModule::situation_kind(boma);
    //Buffer fixes
    if category == 0 {
        
        if compare_cat(cat, *FIGHTER_PAD_CMD_CAT1_FLAG_ESCAPE) {
            cat = clear_buffered_action(cat, *FIGHTER_PAD_CMD_CAT1_FLAG_ESCAPE);
        }
        if compare_cat(cat, *FIGHTER_PAD_CMD_CAT1_FLAG_ESCAPE_B) {
            cat = clear_buffered_action(cat, *FIGHTER_PAD_CMD_CAT1_FLAG_ESCAPE_B);
        }
        if compare_cat(cat, *FIGHTER_PAD_CMD_CAT1_FLAG_ESCAPE_F) {
            cat = clear_buffered_action(cat, *FIGHTER_PAD_CMD_CAT1_FLAG_ESCAPE_F);
        }
        if compare_cat(cat, *FIGHTER_PAD_CMD_CAT1_FLAG_AIR_ESCAPE) &&
            (WorkModule::get_float(boma, *FIGHTER_INSTANCE_WORK_ID_FLOAT_DAMAGE_REACTION_FRAME) > 0.0 || status_kind == *FIGHTER_STATUS_KIND_ATTACK_AIR || situation_kind == *SITUATION_KIND_CLIFF ) {
                cat = clear_buffered_action(cat, *FIGHTER_PAD_CMD_CAT1_FLAG_AIR_ESCAPE);
        }


        
    }

    cat
}