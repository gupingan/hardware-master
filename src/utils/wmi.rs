//! Windows WMI (Windows Management Instrumentation) 查询模块
//!
//! 提供简化的 WMI 查询接口，封装了 Windows COM 初始化、WMI 服务连接和查询操作。

use windows::core::BSTR;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoSetProxyBlanket, CLSCTX_ALL, COINIT_MULTITHREADED,
    EOAC_NONE, RPC_C_AUTHN_LEVEL_PKT_PRIVACY, RPC_C_IMP_LEVEL_IMPERSONATE,
};
use windows::Win32::System::Ole::{
    SafeArrayAccessData, SafeArrayGetLBound, SafeArrayGetUBound, SafeArrayUnaccessData,
};
use windows::Win32::System::Rpc::{RPC_C_AUTHN_NONE, RPC_C_AUTHN_WINNT};
use windows::Win32::System::Variant::{VariantInit, VARIANT, VT_BSTR, VT_I2, VT_I4, VT_UI1, VT_UI2, VT_UI8, VT_BOOL};
use windows::Win32::System::Wmi::{
    IEnumWbemClassObject, IWbemClassObject, IWbemLocator, IWbemServices, WbemLocator,
    WBEM_FLAG_FORWARD_ONLY, WBEM_FLAG_RETURN_IMMEDIATELY, WBEM_INFINITE,
};

/// WMI 连接配置
#[derive(Debug, Clone)]
pub struct WmiConfig {
    /// WMI 命名空间，如 "ROOT\\CIMV2" 或 "ROOT\\wmi"
    pub namespace: String,
}

impl Default for WmiConfig {
    fn default() -> Self {
        Self {
            namespace: "ROOT\\CIMV2".to_string(),
        }
    }
}

/// WMI 查询结果迭代器
pub struct WmiQueryResult {
    enumerator: IEnumWbemClassObject,
}

impl WmiQueryResult {
    /// 获取下一个 WMI 对象
    pub unsafe fn next(&mut self) -> Option<IWbemClassObject> {
        let mut objs = [None; 1];
        let mut returned = 0u32;

        let result = self.enumerator.Next(WBEM_INFINITE, &mut objs, &mut returned);

        if result.is_err() || returned == 0 {
            return None;
        }

        objs[0].take()
    }
}

/// WMI 客户端
pub struct WmiClient {
    _server: IWbemServices,
}

impl WmiClient {
    /// 创建新的 WMI 客户端
    ///
    /// # 参数
    /// * `config` - WMI 连接配置
    ///
    /// # 示例
    /// ```ignore
    /// use hardware_master::utils::wmi::{WmiClient, WmiConfig};
    ///
    /// let config = WmiConfig {
    ///     namespace: "ROOT\\CIMV2".to_string(),
    /// };
    /// let client = WmiClient::connect(&config)?;
    /// ```
    pub unsafe fn connect(config: &WmiConfig) -> Result<Self, String> {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

        let locator: IWbemLocator = CoCreateInstance(&WbemLocator, None, CLSCTX_ALL)
            .map_err(|e| format!("创建 WMI 定位器失败: {:?}", e))?;

        let server: IWbemServices = locator
            .ConnectServer(
                &BSTR::from(&config.namespace),
                &BSTR::new(),
                &BSTR::new(),
                &BSTR::new(),
                0,
                &BSTR::new(),
                None,
            )
            .map_err(|e| format!("连接到 WMI 服务失败: {:?}", e))?;

        CoSetProxyBlanket(
            &server,
            RPC_C_AUTHN_WINNT,
            RPC_C_AUTHN_NONE,
            None,
            RPC_C_AUTHN_LEVEL_PKT_PRIVACY,
            RPC_C_IMP_LEVEL_IMPERSONATE,
            None,
            EOAC_NONE,
        )
        .ok();

        Ok(Self { _server: server })
    }

    /// 执行 WQL 查询
    ///
    /// # 参数
    /// * `query` - WQL 查询语句
    ///
    /// # 示例
    /// ```ignore
    /// let result = client.query("SELECT * FROM Win32_PhysicalMemory")?;
    /// ```
    pub unsafe fn query(&self, query: &str) -> Result<WmiQueryResult, String> {
        let enumerator: IEnumWbemClassObject = self._server
            .ExecQuery(
                &BSTR::from("WQL"),
                &BSTR::from(query),
                WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY,
                None,
            )
            .map_err(|e| format!("执行 WMI 查询失败: {:?}", e))?;

        Ok(WmiQueryResult { enumerator })
    }
}

/// 获取 WMI 对象属性
///
/// # 参数
/// * `obj` - WMI 对象
/// * `name` - 属性名称
pub unsafe fn get_property(obj: &IWbemClassObject, name: &str) -> windows::core::Result<VARIANT> {
    let mut var = VariantInit();
    obj.Get(&BSTR::from(name), 0, &mut var, None, None)?;
    Ok(var)
}

/// 将 VARIANT 转换为字符串
pub unsafe fn variant_to_string(var: &VARIANT) -> Option<String> {
    let vt = var.Anonymous.Anonymous.vt.0 as u32;
    if vt == VT_BSTR.0 as u32 {
        let bstr = &var.Anonymous.Anonymous.Anonymous.bstrVal;
        Some(bstr.to_string())
    } else {
        None
    }
}

/// 将 VARIANT 转换为 bool
pub  unsafe fn variant_to_bool(var: &VARIANT) -> Option<bool> {
    let vt = var.Anonymous.Anonymous.vt.0 as u32;
    if vt == VT_BOOL.0 as u32 {
        Some(var.Anonymous.Anonymous.Anonymous.boolVal.as_bool())
    } else {
        None
    }
}

/// 将 VARIANT 转换为 u8
pub unsafe fn variant_to_u8(var: &VARIANT) -> Option<u8> {
    let vt = var.Anonymous.Anonymous.vt.0 as u32;
    if vt == VT_UI1.0 as u32 {
        Some(var.Anonymous.Anonymous.Anonymous.bVal)
    } else {
        None
    }
}

/// 将 VARIANT 转换为 u16
pub unsafe fn variant_to_u16(var: &VARIANT) -> Option<u16> {
    let vt = var.Anonymous.Anonymous.vt.0 as u32;
    if vt == VT_UI1.0 as u32 {
        Some(var.Anonymous.Anonymous.Anonymous.bVal as u16)
    } else if vt == VT_I2.0 as u32 {
        let val = var.Anonymous.Anonymous.Anonymous.iVal;
        Some(val as u16)
    } else if vt == VT_I4.0 as u32 {
        let val = var.Anonymous.Anonymous.Anonymous.lVal;
        Some(val as u16)
    } else if vt == VT_UI2.0 as u32 {
        let val = var.Anonymous.Anonymous.Anonymous.uiVal;
        Some(val)
    } else {
        None
    }
}

/// 将 VARIANT 转换为 u32
pub unsafe fn variant_to_u32(var: &VARIANT) -> Option<u32> {
    let vt = var.Anonymous.Anonymous.vt.0 as u32;
    if vt == VT_I4.0 as u32 {
        Some(var.Anonymous.Anonymous.Anonymous.lVal as u32)
    } else {
        None
    }
}

/// 将 VARIANT 转换为 u64
pub unsafe fn variant_to_u64(var: &VARIANT) -> Option<u64> {
    let vt = var.Anonymous.Anonymous.vt.0 as u32;

    // VT_UI8 = 21 (64位无符号整数)
    if vt == VT_UI8.0 as u32 {
        Some(var.Anonymous.Anonymous.Anonymous.ullVal)
    }
    // VT_I4 = 3 (32位有符号整数)
    else if vt == VT_I4.0 as u32 {
        Some(var.Anonymous.Anonymous.Anonymous.lVal as u64)
    }
    // VT_BSTR = 8 (字符串类型，WMI 有时会把数字作为字符串返回)
    else if vt == VT_BSTR.0 as u32 {
        let bstr = &var.Anonymous.Anonymous.Anonymous.bstrVal;
        let s = bstr.to_string();
        s.trim().parse::<u64>().ok()
    } else {
        None
    }
}

/// 将 VARIANT 转换为 u16 切片
///
/// WMI 中的 u16 数组通常以字节数组形式存储
pub unsafe fn variant_to_u16_slice(var: &VARIANT) -> Option<Vec<u16>> {
    let vt = var.Anonymous.Anonymous.vt.0 as u32;

    // VT_I2 = 3 (16位有符号整数)
    // VT_ARRAY = 0x2000
    // 对于 int16 数组 (VT_I2 | VT_ARRAY)
    if vt == (3 | 0x2000) as u32 {
        let p_array = var.Anonymous.Anonymous.Anonymous.parray;
        if p_array.is_null() {
            return None;
        }

        // 获取数组边界
        let l_bound = match SafeArrayGetLBound(p_array, 1) {
            Ok(bound) => bound,
            Err(_) => return None,
        };
        let u_bound = match SafeArrayGetUBound(p_array, 1) {
            Ok(bound) => bound,
            Err(_) => return None,
        };

        let count = (u_bound - l_bound + 1) as usize;

        // 获取数组数据
        let mut pv_data: *mut core::ffi::c_void = std::ptr::null_mut();
        if SafeArrayAccessData(p_array, &mut pv_data).is_err() {
            return None;
        }

        // 对于 VT_I2 数组，每个元素是 2 字节
        // 但是数据可能以字节形式存储，需要按字节读取
        let byte_ptr = pv_data as *const u8;
        let mut result = Vec::with_capacity(count / 2);

        for i in (0..count).step_by(2) {
            // 小端序：低字节在前，高字节在后
            let low = *byte_ptr.add(i) as u16;
            let high = *byte_ptr.add(i + 1) as u16;
            let val = low | (high << 8);
            // 过滤掉 null 值
            if val != 0 {
                result.push(val);
            }
        }

        let _ = SafeArrayUnaccessData(p_array);

        Some(result)
    } else if vt == (VT_UI1.0 | 0x2000) as u32 {
        // 字节数组，尝试转换为 u16 数组
        let p_array = var.Anonymous.Anonymous.Anonymous.parray;
        if p_array.is_null() {
            return None;
        }

        let l_bound = match SafeArrayGetLBound(p_array, 1) {
            Ok(bound) => bound,
            Err(_) => return None,
        };
        let u_bound = match SafeArrayGetUBound(p_array, 1) {
            Ok(bound) => bound,
            Err(_) => return None,
        };

        let count = (u_bound - l_bound + 1) as usize;

        let mut pv_data: *mut core::ffi::c_void = std::ptr::null_mut();
        if SafeArrayAccessData(p_array, &mut pv_data).is_err() {
            return None;
        }

        let byte_ptr = pv_data as *const u8;
        let mut result = Vec::with_capacity(count / 2);

        for i in (0..count).step_by(2) {
            let low = *byte_ptr.add(i) as u16;
            let high = *byte_ptr.add(i + 1) as u16;
            result.push(low | (high << 8));
        }

        let _ = SafeArrayUnaccessData(p_array);

        Some(result)
    } else {
        None
    }
}
