//! 工具模块
//!
//! 提供各种辅助功能，包括字符串转换、数学计算、WMI 查询、注册表操作和设备操作等。

pub mod device;
pub mod macros;
pub mod math;
pub mod registry;
pub mod string;
pub mod wmi;
pub mod wmi_date;

pub use math::{cm_to_inches, diagonal_inches_from_cm, div};
pub use string::{u16_bytes_to_string, u16_slice_to_string, u8_slice_to_string, wide_str};
pub use wmi_date::parse_wmi_date;
