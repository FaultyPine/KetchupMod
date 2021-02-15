use smash::app::{self, lua_bind::*};
use smash::lib::lua_const::*;
use crate::utils::*;


#[skyline::hook(replace = ControlModule::get_command_flag_cat)]
pub unsafe fn get_command_flag_cat_hook(boma: &mut app::BattleObjectModuleAccessor, category: i32) -> i32 {
    let mut cat = original!()(boma, category);
    //Buffer fixes
    if category == 0 {
        let status_kind = StatusModule::status_kind(boma);
        let situation_kind = StatusModule::situation_kind(boma);

        if compare_cat(cat, *FIGHTER_PAD_CMD_CAT1_FLAG_ESCAPE) 
        && StatusModule::prev_status_kind(boma, 0) != *FIGHTER_STATUS_KIND_LANDING 
        {
            cat = clear_buffered_action(cat, *FIGHTER_PAD_CMD_CAT1_FLAG_ESCAPE);
        }
        if compare_cat(cat, *FIGHTER_PAD_CMD_CAT1_FLAG_ESCAPE_B) {
            cat = clear_buffered_action(cat, *FIGHTER_PAD_CMD_CAT1_FLAG_ESCAPE_B);
        }
        if compare_cat(cat, *FIGHTER_PAD_CMD_CAT1_FLAG_ESCAPE_F) {
            cat = clear_buffered_action(cat, *FIGHTER_PAD_CMD_CAT1_FLAG_ESCAPE_F);
        }
        if [*FIGHTER_PAD_CMD_CAT1_FLAG_AIR_ESCAPE, *FIGHTER_PAD_CMD_CAT1_FLAG_ESCAPE].iter().any(|x| compare_cat(cat, *x)) 
           && WorkModule::get_float(boma, *FIGHTER_INSTANCE_WORK_ID_FLOAT_DAMAGE_REACTION_FRAME) > 0.0 || status_kind == *FIGHTER_STATUS_KIND_ATTACK_AIR || situation_kind == *SITUATION_KIND_CLIFF
        {
            cat = 
                clear_buffered_action(clear_buffered_action(cat, *FIGHTER_PAD_CMD_CAT1_FLAG_AIR_ESCAPE), *FIGHTER_PAD_CMD_CAT1_FLAG_ESCAPE);
        }


        
    }

    cat
}