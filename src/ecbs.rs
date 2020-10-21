use smash::app::{self, lua_bind::*};
use smash::phx::*;
use smash::lib::lua_const::*;
use crate::utils::*;

extern "C" {
    #[link_name = "\u{1}_ZN3app11FighterUtil33get_ground_correct_kind_air_transERNS_26BattleObjectModuleAccessorEi"]
    pub fn get_ground_correct_kind_air_trans(boma: &mut app::BattleObjectModuleAccessor, smthn: i32) -> u64;
}
 
// ECB shift Aerial State Fixes
#[skyline::hook(replace = get_ground_correct_kind_air_trans)]
pub unsafe fn get_ground_correct_kind_air_trans_hook(boma: &mut app::BattleObjectModuleAccessor, something: i32) -> u64 {
    *GROUND_CORRECT_KIND_AIR as u64
}

#[skyline::hook(replace = GroundModule::get_rhombus)]
pub unsafe fn get_rhombus_hook(boma: &mut app::BattleObjectModuleAccessor, unk: bool) -> u64 {
    original!()(boma, true)
}



//called in sys_line
use crate::vars::custom_vars::ECY_Y_OFFSETS;
pub unsafe fn fixed_ecbs(boma: &mut app::BattleObjectModuleAccessor, status_kind: i32, situtation_kind: i32, fighter_kind: i32, curr_frame: f32) {
    let mut max_offset: f32 = 0.0;
    let mut offset = Vector2f {x:0.0, y:0.0};

    //  Set normal ECB on training mode restart - prevents weird clipping
    if !is_ready_go() {
        GroundModule::set_rhombus_offset(boma, &offset);
        return;
    }

    /*if status_kind == *FIGHTER_STATUS_KIND_ENTRY {
        max_offset = 0.0;
    }*/
    if [*FIGHTER_KIND_KIRBY, *FIGHTER_KIND_PIKACHU, *FIGHTER_KIND_NESS, *FIGHTER_KIND_PURIN, *FIGHTER_KIND_GAMEWATCH, *FIGHTER_KIND_POPO,
        *FIGHTER_KIND_NANA, *FIGHTER_KIND_PICHU, *FIGHTER_KIND_METAKNIGHT, *FIGHTER_KIND_WARIO, *FIGHTER_KIND_PZENIGAME, *FIGHTER_KIND_PFUSHIGISOU, *FIGHTER_KIND_LUCAS, 
        *FIGHTER_KIND_PIKMIN, *FIGHTER_KIND_TOONLINK, *FIGHTER_KIND_DUCKHUNT, *FIGHTER_KIND_MURABITO, *FIGHTER_KIND_INKLING, *FIGHTER_KIND_SHIZUE
    ].contains(&fighter_kind) {
        max_offset = 2.0;
    }

    else if [*FIGHTER_KIND_MARIO, *FIGHTER_KIND_YOSHI, *FIGHTER_KIND_LUIGI, *FIGHTER_KIND_MARIOD, *FIGHTER_KIND_YOUNGLINK, *FIGHTER_KIND_PLIZARDON,
        *FIGHTER_KIND_DIDDY, *FIGHTER_KIND_DEDEDE, *FIGHTER_KIND_ROCKMAN, *FIGHTER_KIND_GEKKOUGA, *FIGHTER_KIND_PACMAN, *FIGHTER_KIND_KOOPAJR, *FIGHTER_KIND_PACKUN,
        *FIGHTER_KIND_MIIFIGHTER, *FIGHTER_KIND_MIISWORDSMAN, *FIGHTER_KIND_MIIGUNNER, *FIGHTER_KIND_PACKUN, *FIGHTER_KIND_BUDDY
    ].contains(&fighter_kind) {
        max_offset = 3.5;
    }

    else if [*FIGHTER_KIND_FOX, *FIGHTER_KIND_FALCO, *FIGHTER_KIND_DAISY, *FIGHTER_KIND_MEWTWO, *FIGHTER_KIND_PIT, *FIGHTER_KIND_PITB, *FIGHTER_KIND_SONIC,
        *FIGHTER_KIND_LUCARIO, *FIGHTER_KIND_ROBOT, *FIGHTER_KIND_WOLF, *FIGHTER_KIND_LITTLEMAC, *FIGHTER_KIND_KROOL, *FIGHTER_KIND_GAOGAEN
    ].contains(&fighter_kind) {
        max_offset = 4.0;
    }

    else if [*FIGHTER_KIND_DONKEY, *FIGHTER_KIND_LINK, *FIGHTER_KIND_SAMUS, *FIGHTER_KIND_SAMUSD, *FIGHTER_KIND_CAPTAIN, *FIGHTER_KIND_PEACH, *FIGHTER_KIND_KOOPA, 
        *FIGHTER_KIND_SHEIK, *FIGHTER_KIND_ZELDA, *FIGHTER_KIND_MARTH, *FIGHTER_KIND_LUCINA, *FIGHTER_KIND_GANON, *FIGHTER_KIND_ROY, *FIGHTER_KIND_CHROM,
        *FIGHTER_KIND_SZEROSUIT, *FIGHTER_KIND_SNAKE, *FIGHTER_KIND_IKE, *FIGHTER_KIND_WIIFIT, *FIGHTER_KIND_ROSETTA, *FIGHTER_KIND_PALUTENA, *FIGHTER_KIND_REFLET,
        *FIGHTER_KIND_SHULK, *FIGHTER_KIND_RYU, *FIGHTER_KIND_KEN, *FIGHTER_KIND_CLOUD, *FIGHTER_KIND_KAMUI, *FIGHTER_KIND_BAYONETTA, *FIGHTER_KIND_RIDLEY,
        *FIGHTER_KIND_SIMON, *FIGHTER_KIND_RICHTER, *FIGHTER_KIND_JACK, *FIGHTER_KIND_BRAVE, *FIGHTER_KIND_DOLLY, *FIGHTER_KIND_MASTER
    ].contains(&fighter_kind) {
        max_offset = 5.0;
    }

    // - boolean to hold if the game should use vanilla ecbs instead of shifting them up; fixes some ECB issues
    let prev_status_kind = StatusModule::prev_status_kind(boma,0);
    let air_trans = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_FRAME_IN_AIR) < 10;
    let vanilla_ecb = [*FIGHTER_STATUS_KIND_CAPTURE_PULLED, *FIGHTER_STATUS_KIND_CAPTURE_WAIT, *FIGHTER_STATUS_KIND_CAPTURE_DAMAGE, *FIGHTER_STATUS_KIND_CAPTURE_CUT, *FIGHTER_STATUS_KIND_THROWN]
                            .contains(&prev_status_kind) 
                            ||
                            [*FIGHTER_STATUS_KIND_CAPTURE_PULLED, *FIGHTER_STATUS_KIND_CAPTURE_WAIT, *FIGHTER_STATUS_KIND_CAPTURE_DAMAGE, *FIGHTER_STATUS_KIND_CAPTURE_CUT, 
                            *FIGHTER_STATUS_KIND_ENTRY, *FIGHTER_STATUS_KIND_THROWN, *FIGHTER_STATUS_KIND_DAMAGE_FLY, *FIGHTER_STATUS_KIND_DAMAGE_FLY_ROLL, *FIGHTER_STATUS_KIND_DAMAGE_FLY_METEOR,
                            *FIGHTER_STATUS_KIND_DAMAGE_FLY_REFLECT_LR, *FIGHTER_STATUS_KIND_DAMAGE_FLY_REFLECT_U, *FIGHTER_STATUS_KIND_DAMAGE_FLY_REFLECT_D, *FIGHTER_STATUS_KIND_DAMAGE_FALL, 
                            *FIGHTER_STATUS_KIND_TREAD_DAMAGE_AIR, *FIGHTER_STATUS_KIND_BURY, *FIGHTER_STATUS_KIND_BURY_WAIT, *FIGHTER_STATUS_KIND_REBIRTH].contains(&status_kind);


    //Do ECB shifts for airborne players
    if situtation_kind == *SITUATION_KIND_AIR {
        if status_kind == *FIGHTER_STATUS_KIND_ESCAPE_AIR {
            if curr_frame <= 1.0 {
                ECY_Y_OFFSETS[get_player_number(boma)] = 6.0;
            }
            else if curr_frame <= 2.0 {
                ECY_Y_OFFSETS[get_player_number(boma)] = 0.0;
            }
            else{
                ECY_Y_OFFSETS[get_player_number(boma)] = max_offset;
            }
        }else{
            ECY_Y_OFFSETS[get_player_number(boma)] = max_offset;
        }



        offset.x = 0.0;
        offset.y = ECY_Y_OFFSETS[get_player_number(boma)];

        // Only change the current player's actual ingame ECB offset if both vanilla_ecb and air_trans are false; 
            //i.e. shift the ECB upwards if we're not in a state where we don't want it shifted and we've been in the air for more than 10 frames
        if !vanilla_ecb && !air_trans {
            GroundModule::set_rhombus_offset(boma, &offset);
        }

    }
    // ECB shifting behavior for grounded players
    else if situtation_kind == *SITUATION_KIND_GROUND {
        //Keep ECB shift at 0 if on the ground
        //max_offset = 0.0;
        offset.x = 0.0;
        offset.y = 0.0;
        if !vanilla_ecb {
            GroundModule::set_rhombus_offset(boma, &offset);
        }
    }

    else{

        ECY_Y_OFFSETS[get_player_number(boma)] = 0.0;
        offset.x = 0.0;
        offset.y = 0.0;

        if !vanilla_ecb {
            GroundModule::set_rhombus_offset(boma, &offset);
        }

    }

    //Shine specific ecbs
    if WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_FRAME_IN_AIR) < 5 && [*FIGHTER_KIND_FOX, *FIGHTER_KIND_WOLF, *FIGHTER_KIND_FALCO].contains(&fighter_kind)
    && situtation_kind == *SITUATION_KIND_AIR && in_range(curr_frame, 2., 5.) && [*FIGHTER_FOX_STATUS_KIND_SPECIAL_LW_HIT, *FIGHTER_FOX_STATUS_KIND_SPECIAL_LW_LOOP, *FIGHTER_STATUS_KIND_SPECIAL_LW].contains(&status_kind)
    {
        offset.x = 0.0;
        offset.y = -3.0;
        GroundModule::set_rhombus_offset(boma, &offset);
    }



}