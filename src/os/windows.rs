/* No implementation yet.
 *
 * The goal is to normalize window's IOCP API to the various *NIX's readiness
 * model. This strategy will require maintaining a slab of buffers that will be
 * used to hold data as it is in-flight, allowing the user's buffer to remain
 * reusable.
 */

use error::{MioResult, MioError, ToMioError};
use net::{AddressFamily, SockAddr, IPv4Addr, SocketType};
use net::SocketType::{Dgram, Stream};
use net::SockAddr::{InetAddr, UnixAddr};
use net::AddressFamily::{Inet, Inet6, Unix};
pub use std::old_io::net::ip::IpAddr;

#[derive(Debug)]
pub struct IoDesc;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SysError {
	pub kind: u32
}

pub struct Awakener;

impl Awakener {
	pub fn new() -> MioResult<Awakener> {
        panic!();
    }

    pub fn wakeup(&self) -> MioResult<()> {
        panic!();
    }

    pub fn desc(&self) -> &IoDesc {
        panic!();
    }

    pub fn cleanup(&self) {
        panic!();
    }
}

impl ToMioError for SysError {
	fn to_mio_error(self) -> MioError {
		panic!();
	}
}

pub fn pipe() -> MioResult<(IoDesc, IoDesc)> {
    panic!();
}

/*
 *
 * ===== Sockets =====
 *
 */

pub fn socket(af: AddressFamily, sock_type: SocketType) -> MioResult<IoDesc> {
    panic!();
}

pub fn connect(io: &IoDesc, addr: &SockAddr) -> MioResult<bool> {
    panic!();
}

pub fn bind(io: &IoDesc, addr: &SockAddr) -> MioResult<()> {
    panic!();
}

pub fn listen(io: &IoDesc, backlog: usize) -> MioResult<()> {
    panic!();
}

pub fn accept(io: &IoDesc) -> MioResult<IoDesc> {
    panic!();
}

#[inline]
pub fn recvfrom(io: &IoDesc, buf: &mut [u8]) -> MioResult<(usize, SockAddr)> {
    panic!();
}

#[inline]
pub fn sendto(io: &IoDesc, buf: &[u8], tgt: &SockAddr) -> MioResult<usize> {
    panic!();
}

#[inline]
pub fn read(io: &IoDesc, dst: &mut [u8]) -> MioResult<usize> {
    panic!();
}

#[inline]
pub fn write(io: &IoDesc, src: &[u8]) -> MioResult<usize> {
    panic!();
}

// ===== Socket options =====

pub fn reuseaddr(_io: &IoDesc) -> MioResult<usize> {
    unimplemented!()
}

pub fn set_reuseaddr(io: &IoDesc, val: bool) -> MioResult<()> {
    panic!();
}

pub fn set_reuseport(io: &IoDesc, val: bool) -> MioResult<()> {
    panic!();
}

pub fn set_tcp_nodelay(io: &IoDesc, val: bool) -> MioResult<()> {
    panic!();
}

pub fn join_multicast_group(io: &IoDesc, addr: &IpAddr, interface: &Option<IpAddr>) -> MioResult<()> {
    panic!();
}

pub fn leave_multicast_group(io: &IoDesc, addr: &IpAddr, interface: &Option<IpAddr>) -> MioResult<()> {
    panic!();
}

pub fn set_multicast_ttl(io: &IoDesc, val: u8) -> MioResult<()> {
    panic!();
}

pub fn linger(io: &IoDesc) -> MioResult<usize> {
    panic!();
}

pub fn getpeername(io: &IoDesc) -> MioResult<SockAddr> {
    panic!();
}

pub fn getsockname(io: &IoDesc) -> MioResult<SockAddr> {
    panic!();
}

pub fn set_linger(io: &IoDesc, dur_s: usize) -> MioResult<()> {
    panic!();
}
