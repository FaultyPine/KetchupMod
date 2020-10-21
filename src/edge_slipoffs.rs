use smash::app::{self, lua_bind::*};
use smash::lib::lua_const::*;

#[skyline::hook(replace = GroundModule::correct)]
pub unsafe fn correct_hook(boma: &mut app::BattleObjectModuleAccessor, param_2: u64) -> u64{

    if app::utility::get_category(boma) == *BATTLE_OBJECT_CATEGORY_FIGHTER {
        let status_kind = StatusModule::status_kind(boma);
        if [/*Air Dodge*/ 0x22, /*Landing*/0x16, /*Run brake*/0x5, /*Turn Dash*/0x7, /*Dash*/0x3].contains(&status_kind) {
            return original!()(boma, *GROUND_CORRECT_KIND_GROUND as u64);
        }
    }
    original!()(boma, param_2)
}