use crate::models::hardware::MemBreakdown;

const VM_VM_STATISTICS64: libc::c_int = 6;

#[cfg(target_os = "macos")]
pub fn coletar_breakdown() -> Option<MemBreakdown> {
    use std::mem;

    let mut page_size = 0u64;
    let mut size = mem::size_of::<u64>();
    let mut mib_pg = [libc::CTL_HW, libc::HW_PAGESIZE];
    let ret = unsafe {
        libc::sysctl(
            mib_pg.as_mut_ptr(),
            mib_pg.len() as u32,
            &mut page_size as *mut _ as *mut libc::c_void,
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };
    if ret != 0 || page_size == 0 {
        return None;
    }

    let mut vm_stat: libc::vm_statistics64 = unsafe { mem::zeroed() };
    let mut size = mem::size_of::<libc::vm_statistics64>();
    let mut mib_vm = [libc::CTL_VM, VM_VM_STATISTICS64];
    let ret = unsafe {
        libc::sysctl(
            mib_vm.as_mut_ptr(),
            mib_vm.len() as u32,
            &mut vm_stat as *mut _ as *mut libc::c_void,
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };
    if ret != 0 {
        return None;
    }

    let app = vm_stat.active_count as u64 * page_size;
    let wired = vm_stat.wire_count as u64 * page_size;
    let compressed = vm_stat.compressor_page_count as u64 * page_size;
    let cached = (vm_stat.inactive_count + vm_stat.purgeable_count) as u64 * page_size;

    Some(MemBreakdown {
        app_memory: app,
        wired_memory: wired,
        compressed_memory: compressed,
        cached_memory: cached,
    })
}

#[cfg(not(target_os = "macos"))]
pub fn coletar_breakdown() -> Option<MemBreakdown> {
    None
}
