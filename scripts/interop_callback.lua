function make_callback()
    return function(value)
        return interop_callback(value)
    end
end
