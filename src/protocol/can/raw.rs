use std::{
    io,
    os::unix::prelude::{AsRawFd, RawFd},
    task::Poll,
    time::Duration,
};

use futures::ready;
use mio::{event::Source, unix::SourceFd, Interest, Registry, Token};
use socketcan::{CANFilter, CANFrame, CANSocket};
use tokio::{io::unix::AsyncFd, macros::support::poll_fn, time::timeout};

use crate::common::config::CANParameters;

use super::error::{SocketBuildError, SocketError};

#[derive(Default)]
pub struct RawSocketBuilder {
    source_id_filter: Option<CANFilter>,
    can_parameters: Option<CANParameters>,
}

impl RawSocketBuilder {
    pub fn source_id_filter(mut self, filter: CANFilter) -> Self {
        self.source_id_filter = Some(filter);
        self
    }

    pub fn can_parameters(mut self, params: CANParameters) -> Self {
        self.can_parameters = Some(params);
        self
    }

    pub fn build(self) -> Result<RawSocket, SocketBuildError> {
        let source_id_filter = self.source_id_filter;
        let can_parameters = self
            .can_parameters
            .ok_or(SocketBuildError::MissingRequiredField {
                field_name: "can_parameters",
            })?;

        let socket = CANSocket::open(can_parameters.socket_name.as_ref())?;
        socket.set_nonblocking(true)?;

        if let Some(filter) = source_id_filter {
            socket.set_filter(&[filter])?;
        } else {
            socket.filter_accept_all()?;
        }

        Ok(RawSocket {
            inner: AsyncFd::new(EventedRawSocket { inner: socket })?,
            default_read_timeout: Some(can_parameters.read_timeout),
            default_write_timeout: Some(can_parameters.write_timeout),
        })
    }
}

pub struct EventedRawSocket {
    inner: CANSocket,
}

impl EventedRawSocket {
    fn get_ref(&self) -> &CANSocket {
        &self.inner
    }

    fn get_mut(&mut self) -> &mut CANSocket {
        &mut self.inner
    }
}

impl AsRawFd for EventedRawSocket {
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

impl Source for EventedRawSocket {
    fn register(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> io::Result<()> {
        SourceFd(&self.inner.as_raw_fd()).register(registry, token, interests)
    }

    fn reregister(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> io::Result<()> {
        SourceFd(&self.inner.as_raw_fd()).reregister(registry, token, interests)
    }

    fn deregister(&mut self, registry: &Registry) -> io::Result<()> {
        SourceFd(&self.inner.as_raw_fd()).deregister(registry)
    }
}

pub struct RawSocket {
    inner: AsyncFd<EventedRawSocket>,
    default_read_timeout: Option<Duration>,
    default_write_timeout: Option<Duration>,
}

impl RawSocket {
    pub fn builder() -> RawSocketBuilder {
        RawSocketBuilder::default()
    }

    pub async fn read(&mut self) -> Result<CANFrame, SocketError> {
        let read = poll_fn(|cx| loop {
            let mut ready_guard = ready!(self.inner.poll_read_ready_mut(cx))?;
            match ready_guard.try_io(evented_read_owned) {
                Ok(result) => return Poll::Ready(result),
                Err(_would_block) => continue,
            }
        });

        let result = if let Some(duration) = self.default_read_timeout {
            timeout(duration, read)
                .await
                .map_err(|_| SocketError::Timeout(duration))?
        } else {
            read.await
        };
        result.map_err(Into::into)
    }

    pub async fn write(&mut self, frame: CANFrame) -> Result<(), SocketError> {
        let write = poll_fn(|cx| loop {
            let mut ready_guard = ready!(self.inner.poll_write_ready_mut(cx))?;
            match ready_guard.try_io(|inner| evented_write(inner, &frame)) {
                Ok(result) => return Poll::Ready(result),
                Err(_would_block) => continue,
            }
        });

        let result = if let Some(duration) = self.default_write_timeout {
            timeout(duration, write)
                .await
                .map_err(|_| SocketError::Timeout(duration))?
        } else {
            write.await
        };
        result.map_err(Into::into)
    }
}

fn evented_read_owned(af: &mut AsyncFd<EventedRawSocket>) -> io::Result<CANFrame> {
    af.get_mut().get_mut().read_frame()
}

fn evented_write(af: &AsyncFd<EventedRawSocket>, frame: &CANFrame) -> io::Result<()> {
    af.get_ref().get_ref().write_frame(&frame)
}
