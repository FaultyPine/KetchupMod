Lib.open_dir = "C:/Users/gclar/Desktop/Modding/ArcCross/root" -- absolute path to your ArcCross\root folder
Lib.save_dir = "KetchupModRomfs" -- can specify absolute or relative paths

local start = clock()

do
    local root = assert(Lib:open("fighter/common/param/fighter_param.prc"))
    -- "t" here means a table containing each fighter param struct
    local t = assert(root:by_hash(hash("fighter_param_table"))):to_table()

    local char_mods = { -- all character-specific changes go here (fwiw, these take priority over all_char_mods)
        mario = {
            --jump_y = 45.0
        },
		
    }
	
	local all_char_mods = { -- all changes that apply for every character
		--escape_air_landing_frame = 40
	}

	--	convert the indeces of the table from strings to their hash
    do -- for char_mods
        local _mods = {}
        for charname, chartable in pairs(char_mods) do
            _mods[hash("fighter_kind_"..charname)] = chartable
        end
        char_mods = _mods
    end
    do -- for all_char_mods
        local _mods = {}
        for param_name, param_value in pairs(all_char_mods) do
            _mods[hash(param_name)] = param_value
        end
        all_char_mods = _mods
    end
	

    for _, p in ipairs(t) do -- iterates through fighter_param_motion_table
	
		for param, value in pairs(all_char_mods) do
			assert(p:by_hash(param)).value = value
		end
	
        local ft_kind_hash = assert(p:by_hash(hash("fighter_kind"))).value -- assumes there WILL be a fighter_kind key.. assigns value of that to ft_kind_hash
        local char_mod_table = char_mods[ft_kind_hash] -- actual param struct

        if char_mod_table then
            for param_name, value in pairs(char_mod_table) do -- iterates through each *param* in a fighter_param_motion_table struct
                assert(p:by_hash(hash(param_name))).value = value
            end
        end
    end

    Lib:save(root) -- saves to "mods/fighter/common/param/fighter_param_motion.prc"
end

print("elapsed: "..clock() - start)