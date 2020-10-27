use smash::app::{self, lua_bind::*};
use smash::hash40;

//9.0.0/9.0.1 (same offset apparently) -> 0x4dae10
#[skyline::hook(offset=0x4dae10)]
pub unsafe fn get_param_float_middle(x0: u64, param_type: u64, param_hash: u64) -> f32 {
    let boma = &mut *(*((x0 as *mut u64).offset(1)) as *mut app::BattleObjectModuleAccessor);
    let fighter_kind = app::utility::get_kind(boma);
    let status_kind = StatusModule::status_kind(boma);


    if param_hash == 0 { //fighter_param



    }

    else if param_type == hash40("param_motion") { // fighter_param_motion


        if param_hash == hash40("landing_frame_escape_air_slide_max") { //directional airdodge landing lag
            if MotionModule::frame(boma) > 8.0 { // reduces landing lag on wavedashes, but keeps LL high for airdodges from higher up
                return 20.0; // "distanced" airdodge
            }
            return 10.0; // ""perfect"" airdodge (wavedash)
        }


    }

    else if param_type == hash40("common") {

        

    }



    original!()(x0, param_type, param_hash)
}