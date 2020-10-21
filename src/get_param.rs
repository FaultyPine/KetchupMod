use smash::app::{self, lua_bind::*};
use smash::hash40;
use smash::lib::lua_const::*;
use crate::utils::*;

//9.0.0 -> 0x4dae10
#[skyline::hook(offset=0x4dae10)] //9.0.0
pub unsafe fn get_param_float_middle(x0: u64, param_type: u64, param_hash: u64) -> f32 {
    let mut boma = &mut *(*((x0 as *mut u64).offset(1)) as *mut app::BattleObjectModuleAccessor);
    let fighter_kind = app::utility::get_kind(boma);
    let status_kind = StatusModule::status_kind(boma);


    if param_hash == 0 { //fighter_param

        if param_type == hash40("escape_air_landing_frame") {

        }


    }

    if param_type == hash40("param_motion") { // fighter_param_motion
        if param_hash == hash40("landing_frame_escape_air_slide") {
            if MotionModule::frame(boma) > 3.0 { // reduces landing lag on wavedashes, but keeps LL high for airdodges from higher up
                return 20;
            }
            return 10;
        }
    }



    original!()(x0, param_type, param_hash)
}