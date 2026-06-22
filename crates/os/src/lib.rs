pub mod env;
pub mod exec;
pub mod info;
pub mod process;

use helper::Value;

pub fn os_exec(args: &[Value]) -> Result<Value, String> {
    exec::os_exec(args)
}
pub fn os_exit(args: &[Value]) -> Result<Value, String> {
    process::os_exit(args)
}
pub fn os_env_get(args: &[Value]) -> Result<Value, String> {
    env::os_env_get(args)
}
pub fn os_env_set(args: &[Value]) -> Result<Value, String> {
    env::os_env_set(args)
}
pub fn os_platform(args: &[Value]) -> Result<Value, String> {
    info::os_platform(args)
}
pub fn os_cwd(args: &[Value]) -> Result<Value, String> {
    info::os_cwd(args)
}
pub fn os_hostname(args: &[Value]) -> Result<Value, String> {
    info::os_hostname(args)
}
pub fn os_args(args: &[Value]) -> Result<Value, String> {
    info::os_args(args)
}
pub fn os_cpu(args: &[Value]) -> Result<Value, String> {
    info::os_cpu(args)
}
pub fn os_ram(args: &[Value]) -> Result<Value, String> {
    info::os_ram(args)
}
