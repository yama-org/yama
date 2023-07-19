local mp = require 'mp'
require 'mp.options'

local options = {
    min_time = 10.0,
}
read_options(options, "save_info")
local min_time = options.min_time

function on_unload()
    local filename = string.format("%s.%s", mp.get_property("path"):gsub("(.*)%..*$","%1"), "md")
    local file = io.open(filename, "w")
    local remaining = mp.get_property("time-remaining")
    local watched = tonumber(remaining) < min_time
    local current = watched and "0.00" or tostring(mp.get_property("time-pos"))

    file:write("{\n")
    file:write("\t\"Duration\": ", mp.get_property("duration"), ",\n")
    file:write("\t\"Current\": ", current, ",\n")
    file:write("\t\"Remaining\": ", remaining, ",\n")
    file:write("\t\"Watched\": ", tostring(watched), "\n")
    file:write("}")

    file:close(file)
end

mp.add_hook('on_unload', 50, on_unload)
