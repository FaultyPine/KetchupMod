use smash::app::{self, lua_bind::*};
use smash::lib::lua_const::*;

#[skyline::hook(replace = StatusModule::init_settings)]
pub unsafe fn init_settings_hook(boma: &mut app::BattleObjectModuleAccessor, situation_kind: i32, param_3: i32, param_4: i32, param_5: u64, param_6: bool, param_7: i32, param_8: i32, param_9: i32, param_10: i32){

    if app::utility::get_category(boma) == *BATTLE_OBJECT_CATEGORY_FIGHTER {
        JostleModule::set_team(boma, 0);
        if app::utility::get_kind(boma) == *FIGHTER_KIND_DOLLY && [*FIGHTER_DOLLY_STATUS_KIND_SUPER_SPECIAL].contains(&StatusModule::status_kind(boma)) {
            JostleModule::set_team(boma, 1);
        }

    }
    original!()(boma, situation_kind, param_3, param_4, param_5, param_6, param_7, param_8, param_9, param_10)
}