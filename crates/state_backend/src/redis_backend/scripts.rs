/// Fixed-window limiter implemented atomically in Redis via Lua.
///
/// Semantics:
/// - INCR the counter for {namespace}:{key}
/// - if first hit in the window, set PEXPIRE(window_ms)
/// - allow if count <= max
/// - deny otherwise, returning current PTTL as retry_after_ms
///
/// Returns: array [allowed(int 0/1), retry_after_ms(int)]
pub const FIXED_WINDOW_LUA: &str = r#"
local k = KEYS[1]
local window_ms = tonumber(ARGV[1])
local max = tonumber(ARGV[2])

local n = redis.call("INCR", k)
if n == 1 then
  redis.call("PEXPIRE", k, window_ms)
end

if n <= max then
  return {1, 0}
else
  local pttl = redis.call("PTTL", k)
  if pttl < 0 then
    pttl = window_ms
  end
  return {0, pttl}
end
"#;
