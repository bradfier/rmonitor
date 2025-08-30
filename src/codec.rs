//! A tokio-util [`Decoder`] implementation for RMonitor.
//!
//! [`Decoder`]: tokio_util::codec::Decoder
use thiserror::Error;
use tokio_util::bytes::BytesMut;
use tokio_util::codec::{Decoder, LinesCodec, LinesCodecError};

use crate::protocol::*;

/// An error was encountered when trying to decode an RMonitor record from
/// the input byte stream.
#[derive(Error, Debug)]
pub enum RMonitorCodecError {
    /// An error occured when trying to decode a Record from an otherwise valid line
    #[error("unable to decode record from line '{line}': {source}")]
    RecordDecode {
        line: String,
        #[source]
        source: RecordError,
    },
    /// The underlying LinesCodec encountered an error trying to extract a single line
    #[error(transparent)]
    LinesCodec(#[from] LinesCodecError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// A decoder for RMonitor records, which wraps an underlying [`LinesCodec`]
/// to provide framing logic.
///
/// [`LinesCodec`]: tokio_util::codec::LinesCodec
#[derive(Default, Debug)]
pub struct RMonitorDecoder {
    lines_codec: LinesCodec,
}

impl RMonitorDecoder {
    /// Returns an `RMonitorDecoder` for decoding RMonitor records from a TCP stream.
    ///
    /// # Note
    ///
    /// The returned `RMonitorDecoder` will have an underlying `LinesCodec` with no upper
    /// bound on the length of a buffered line. Consider using [`new_with_max_length`] instead.
    ///
    /// [`new_with_max_length`]: crate::codec::RMonitorDecoder::new_with_max_length()
    pub fn new() -> Self {
        Self {
            lines_codec: LinesCodec::new(),
        }
    }

    /// Returns an `RMonitorDecoder` where the underlying `LinesCodec` has a maximum line length
    /// limit.
    ///
    /// It is recommended to set such a limit where the input to be supplied to the decoder is
    /// untrusted, as an attacker could send an unbounded amount of input with no newline
    /// characters.
    pub fn new_with_max_length(max_length: usize) -> Self {
        Self {
            lines_codec: LinesCodec::new_with_max_length(max_length),
        }
    }
}

impl Decoder for RMonitorDecoder {
    type Item = Record;
    type Error = RMonitorCodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let line = self.lines_codec.decode(src)?;
        if let Some(line) = line {
            // If we've somehow started decoding in the middle of a record, or this line is
            // completely empty, discard it and continue from the next one.
            if line.is_empty() || line.as_bytes()[0] != b'$' {
                return Ok(None);
            }
            Ok(Some(Record::decode(&line).map_err(|source| {
                RMonitorCodecError::RecordDecode {
                    line: line.clone(),
                    source,
                }
            })?))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn consume(
        decoder: &mut RMonitorDecoder,
        bytes: &mut BytesMut,
    ) -> Vec<Result<Option<Record>, RMonitorCodecError>> {
        let mut result = vec![];
        loop {
            match decoder.decode(bytes) {
                Ok(None) => {
                    break;
                }
                out => result.push(out),
            }
        }
        result
    }

    #[test]
    fn test_decodes_single_line() {
        let mut decoder = RMonitorDecoder::new_with_max_length(2048);
        let mut bytes =
            BytesMut::from("$F,9999,\"00:00:00\",\"14:09:52\",\"00:59:59\",\"      \"\r\n");

        let result = consume(&mut decoder, &mut bytes);

        assert_eq!(0, bytes.len());
        assert_eq!(1, result.len());

        assert!(matches!(result[0], Ok(Some(Record::Heartbeat(_)))));
    }

    #[test]
    fn test_decodes_large_sample() {
        let mut decoder = RMonitorDecoder::new_with_max_length(2048);
        let data: Vec<u8> = std::fs::read("sample/2009_Sebring_ALMS_Session_5.txt").unwrap();

        let mut bytes = BytesMut::from(data.as_slice());

        let result = consume(&mut decoder, &mut bytes);

        // All bytes were consumed
        assert_eq!(0, bytes.len());
        // And no errors were encountered when processing the file
        assert!(result.into_iter().all(|r| r.is_ok()));
    }

    #[test]
    fn test_error_includes_line_content() {
        let mut decoder = RMonitorDecoder::new_with_max_length(2048);
        // Create a malformed line that should trigger a decode error
        let mut bytes = BytesMut::from("$F,invalid,data,here\r\n");

        let result = decoder.decode(&mut bytes);

        assert!(result.is_err());
        let error = result.unwrap_err();

        // Check that the error message includes both the line content and the underlying error
        let error_msg = format!("{}", error);
        assert!(error_msg.contains("$F,invalid,data,here"));
        assert!(error_msg.contains("unable to decode record from line"));
    }
}
