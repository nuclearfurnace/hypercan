use std::{
    io,
    os::unix::prelude::{AsRawFd, RawFd},
    task::Poll,
    time::Duration,
};

use can::identifier::Id;
use futures::ready;
use mio::{event::Source, unix::SourceFd, Interest, Registry, Token};
use socketcan_isotp::{IsoTpBehaviour, IsoTpOptions, IsoTpSocket};
use tokio::{io::unix::AsyncFd, macros::support::poll_fn, time::timeout};

use crate::common::config::CANParameters;

use super::error::{SocketBuildError, SocketError};

#[derive(Default)]
pub struct ISOTPSocketBuilder {
    source_id: Option<Id>,
    destination_id: Option<Id>,
    can_parameters: Option<CANParameters>,
}

impl ISOTPSocketBuilder {
    pub fn source_id(mut self, id: impl Into<Id>) -> Self {
        self.source_id = Some(id.into());
        self
    }

    pub fn destination_id(mut self, id: impl Into<Id>) -> Self {
        self.destination_id = Some(id.into());
        self
    }

    pub fn can_parameters(mut self, params: CANParameters) -> Self {
        self.can_parameters = Some(params);
        self
    }

    pub fn build(self) -> Result<ISOTPSocket, SocketBuildError> {
        let source_id = self
            .source_id
            .ok_or(SocketBuildError::MissingRequiredField {
                field_name: "source_id",
            })?;
        let destination_id = self
            .destination_id
            .ok_or(SocketBuildError::MissingRequiredField {
                field_name: "destination_id",
            })?;
        let can_parameters = self
            .can_parameters
            .ok_or(SocketBuildError::MissingRequiredField {
                field_name: "can_parameters",
            })?;

        let mut isotp_options = IsoTpOptions::default();
        if !can_parameters.disable_isotp_frame_padding {
            isotp_options.set_txpad_content(can_parameters.tx_frame_padding);

            let new_flags = isotp_options
                .get_flags()
                .map(|flags| flags | IsoTpBehaviour::CAN_ISOTP_TX_PADDING)
                .unwrap_or(IsoTpBehaviour::CAN_ISOTP_TX_PADDING);

            isotp_options.set_flags(new_flags);
        }

        let socket = IsoTpSocket::open_with_opts(
            can_parameters.socket_name.as_ref(),
            source_id,
            destination_id,
            Some(isotp_options),
            None,
            None,
        )?;
        socket.set_nonblocking(true)?;

        Ok(ISOTPSocket {
            inner: AsyncFd::new(EventedISOTPSocket { inner: socket })?,
            default_read_timeout: Some(can_parameters.read_timeout),
            default_write_timeout: Some(can_parameters.write_timeout),
        })
    }
}

pub struct EventedISOTPSocket {
    inner: IsoTpSocket,
}

impl EventedISOTPSocket {
    fn get_ref(&self) -> &IsoTpSocket {
        &self.inner
    }

    fn get_mut(&mut self) -> &mut IsoTpSocket {
        &mut self.inner
    }
}

impl AsRawFd for EventedISOTPSocket {
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

impl Source for EventedISOTPSocket {
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

pub struct ISOTPSocket {
    inner: AsyncFd<EventedISOTPSocket>,
    default_read_timeout: Option<Duration>,
    default_write_timeout: Option<Duration>,
}

impl ISOTPSocket {
    pub fn builder() -> ISOTPSocketBuilder {
        ISOTPSocketBuilder::default()
    }

    pub async fn read(&mut self) -> Result<Vec<u8>, SocketError> {
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

    pub async fn write(&mut self, buf: &[u8]) -> Result<(), SocketError> {
        let write = poll_fn(|cx| loop {
            let mut ready_guard = ready!(self.inner.poll_write_ready_mut(cx))?;
            match ready_guard.try_io(|inner| evented_write(inner, buf)) {
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

fn evented_read_owned(af: &mut AsyncFd<EventedISOTPSocket>) -> io::Result<Vec<u8>> {
    af.get_mut().get_mut().read().map(|b| b.to_vec())
}

fn evented_write(af: &AsyncFd<EventedISOTPSocket>, buf: &[u8]) -> io::Result<()> {
    af.get_ref().get_ref().write(buf)
}
