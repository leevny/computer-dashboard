//! GPU 利用率 / 显存 / 按进程占用，来自 Windows PDH 性能计数器 + DXGI 适配器信息。
//! 全厂商通用（N/A/I 卡），无需厂商 SDK。
use std::collections::HashMap;
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE};
use windows::Win32::System::Performance::{
    PdhAddEnglishCounterW, PdhCollectQueryData, PdhGetFormattedCounterArrayW, PdhOpenQueryW,
    PDH_FMT_COUNTERVALUE_ITEM_W, PDH_FMT_DOUBLE, PDH_HCOUNTER, PDH_HQUERY,
};

fn check(code: u32, what: &str) -> Result<(), String> {
    if code == 0 {
        Ok(())
    } else {
        Err(format!("{what}: PDH 0x{code:08x}"))
    }
}

pub struct Gpu {
    query: PDH_HQUERY,
    util: PDH_HCOUNTER,      // \GPU Engine(*)\Utilization Percentage
    dedicated: PDH_HCOUNTER, // \GPU Adapter Memory(*)\Dedicated Usage → 总专用显存
    proc_mem: PDH_HCOUNTER,  // \GPU Process Memory(*)\Dedicated Usage → 按进程显存
}

impl Gpu {
    pub fn new() -> Result<Self, String> {
        let mut query = PDH_HQUERY::default();
        unsafe { check(PdhOpenQueryW(None, 0, &mut query), "PdhOpenQueryW")? };
        let add = |path: &str| -> Result<PDH_HCOUNTER, String> {
            let mut w: Vec<u16> = path.encode_utf16().chain([0]).collect();
            let mut c = PDH_HCOUNTER::default();
            unsafe { check(PdhAddEnglishCounterW(query, PCWSTR(w.as_ptr()), 0, &mut c), "PdhAddEnglishCounterW")? };
            Ok(c)
        };
        let util = add(r"\GPU Engine(*)\Utilization Percentage")?;
        let dedicated = add(r"\GPU Adapter Memory(*)\Dedicated Usage")?;
        let proc_mem = add(r"\GPU Process Memory(*)\Dedicated Usage")?;
        unsafe { check(PdhCollectQueryData(query), "PdhCollectQueryData")? }; // 基线采样，读数从第二次 poll 开始有效
        Ok(Self { query, util, dedicated, proc_mem })
    }

    /// 返回 (按 pid 的 GPU 利用率%, 总专用显存字节, 按 pid 的专用显存字节)
    pub fn poll(&self) -> (HashMap<u32, f32>, u64, HashMap<u32, u64>) {
        unsafe { PdhCollectQueryData(self.query) };

        // 利用率：按 pid 聚合所有 engine 实例
        let mut pid_util: HashMap<u32, f32> = HashMap::new();
        for (inst, v) in items(self.util) {
            if let Some(pid) = pid_of(&inst) {
                *pid_util.entry(pid).or_default() += v as f32;
            }
        }
        for v in pid_util.values_mut() {
            *v = v.min(100.0); // ponytail: 多卡/多 engine 求和后截断，多卡精确分卡再加
        }

        let dedicated_total: u64 = items(self.dedicated).iter().map(|(_, v)| *v as u64).sum();

        let mut pid_mem: HashMap<u32, u64> = HashMap::new();
        for (inst, v) in items(self.proc_mem) {
            if let Some(pid) = pid_of(&inst) {
                *pid_mem.entry(pid).or_default() += v as u64;
            }
        }
        (pid_util, dedicated_total, pid_mem)
    }

    /// 总专用显存上限（字节），来自 DXGI；排除软件渲染适配器
    pub fn total_dedicated() -> u64 {
        // ponytail: 多卡时用总量求和做单一表盘，分卡显示再加
        let mut total = 0u64;
        unsafe {
            if let Ok(factory) = CreateDXGIFactory1::<IDXGIFactory1>() {
                let mut i = 0u32;
                while let Ok(adapter) = factory.EnumAdapters1(i) {
                    i += 1;
                    if let Ok(desc) = adapter.GetDesc1() {
                        if desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32 == 0 {
                            total += desc.DedicatedVideoMemory as u64;
                        }
                    }
                }
            }
        }
        total
    }
}

/// instance 名形如 "pid_1234_luid_..._engtype_3D"，取 pid 段
fn pid_of(inst: &str) -> Option<u32> {
    let s = inst.strip_prefix("pid_")?;
    let end = s.find('_').unwrap_or(s.len());
    s[..end].parse().ok()
}

/// 拉取通配符计数器的全部实例值（instance 名, double）
fn items(counter: PDH_HCOUNTER) -> Vec<(String, f64)> {
    unsafe {
        let mut size = 0u32;
        let mut count = 0u32;
        // 首次调用必然 PDH_MORE_DATA，只为了拿缓冲区大小
        let _ = PdhGetFormattedCounterArrayW(counter, PDH_FMT_DOUBLE, &mut size, &mut count, None);
        if size == 0 {
            return Vec::new();
        }
        let mut buf = vec![0u8; size as usize];
        if PdhGetFormattedCounterArrayW(
            counter,
            PDH_FMT_DOUBLE,
            &mut size,
            &mut count,
            Some(buf.as_mut_ptr().cast()),
        ) != 0
        {
            return Vec::new();
        }
        let base = buf.as_ptr() as *const PDH_FMT_COUNTERVALUE_ITEM_W;
        (0..count as usize)
            .map(|i| &*base.add(i))
            // CStatus != 0 时 FmtValue 联合体里是垃圾值（可能是 Inf/NaN），必须丢弃
            .filter(|it| it.FmtValue.CStatus == 0)
            .map(|it| (PWSTR(it.szName.as_ptr()).to_string().unwrap_or_default(), it.FmtValue.Anonymous.doubleValue))
            .collect()
    }
}
