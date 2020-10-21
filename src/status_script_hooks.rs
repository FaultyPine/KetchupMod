//this folder is for status script hooks relating to more general mechanics
use smash::app;
use smash::lib::{lua_const::*, L2CValue, L2CAgent};
use smash::lua2cpp::L2CFighterCommon;


//Turn dash (runs once at the beginning of the status)
//fixes "loose" feeling that ult dashdances had... still not perfect but with this its definitely better
#[skyline::hook(replace = smash::lua2cpp::L2CFighterCommon_status_TurnDash_Sub)]
pub unsafe fn status_turndash_sub_hook(fighter: &mut L2CFighterCommon) -> L2CValue {    
    let ret = original!()(fighter);
        
    let mut l2c_agent = L2CAgent::new(fighter.lua_state_agent);
    l2c_agent.clear_lua_stack();
    l2c_agent.push_lua_stack(&mut L2CValue::new_int(*FIGHTER_KINETIC_ENERGY_ID_MOTION as u64));
    l2c_agent.push_lua_stack(&mut L2CValue::new_num(0.0));
    app::sv_kinetic_energy::set_speed(fighter.lua_state_agent);

    ret
}