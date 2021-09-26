use std;

use crate::errors::{ ErrorKind, Result };
use std::io::Write;
// 两种 Delta 对象的压缩解压逻辑
pub const OFS_DELTA: u8 = 6;
pub const REF_DELTA: u8 = 7;

#[derive(Debug)]
struct CopyState {
    offset: usize,
    extent: usize
}

#[derive(Debug)]
struct InsertState {
    extent: usize
}

#[derive(Debug)]
enum DeltaDecoderState {
    NextCommand,
    Copy(CopyState),
    Insert(InsertState),
    Done
}

#[derive(Debug)]
pub struct DeltaDecoderStream {
    state: DeltaDecoderState,
    instructions: Vec<u8>,
    inner: Vec<u8>,
    index: usize,
    output_size: usize,
    written: usize
}

#[derive(Debug)]
pub struct DeltaDecoder {
    instructions: Vec<u8>,
    inner: Vec<u8>,
    output_size: usize
}

impl DeltaDecoder {
    pub fn new (instructions: &[u8], base: Vec<u8>) -> Result<DeltaDecoder> {
        let (after_base_size, base_size) = read_varint(0, instructions);
        let (index, output_size) = read_varint(after_base_size, instructions);

        if base.len() != base_size {
            return Err(ErrorKind::BadDeltaBase.into())
        }

        Ok(DeltaDecoder {
            instructions: Vec::from(&instructions[index..]),
            output_size,
            inner: base
        })
    }

    pub fn output_size (&self) -> usize {
        self.output_size
    }
}

impl std::convert::Into<DeltaDecoderStream> for DeltaDecoder {
    fn into(self) -> DeltaDecoderStream {
        DeltaDecoderStream {
            instructions: self.instructions,
            index: 0,
            written: 0,
            state: DeltaDecoderState::NextCommand,
            inner: self.inner,
            output_size: self.output_size
        }
    }
}

fn read_varint(base_offset: usize, bytes: &[u8]) -> (usize, usize) {
    let mut shift: usize = 0;
    let mut result: usize = 0;
    let mut offset = base_offset;

    while {
        let byt = bytes[offset];
        result += ((byt & 0x7F) as usize) << shift;
        shift += 7;
        offset += 1;
        byt >= 0x80
    } {}

    (offset, result)
}

impl std::io::Read for DeltaDecoderStream {
    fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize> {
        let mut written = 0;
        loop {
            let (next_state, exhausted) = match &self.state {
                DeltaDecoderState::Done => {
                    if self.written != self.output_size {
                        return Err(std::io::ErrorKind::WriteZero.into())
                    }
                    return Ok(written)
                },

                DeltaDecoderState::NextCommand => {
                    if self.index >= self.instructions.len() {
                        self.written += written;
                        (DeltaDecoderState::Done, false)
                    } else {
                        let cmd = self.instructions[self.index];
                        self.index += 1;
                        let next = if (cmd & 0x80) != 0 {
                            // copy
                            let mut check = 1;
                            let mut offset: usize = 0;
                            let mut extent: usize = 0;

                            for i in 0..4 {
                                if (cmd & check) != 0 {
                                    offset |= (self.instructions[self.index] as usize) << (8 * i);
                                    self.index += 1;
                                }
                                check <<= 1;
                            }

                            for i in 0..3 {
                                if (cmd & check) != 0 {
                                    extent |= (self.instructions[self.index] as usize) << (8 * i);
                                    self.index += 1;
                                }
                                check <<= 1;
                            }

                            // read 4 bytes out of delta -> extent

                            extent &= 0x00FF_FFFF;
                            extent = if extent == 0 { 0x10000 } else { extent };
                            DeltaDecoderState::Copy(CopyState {
                                offset,
                                extent
                            })
                        } else {
                            DeltaDecoderState::Insert(InsertState {
                                extent: cmd as usize
                            })
                        };

                        (next, false)
                    }
                },

                DeltaDecoderState::Copy(state) => {
                    let wrote = buf.write(&self.inner[state.offset .. state.offset + state.extent])?;

                    let extent = state.extent - wrote;
                    let offset = state.offset + wrote;
                    written += wrote;
                    if extent == 0 {
                        (DeltaDecoderState::NextCommand, false)
                    } else {
                        (DeltaDecoderState::Copy(CopyState {
                            extent,
                            offset
                        }), true)
                    }
                },

                DeltaDecoderState::Insert(state) => {
                    let wrote = buf.write(&self.instructions[self.index .. self.index + state.extent])?;
                    self.index += wrote;
                    let extent = state.extent - wrote;
                    written += wrote;

                    if extent == 0 {
                        (DeltaDecoderState::NextCommand, false)
                    } else {
                        (DeltaDecoderState::Insert(InsertState {
                            extent
                        }), true)
                    }
                }
            };

            self.state = next_state;
            if exhausted {
                break
            }
        }

        self.written += written;
        Ok(written)
    }
}
