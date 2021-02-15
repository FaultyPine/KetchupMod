
pub unsafe fn get_player_number(boma: &mut smash::app::BattleObjectModuleAccessor) -> usize {
    smash::app::lua_bind::WorkModule::get_int(boma, *smash::lib::lua_const::FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize
}

extern "C"{
    #[link_name = "\u{1}_ZN3app14sv_information11is_ready_goEv"]
    pub fn is_ready_go() -> bool;
}

pub fn in_range(num: f32, lower: f32, upper: f32) -> bool{
    num>lower && num<upper
}

pub unsafe fn clamp(x: f32, min: f32, max: f32) -> f32 {
    return if x < min { min } else if x < max { x } else { max };
}


pub unsafe fn clear_buffered_action(flag: i32, cmd: i32) -> i32 {
    return flag & !(1 << cmd);
}

pub unsafe fn add_buffered_action(flag: i32, cmd: i32) -> i32 {
    return flag | cmd;
}

pub unsafe fn compare_cat(cat: i32, fighter_pad_cmd_flag: i32) -> bool {
    return (cat & fighter_pad_cmd_flag) != 0;
}