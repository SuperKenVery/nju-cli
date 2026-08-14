//! 南京大学 Web VPN 登录库。
//!
//! [`LoginOperation`] 将短信认证拆成可序列化并跨进程恢复的两阶段流程，库本身不会读取
//! 标准输入或阻塞等待验证码。

mod models;
mod operation;
mod request;
mod utils;

pub use operation::LoginOperation;

#[cfg(test)]
mod tests;
