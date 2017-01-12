//! Communication between endpoints.
//!
//! The `socket` module deals with *network endpoints* and *buffering*.
//! It provides interfaces for accessing buffers of data, and protocol state machines
//! for filling and emptying these buffers.
//!
//! The programming interface implemented here differs greatly from the common Berkeley socket
//! interface. Specifically, in the Berkeley interface the buffering is implicit:
//! the operating system decides on the good size for a buffer and manages it.
//! The interface implemented by this module uses explicit buffering: you decide on the good
//! size for a buffer, allocate it, and let the networking stack use it.

use Error;
use wire::IpRepr;

mod udp;
mod tcp;
mod set;

pub use self::udp::PacketBuffer as UdpPacketBuffer;
pub use self::udp::SocketBuffer as UdpSocketBuffer;
pub use self::udp::UdpSocket;

pub use self::tcp::SocketBuffer as TcpSocketBuffer;
pub use self::tcp::State as TcpState;
pub use self::tcp::TcpSocket;

pub use self::set::{Set as SocketSet, Handle as SocketHandle};
pub use self::set::{Iter as SocketSetIter, IterMut as SocketSetIterMut};

/// A network socket.
///
/// This enumeration abstracts the various types of sockets based on the IP protocol.
/// To downcast a `Socket` value down to a concrete socket, use
/// the [AsSocket](trait.AsSocket.html) trait, and call e.g. `socket.as_socket::<UdpSocket<_>>()`.
///
/// The `process` and `dispatch` functions are fundamentally asymmetric and thus differ in
/// their use of the [trait PacketRepr](trait.PacketRepr.html). When `process` is called,
/// the packet length is already known and no allocation is required; on the other hand,
/// `process` would have to downcast a `&PacketRepr` to e.g. an `&UdpRepr` through `Any`,
/// which is rather inelegant. Conversely, when `dispatch` is called, the packet length is
/// not yet known and the packet storage has to be allocated; but the `&PacketRepr` is sufficient
/// since the lower layers treat the packet as an opaque octet sequence.
#[derive(Debug)]
pub enum Socket<'a, 'b: 'a> {
    Udp(UdpSocket<'a, 'b>),
    Tcp(TcpSocket<'a>),
    #[doc(hidden)]
    __Nonexhaustive
}

impl<'a, 'b> Socket<'a, 'b> {
    /// Process a packet received from a network interface.
    ///
    /// This function checks if the packet contained in the payload matches the socket endpoint,
    /// and if it does, copies it into the internal buffer, otherwise, `Err(Error::Rejected)`
    /// is returned.
    ///
    /// This function is used internally by the networking stack.
    pub fn process(&mut self, timestamp: u64, ip_repr: &IpRepr,
                   payload: &[u8]) -> Result<(), Error> {
        match self {
            &mut Socket::Udp(ref mut socket) =>
                socket.process(timestamp, ip_repr, payload),
            &mut Socket::Tcp(ref mut socket) =>
                socket.process(timestamp, ip_repr, payload),
            &mut Socket::__Nonexhaustive => unreachable!()
        }
    }

    /// Prepare a packet to be transmitted to a network interface.
    ///
    /// This function checks if the internal buffer is empty, and if it is not, calls `f` with
    /// the representation of the packet to be transmitted, otherwise, `Err(Error::Exhausted)`
    /// is returned.
    ///
    /// This function is used internally by the networking stack.
    pub fn dispatch<F, R>(&mut self, timestamp: u64, emit: &mut F) -> Result<R, Error>
            where F: FnMut(&IpRepr, &IpPayload) -> Result<R, Error> {
        match self {
            &mut Socket::Udp(ref mut socket) =>
                socket.dispatch(timestamp, emit),
            &mut Socket::Tcp(ref mut socket) =>
                socket.dispatch(timestamp, emit),
            &mut Socket::__Nonexhaustive => unreachable!()
        }
    }
}

/// An IP-encapsulated packet representation.
///
/// This trait abstracts the various types of packets layered under the IP protocol,
/// and serves as an accessory to [trait Socket](trait.Socket.html).
pub trait IpPayload {
    /// Return the length of the buffer required to serialize this high-level representation.
    fn buffer_len(&self) -> usize;

    /// Emit this high-level representation into a sequence of octets.
    fn emit(&self, ip_repr: &IpRepr, payload: &mut [u8]);
}

/// A conversion trait for network sockets.
///
/// This trait is used to concisely downcast [Socket](trait.Socket.html) values to their
/// concrete types.
pub trait AsSocket<T> {
    fn as_socket(&mut self) -> &mut T;
}

impl<'a, 'b> AsSocket<UdpSocket<'a, 'b>> for Socket<'a, 'b> {
    fn as_socket(&mut self) -> &mut UdpSocket<'a, 'b> {
        match self {
            &mut Socket::Udp(ref mut socket) => socket,
            _ => panic!(".as_socket::<UdpSocket> called on wrong socket type")
        }
    }
}

impl<'a, 'b> AsSocket<TcpSocket<'a>> for Socket<'a, 'b> {
    fn as_socket(&mut self) -> &mut TcpSocket<'a> {
        match self {
            &mut Socket::Tcp(ref mut socket) => socket,
            _ => panic!(".as_socket::<TcpSocket> called on wrong socket type")
        }
    }
}
