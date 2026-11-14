//! What the 电脑状态 example shows, read from Windows directly: the CPU load, the memory in
//! use, the local drives' free space and this PC's IPv4 address on the network. The example's
//! PowerShell asks WMI for the same numbers; PowerShell kept running for that took about
//! 100 MB of memory, these calls take next to none.

use crate::bindings::*;
use crate::wide::to_wide;
use std::net::Ipv4Addr;
use windows_core::PCWSTR;

/// Idle and total CPU time of all processors since Windows started, in 100 ns units.
#[derive(Clone, Copy, Debug, Default)]
pub struct CpuTimes {
    idle: u64,
    total: u64,
}

impl CpuTimes {
    pub fn now() -> Option<Self> {
        let (mut idle, mut kernel, mut user) = Default::default();
        // SAFETY: three live FILETIMEs.
        unsafe { GetSystemTimes(Some(&mut idle), Some(&mut kernel), Some(&mut user)) }
            .ok()
            .ok()?;
        let ticks = |t: FILETIME| (u64::from(t.dwHighDateTime) << 32) | u64::from(t.dwLowDateTime);
        // Kernel time includes the idle time.
        Some(Self {
            idle: ticks(idle),
            total: ticks(kernel) + ticks(user),
        })
    }

    /// The CPU load between `earlier` and these times, in percent.
    pub fn load_since(&self, earlier: &Self) -> u32 {
        let total = self.total.saturating_sub(earlier.total);
        let idle = self.idle.saturating_sub(earlier.idle).min(total);
        if total == 0 {
            return 0;
        }
        ((total - idle) as f64 * 100.0 / total as f64).round() as u32
    }
}

/// The physical memory in use, in percent (what is not available, as Task Manager counts it).
pub fn memory_load() -> Option<u32> {
    let mut m = MEMORYSTATUSEX {
        dwLength: size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };
    // SAFETY: a live struct with its length set.
    unsafe { GlobalMemoryStatusEx(&mut m) }.ok().ok()?;
    if m.ullTotalPhys.0 == 0 {
        return None;
    }
    Some((100.0 - m.ullAvailPhys.0 as f64 * 100.0 / m.ullTotalPhys.0 as f64).round() as u32)
}

/// A local drive with a disk in it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Drive {
    pub letter: char,
    pub free: u64,
    pub size: u64,
}

/// Removable drives, fixed drives and RAM disks with a disk in them, by letter. Network drives
/// (a lost one can hang) and CD/DVD drives are never asked.
pub fn local_drives() -> Vec<Drive> {
    const REMOVABLE: u32 = 2;
    const FIXED: u32 = 3;
    const RAMDISK: u32 = 6;
    // An empty card reader answers with an error instead of a "no disk" message box.
    // SAFETY: plain flag; the old mode is not needed back on this worker thread.
    unsafe {
        let _ = SetThreadErrorMode(SEM_FAILCRITICALERRORS as u32, None);
    }
    // SAFETY: no arguments; returns a bitmask.
    let drives = unsafe { GetLogicalDrives() };
    let mut out = Vec::new();
    for i in 0..26u8 {
        if drives & (1 << i) == 0 {
            continue;
        }
        let letter = (b'A' + i) as char;
        let root = to_wide(&format!("{letter}:\\"));
        // SAFETY: the string is NUL-terminated and outlives the calls.
        let kind = unsafe { GetDriveTypeW(PCWSTR(root.as_ptr())) };
        if ![REMOVABLE, FIXED, RAMDISK].contains(&kind) {
            continue;
        }
        let (mut size, mut free) = (0u64, 0u64);
        // SAFETY: as above; two live out values.
        let ok = unsafe {
            GetDiskFreeSpaceExW(
                PCWSTR(root.as_ptr()),
                None,
                Some(&mut size),
                Some(&mut free),
            )
        }
        .as_bool();
        if ok && size > 0 {
            out.push(Drive { letter, free, size });
        }
    }
    out
}

/// A network adapter as the address choice needs it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Adapter {
    pub up: bool,
    /// It has an IPv4 default gateway.
    pub gateway: bool,
    /// A Wi-Fi or Ethernet card (a connector is present); a VPN or a virtual switch is not.
    pub physical: bool,
    pub metric: u32,
    pub addresses: Vec<Ipv4Addr>,
}

/// This PC's IPv4 address on the network: from a connected adapter with a default gateway (an
/// unplugged adapter can keep an old address, virtual networks have none), Wi-Fi and Ethernet
/// before VPNs, then the one Windows uses first; no loopback or link-local address.
pub fn pick_ipv4(adapters: &[Adapter]) -> Option<Ipv4Addr> {
    let mut usable: Vec<&Adapter> = adapters.iter().filter(|a| a.up && a.gateway).collect();
    usable.sort_by_key(|a| (!a.physical, a.metric));
    usable
        .iter()
        .flat_map(|a| a.addresses.iter())
        .find(|ip| !ip.is_loopback() && !ip.is_link_local())
        .copied()
}

/// The adapters with their IPv4 addresses.
pub fn adapters() -> Vec<Adapter> {
    let flags = (GAA_FLAG_INCLUDE_GATEWAYS
        | GAA_FLAG_SKIP_ANYCAST
        | GAA_FLAG_SKIP_MULTICAST
        | GAA_FLAG_SKIP_DNS_SERVER
        | GAA_FLAG_SKIP_FRIENDLY_NAME) as u32;
    // Microsoft's advice: start with 15 KB and grow when told.
    let mut buf: Vec<u64> = vec![0; 15 * 1024 / 8];
    let mut list = std::ptr::null_mut::<IP_ADAPTER_ADDRESSES_LH>();
    for _ in 0..3 {
        let mut len = (buf.len() * 8) as u32;
        let first = buf.as_mut_ptr().cast::<IP_ADAPTER_ADDRESSES_LH>();
        // SAFETY: `buf` is `len` bytes, 8-byte aligned, and lives until the list is read.
        let r = unsafe { GetAdaptersAddresses(AF_INET as u32, flags, None, Some(first), &mut len) };
        if r == ERROR_BUFFER_OVERFLOW as u32 {
            buf = vec![0; (len as usize).div_ceil(8)];
            continue;
        }
        if r == 0 {
            list = first;
        }
        break;
    }
    let mut out = Vec::new();
    let mut at = list;
    while !at.is_null() {
        // SAFETY: a node of the list Windows wrote into `buf`.
        let a = unsafe { &*at };
        let mut addresses = Vec::new();
        let mut u = a.FirstUnicastAddress;
        while !u.is_null() {
            // SAFETY: a node of the same list.
            let ua = unsafe { &*u };
            if let Some(ip) = ipv4_of(&ua.Address) {
                addresses.push(ip);
            }
            u = ua.Next;
        }
        let mut gateway = false;
        let mut g = a.FirstGatewayAddress;
        while !g.is_null() {
            // SAFETY: a node of the same list.
            let ga = unsafe { &*g };
            gateway |= ipv4_of(&ga.Address).is_some();
            g = ga.Next;
        }
        out.push(Adapter {
            up: a.OperStatus == IfOperStatusUp,
            gateway,
            physical: connector_present(a.Luid),
            metric: a.Ipv4Metric,
            addresses,
        });
        at = a.Next;
    }
    out
}

fn ipv4_of(address: &SOCKET_ADDRESS) -> Option<Ipv4Addr> {
    if address.lpSockaddr.is_null() || (address.iSockaddrLength as usize) < size_of::<SOCKADDR_IN>()
    {
        return None;
    }
    // SAFETY: a socket address of at least SOCKADDR_IN's size, from the adapter list.
    let sa = unsafe { &*address.lpSockaddr.cast::<SOCKADDR_IN>() };
    if i32::from(sa.sin_family.0) != AF_INET {
        return None;
    }
    // SAFETY: the union's plain 32-bit form, in network byte order.
    let raw = unsafe { sa.sin_addr.S_un.S_addr };
    Some(Ipv4Addr::from(u32::from_be(raw)))
}

/// Whether the adapter has a hardware connector (Get-NetAdapter's ConnectorPresent).
fn connector_present(luid: IF_LUID) -> bool {
    let mut row = MIB_IF_ROW2 {
        InterfaceLuid: luid,
        ..Default::default()
    };
    // SAFETY: a live row with its LUID set.
    if unsafe { GetIfEntry2(&mut row) }.is_err() {
        return false;
    }
    // The flags are a C bit field: HardwareInterface, FilterInterface, ConnectorPresent, ...
    // Read as a byte (the binding's bool may hold other values than 0 and 1).
    // SAFETY: a one-byte field of the live row.
    let flags = unsafe { *std::ptr::addr_of!(row.InterfaceAndOperStatusFlags).cast::<u8>() };
    flags & 0b100 != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn adapter(up: bool, ips: &[&str], gateway: bool, physical: bool, metric: u32) -> Adapter {
        Adapter {
            up,
            gateway,
            physical,
            metric,
            addresses: ips.iter().map(|s| s.parse().unwrap()).collect(),
        }
    }

    // The cases of scripts/test-pc-status.ps1, which checks the PowerShell example.
    #[test]
    fn the_address_comes_from_a_connected_gateway_adapter() {
        let pick = |list: &[Adapter]| pick_ipv4(list).map(|ip| ip.to_string()).unwrap_or_default();
        // Disconnected stale IP and virtual adapters before Wi-Fi.
        assert_eq!(
            pick(&[
                adapter(false, &["192.168.8.117"], true, true, 35),
                adapter(true, &["172.21.208.1"], false, true, 35),
                adapter(true, &["172.20.192.1"], false, true, 35),
                adapter(true, &["192.168.40.162"], true, true, 35),
            ]),
            "192.168.40.162"
        );
        // Several addresses on one adapter: the first.
        assert_eq!(
            pick(&[adapter(true, &["10.0.0.2", "10.0.0.3"], true, true, 35)]),
            "10.0.0.2"
        );
        // No loopback or link-local address.
        assert_eq!(
            pick(&[adapter(
                true,
                &["127.0.0.1", "169.254.1.2", "10.0.0.2"],
                true,
                true,
                35
            )]),
            "10.0.0.2"
        );
        // No connected adapter with a gateway.
        assert_eq!(
            pick(&[
                adapter(false, &["192.168.8.117"], true, true, 35),
                adapter(true, &["172.21.208.1"], false, true, 35),
            ]),
            ""
        );
        assert_eq!(pick(&[adapter(true, &[], false, true, 35)]), "");
        // A VPN with its own gateway and a lower metric comes after Wi-Fi.
        assert_eq!(
            pick(&[
                adapter(true, &["198.18.0.1"], true, false, 5),
                adapter(true, &["192.168.40.162"], true, true, 35),
            ]),
            "192.168.40.162"
        );
        // Of two cards, the lower metric.
        assert_eq!(
            pick(&[
                adapter(true, &["192.168.40.162"], true, true, 35),
                adapter(true, &["192.168.40.20"], true, true, 25),
            ]),
            "192.168.40.20"
        );
        // Only a virtual switch has the gateway (a Hyper-V external switch).
        assert_eq!(
            pick(&[
                adapter(true, &[], false, true, 25),
                adapter(true, &["192.168.1.50"], true, false, 15),
            ]),
            "192.168.1.50"
        );
    }

    #[test]
    fn cpu_load_is_the_busy_share_of_the_time_between() {
        let a = CpuTimes {
            idle: 100,
            total: 1000,
        };
        let b = CpuTimes {
            idle: 175,
            total: 1100,
        };
        assert_eq!(b.load_since(&a), 25);
        assert_eq!(a.load_since(&a), 0);
    }
}
