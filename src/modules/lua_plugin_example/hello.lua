-- Simple Lua plugin example for wigspace-rust
-- This function will be called by the Rust host

function handle(input)
    return "[lua_plugin] got: " .. input
end

-- Optional initialization function
function init()
    return "[lua_plugin] initialized"
end

-- Optional cleanup function
function cleanup()
    return "[lua_plugin] cleanup completed"
end