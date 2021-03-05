Lib.open_dir = "C:/Users/gclar/Desktop/param_b0000" -- absolute path to your ArcCross\root folder
Lib.save_dir = "KetchupModRomfs" -- can specify absolute or relative paths

local start = clock()

do
    local root = assert(Lib:open("fighter/common/param/common.prc"))

	local mods = {
        turn_dash_frame = -1,
		landing_frame_escape_air = 15
	}
	--	convert the indeces of the table from strings to their hash
    do
        local _mods = {}
        for param_name, value in pairs(mods) do
            _mods[hash(param_name)] = value
        end
        mods = _mods
    end

    for param_name, value in pairs(mods) do
        root:by_hash(param_name).value = value
    end
    Lib:save(root) -- saves to "mods/fighter/common/param/fighter_param_motion.prc"
end

print("elapsed: "..clock() - start)