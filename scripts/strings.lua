local count = 5000
local unicode_text = "héllo—世界🚀"

local function split(text, separator)
    local result = {}
    local start = 1
    while true do
        local first, last = string.find(text, separator, start, true)
        if not first then
            table.insert(result, string.sub(text, start))
            break
        end
        table.insert(result, string.sub(text, start, first - 1))
        start = last + 1
    end
    return result
end

function bench()
    local parts = {}
    for index = 0, count - 1 do
        parts[index + 1] = string.format("item-%d", index)
    end

    local joined = table.concat(parts, ",")
    local split_parts = split(joined, ",")
    local pipe_joined = table.concat(split_parts, "|")
    local replaced = string.gsub(pipe_joined, "item%-", "row-")
    local marked = string.gsub(replaced, "row%-4999", "last", 1)
    local unicode_length = utf8.len(unicode_text)

    local checksum = #joined + #pipe_joined + #marked + #split_parts + unicode_length
    if string.find(marked, "last", 1, true) then
        checksum = checksum + 1
    end
    return checksum
end
