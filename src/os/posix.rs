use std::mem;
use std::num::Int;
use std::error::FromError;
use error::{MioResult, MioError, MioErrorKind, ToMioError};
use net::{AddressFamily, SockAddr, IPv4Addr, SocketType};
use net::SocketType::{Dgram, Stream};
use net::SockAddr::{InetAddr, UnixAddr};
use net::AddressFamily::{Inet, Inet6, Unix};
pub use std::old_io::net::ip::IpAddr;
pub use nix::errno::SysError;

mod nix {
    pub use nix::c_int;
    pub use nix::fcntl::{Fd, O_NONBLOCK, O_CLOEXEC};
    pub use nix::errno::{EINPROGRESS, EAGAIN, EADDRINUSE};
    pub use nix::sys::socket::*;
    pub use nix::unistd::*;
}

/*
 *
 * ===== Awakener =====
 *
 */

pub struct PipeAwakener {
    reader: IoDesc,
    writer: IoDesc
}

impl PipeAwakener {
    pub fn new() -> MioResult<PipeAwakener> {
        let (rd, wr) = try!(pipe());

        Ok(PipeAwakener {
            reader: rd,
            writer: wr
        })
    }

    pub fn wakeup(&self) -> MioResult<()> {
        write(&self.writer, b"0x01")
            .map(|_| ())
    }

    pub fn desc(&self) -> &IoDesc {
        &self.reader
    }

    pub fn cleanup(&self) {
        let mut buf: [u8; 128] = unsafe { mem::uninitialized() };

        loop {
            // Consume data until all bytes are purged
            match read(&self.reader, buf.as_mut_slice()) {
                Ok(_) => {}
                Err(_) => return
            }
        }
    }
}

/// Represents the OS's handle to the IO instance. In this case, it is the file
/// descriptor.
#[derive(Show)]
pub struct IoDesc {
    pub fd: nix::Fd
}

impl Drop for IoDesc {
    fn drop(&mut self) {
        let _ = nix::close(self.fd);
    }
}

/*
 *
 * ===== Pipes =====
 *
 */

pub fn pipe() -> MioResult<(IoDesc, IoDesc)> {
    let (rd, wr) = try!(nix::pipe2(nix::O_NONBLOCK | nix::O_CLOEXEC));

    Ok((IoDesc { fd: rd }, IoDesc { fd: wr }))
}

/*
 *
 * ===== Sockets =====
 *
 */

pub fn socket(af: AddressFamily, sock_type: SocketType) -> MioResult<IoDesc> {
    let family = match af {
        Inet  => nix::AF_INET,
        Inet6 => nix::AF_INET6,
        Unix  => nix::AF_UNIX
    };

    let socket_type = match sock_type {
        Dgram  => nix::SOCK_DGRAM,
        Stream => nix::SOCK_STREAM
    };

    Ok(IoDesc {
        fd: try!(nix::socket(family, socket_type, nix::SOCK_NONBLOCK | nix::SOCK_CLOEXEC))
    })
}

pub fn connect(io: &IoDesc, addr: &SockAddr) -> MioResult<bool> {
    match nix::connect(io.fd, &from_sockaddr(addr)) {
        Ok(_) => Ok(true),
        Err(e) => {
            match e.kind {
                nix::EINPROGRESS => Ok(false),
                _ => Err(FromError::from_error(e))
            }
        }
    }
}

pub fn bind(io: &IoDesc, addr: &SockAddr) -> MioResult<()> {
    Ok(try!(nix::bind(io.fd, &from_sockaddr(addr))))
}

pub fn listen(io: &IoDesc, backlog: usize) -> MioResult<()> {
    Ok(try!(nix::listen(io.fd, backlog)))
}

pub fn accept(io: &IoDesc) -> MioResult<IoDesc> {
    Ok(IoDesc {
        fd: try!(nix::accept4(io.fd, nix::SOCK_NONBLOCK | nix::SOCK_CLOEXEC))
    })
}

#[inline]
pub fn recvfrom(io: &IoDesc, buf: &mut [u8]) -> MioResult<(usize, SockAddr)> {
    let (cnt, addr) = try!(nix::recvfrom(io.fd, buf));
	Ok((cnt, to_sockaddr(&addr)))
}

#[inline]
pub fn sendto(io: &IoDesc, buf: &[u8], tgt: &SockAddr) -> MioResult<usize> {
    let res = try!(nix::sendto(io.fd, buf, &from_sockaddr(tgt), nix::MSG_DONTWAIT));
    Ok(res)
}

#[inline]
pub fn read(io: &IoDesc, dst: &mut [u8]) -> MioResult<usize> {
    let res = try!(nix::read(io.fd, dst));

    if res == 0 {
        return Err(MioError::eof());
    }

    Ok(res)
}

#[inline]
pub fn write(io: &IoDesc, src: &[u8]) -> MioResult<usize> {
	Ok(try!(nix::write(io.fd, src)))
}

// ===== Socket options =====

pub fn reuseaddr(_io: &IoDesc) -> MioResult<usize> {
    unimplemented!()
}

pub fn set_reuseaddr(io: &IoDesc, val: bool) -> MioResult<()> {
    let v: nix::c_int = if val { 1 } else { 0 };

    Ok(try!(nix::setsockopt(io.fd, nix::SOL_SOCKET, nix::SO_REUSEADDR, &v)))
}

pub fn set_reuseport(io: &IoDesc, val: bool) -> MioResult<()> {
    let v: nix::c_int = if val { 1 } else { 0 };

    Ok(try!(nix::setsockopt(io.fd, nix::SOL_SOCKET, nix::SO_REUSEPORT, &v)))
}

pub fn set_tcp_nodelay(io: &IoDesc, val: bool) -> MioResult<()> {
    let v: nix::c_int = if val { 1 } else { 0 };

    Ok(try!(nix::setsockopt(io.fd, nix::IPPROTO_TCP, nix::TCP_NODELAY, &v)))
}

pub fn join_multicast_group(io: &IoDesc, addr: &IpAddr, interface: &Option<IpAddr>) -> MioResult<()> {
    let grp_req = try!(make_ip_mreq(addr, interface));

    Ok(try!(nix::setsockopt(io.fd, nix::IPPROTO_IP, nix::IP_ADD_MEMBERSHIP, &grp_req)))
}

pub fn leave_multicast_group(io: &IoDesc, addr: &IpAddr, interface: &Option<IpAddr>) -> MioResult<()> {
    let grp_req = try!(make_ip_mreq(addr, interface));

    Ok(try!(nix::setsockopt(io.fd, nix::IPPROTO_IP, nix::IP_ADD_MEMBERSHIP, &grp_req)))
}

pub fn set_multicast_ttl(io: &IoDesc, val: u8) -> MioResult<()> {
    let v: nix::IpMulticastTtl = val;

    Ok(try!(nix::setsockopt(io.fd, nix::IPPROTO_IP, nix::IP_MULTICAST_TTL, &v)))
}

pub fn linger(io: &IoDesc) -> MioResult<usize> {
    let mut linger: nix::linger = unsafe { mem::uninitialized() };

    try!(nix::getsockopt(io.fd, nix::SOL_SOCKET, nix::SO_LINGER, &mut linger));

    if linger.l_onoff > 0 {
        Ok(linger.l_linger as usize)
    } else {
        Ok(0)
    }
}

pub fn getpeername(io: &IoDesc) -> MioResult<SockAddr> {
    let sa : nix::sockaddr_in = unsafe { mem::zeroed() };
    let mut a = nix::SockAddr::SockIpV4(sa);

    try!(nix::getpeername(io.fd, &mut a));

    Ok(to_sockaddr(&a))
}

pub fn getsockname(io: &IoDesc) -> MioResult<SockAddr> {
    let sa : nix::sockaddr_in = unsafe { mem::zeroed() };
    let mut a = nix::SockAddr::SockIpV4(sa);

    try!(nix::getsockname(io.fd, &mut a));

    Ok(to_sockaddr(&a))
}

pub fn set_linger(io: &IoDesc, dur_s: usize) -> MioResult<()> {
    let linger = nix::linger {
        l_onoff: (if dur_s > 0 { 1is } else { 0is }) as nix::c_int,
        l_linger: dur_s as nix::c_int
    };

    Ok(try!(nix::setsockopt(io.fd, nix::SOL_SOCKET, nix::SO_LINGER, &linger)))
}

fn make_ip_mreq(group_addr: &IpAddr, iface_addr: &Option<IpAddr>) -> MioResult<nix::ip_mreq> {
    Ok(nix::ip_mreq {
        imr_multiaddr: from_ip_addr_to_inaddr(&Some(*group_addr)),
        imr_interface: from_ip_addr_to_inaddr(iface_addr)
    })
}

fn from_ip_addr_to_inaddr(addr: &Option<IpAddr>) -> nix::in_addr {
    match *addr {
        Some(ip) => {
            match ip {
                IPv4Addr(a, b, c, d) => ipv4_to_inaddr(a, b, c, d),
                _ => unimplemented!()
            }
        }
        None => nix::in_addr { s_addr: nix::INADDR_ANY }
    }
}

fn to_sockaddr(addr: &nix::SockAddr) -> SockAddr {
    match *addr {
        nix::SockAddr::SockIpV4(sin) => {
            InetAddr(u32be_to_ipv4(sin.sin_addr.s_addr), Int::from_be(sin.sin_port))
        }
        nix::SockAddr::SockUnix(addr) => {
            let mut str_path = String::new();
            for c in addr.sun_path.iter() {
                if *c == 0 { break; }
                str_path.push(*c as u8 as char);
            }

            UnixAddr(Path::new(str_path))
        }
        _ => unimplemented!()
    }
}

fn from_sockaddr(addr: &SockAddr) -> nix::SockAddr {
    use std::mem;

    match *addr {
        InetAddr(ip, port) => {
            match ip {
                IPv4Addr(a, b, c, d) => {
                    let mut addr: nix::sockaddr_in = unsafe { mem::zeroed() };

                    addr.sin_family = nix::AF_INET as nix::sa_family_t;
                    addr.sin_port = port.to_be();
                    addr.sin_addr = ipv4_to_inaddr(a, b, c, d);

                    nix::SockAddr::SockIpV4(addr)
                }
                _ => unimplemented!()
            }
        }
        UnixAddr(ref path) => {
            let mut addr: nix::sockaddr_un = unsafe { mem::zeroed() };

            addr.sun_family = nix::AF_UNIX as nix::sa_family_t;

            let c_path_ptr = path.as_vec();
            assert!(c_path_ptr.len() < addr.sun_path.len());
            for (sp_iter, path_iter) in addr.sun_path.iter_mut().zip(c_path_ptr.iter()) {
                *sp_iter = *path_iter as i8;
            }

            nix::SockAddr::SockUnix(addr)
        }
    }
}

fn ipv4_to_u32(a: u8, b: u8, c: u8, d: u8) -> nix::InAddrT {
    Int::from_be(((a as u32) << 24) |
                 ((b as u32) << 16) |
                 ((c as u32) <<  8) |
                 ((d as u32) <<  0))
}

fn u32be_to_ipv4(net: u32) -> IpAddr {
    u32_to_ipv4(Int::from_be(net))
}

fn u32_to_ipv4(net: u32) -> IpAddr {
    IPv4Addr(((net >> 24) & 0xff) as u8,
         ((net >> 16) & 0xff) as u8,
         ((net >> 8) & 0xff) as u8,
         (net & 0xff) as u8)
}

fn ipv4_to_inaddr(a: u8, b: u8, c: u8, d: u8) -> nix::in_addr {
    nix::in_addr {
        s_addr: ipv4_to_u32(a, b, c, d)
    }
}

