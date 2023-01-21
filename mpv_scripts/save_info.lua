local mp = require 'mp'

function on_unload()
    local filename = string.format("%s.%s", mp.get_property("path"):gsub("(.*)%..*$","%1"), "md")
    local file = io.open(filename, "w")
    local remaining = mp.get_property("time-remaining")
    local status = tostring(tonumber(remaining) < 10.0)

    file:write("Duration: ", mp.get_property("duration"), "\n")
    file:write("Current: ", mp.get_property("time-pos"), "\n")
    file:write("Remaining: ", remaining, "\n")
    file:write("Status: ", status, "\n")
    
    file:close(file)
end

mp.add_hook('on_unload', 50, on_unload)