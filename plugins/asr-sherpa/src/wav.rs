//! 读 16 kHz 单声道 16 位 PCM 的 WAV（`audio:pcm16k`）。别的格式一律不收，不重采样。
//!
//! 头自己解析：跳过 `LIST` 一类不认识的块（ffmpeg 常在 `data` 前写一个），奇数长的块后面有一个补齐字节。
//! 采样按块读出来，内存不随时长涨。

use std::fs::File;
use std::io::{self, BufReader, Read, Seek, SeekFrom};
use std::path::Path;

pub const SAMPLE_RATE: u32 = 16_000;

const FORMAT_PCM: u16 = 1;
const FORMAT_EXTENSIBLE: u16 = 0xFFFE;

#[derive(Debug, thiserror::Error)]
pub enum WavError {
    #[error("read the WAV: {0}")]
    Io(#[from] io::Error),
    #[error("not a RIFF/WAVE file")]
    NotWave,
    #[error("no `fmt ` chunk before `data`")]
    NoFormat,
    #[error("no `data` chunk")]
    NoData,
    #[error("format tag {0:#06x} is not PCM")]
    NotPcm(u16),
    #[error("{0} channels; only mono is accepted")]
    Channels(u16),
    #[error("{0} Hz; only {SAMPLE_RATE} Hz is accepted")]
    SampleRate(u32),
    #[error("{0} bits per sample; only 16 is accepted")]
    Bits(u16),
}

impl WavError {
    /// 文件本身读不出来（宿主给的路径有问题），不是格式不对。
    pub fn is_io(&self) -> bool {
        matches!(self, WavError::Io(_))
    }
}

/// `data` 块：从文件开头算的位置和字节数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DataChunk {
    offset: u64,
    len: u64,
}

/// 一个打开的 16 kHz 单声道 WAV，读到哪算到哪。
pub struct Pcm16k {
    reader: BufReader<File>,
    /// 还没读的采样数。
    left: u64,
    samples: u64,
}

impl Pcm16k {
    pub fn open(path: &Path) -> Result<Self, WavError> {
        let mut file = File::open(path)?;
        let file_len = file.metadata()?.len();
        let data = parse_header(&mut BufReader::new(&mut file), file_len)?;
        file.seek(SeekFrom::Start(data.offset))?;
        let samples = data.len / 2;
        Ok(Self {
            reader: BufReader::new(file),
            left: samples,
            samples,
        })
    }

    pub fn duration_sec(&self) -> f64 {
        self.samples as f64 / f64::from(SAMPLE_RATE)
    }

    /// 读最多 `max` 个采样到 `out`（先清空），换成 -1..1 的浮点；读完了 `out` 为空。
    pub fn read_block(&mut self, out: &mut Vec<f32>, max: usize) -> io::Result<()> {
        out.clear();
        let n = self.left.min(max as u64) as usize;
        let mut bytes = vec![0u8; n * 2];
        self.reader.read_exact(&mut bytes)?;
        out.extend(
            bytes
                .chunks_exact(2)
                .map(|b| f32::from(i16::from_le_bytes([b[0], b[1]])) / 32768.0),
        );
        self.left -= n as u64;
        Ok(())
    }
}

/// 解析 WAV 头，查格式，回 `data` 块在哪。`data` 的长度写错（管道里写的 `0xFFFFFFFF`）时按文件实际长度截。
fn parse_header(r: &mut impl Read, file_len: u64) -> Result<DataChunk, WavError> {
    let mut riff = [0u8; 12];
    read_or_not_wave(r, &mut riff)?;
    if &riff[0..4] != b"RIFF" || &riff[8..12] != b"WAVE" {
        return Err(WavError::NotWave);
    }
    let mut pos = 12u64;
    let mut format_seen = false;
    loop {
        let mut head = [0u8; 8];
        if !read_chunk_head(r, &mut head)? {
            return Err(WavError::NoData);
        }
        pos += 8;
        let id = [head[0], head[1], head[2], head[3]];
        let len = u64::from(u32::from_le_bytes([head[4], head[5], head[6], head[7]]));
        match &id {
            b"fmt " => {
                if len < 16 {
                    return Err(WavError::NoFormat);
                }
                let mut fmt = vec![0u8; len as usize];
                read_or_not_wave(r, &mut fmt)?;
                check_format(&fmt)?;
                format_seen = true;
            }
            b"data" => {
                if !format_seen {
                    return Err(WavError::NoFormat);
                }
                let len = len.min(file_len.saturating_sub(pos));
                return Ok(DataChunk { offset: pos, len });
            }
            _ => {
                io::copy(&mut r.by_ref().take(len), &mut io::sink())?;
            }
        }
        pos += len;
        if len % 2 == 1 {
            io::copy(&mut r.by_ref().take(1), &mut io::sink())?;
            pos += 1;
        }
    }
}

fn check_format(fmt: &[u8]) -> Result<(), WavError> {
    let u16_at = |i: usize| u16::from_le_bytes([fmt[i], fmt[i + 1]]);
    let tag = u16_at(0);
    let pcm = match tag {
        FORMAT_PCM => true,
        // WAVE_FORMAT_EXTENSIBLE：子格式 GUID 的头两个字节是真正的格式码。
        FORMAT_EXTENSIBLE => fmt.len() >= 26 && u16_at(24) == FORMAT_PCM,
        _ => false,
    };
    if !pcm {
        return Err(WavError::NotPcm(tag));
    }
    let channels = u16_at(2);
    if channels != 1 {
        return Err(WavError::Channels(channels));
    }
    let rate = u32::from_le_bytes([fmt[4], fmt[5], fmt[6], fmt[7]]);
    if rate != SAMPLE_RATE {
        return Err(WavError::SampleRate(rate));
    }
    let bits = u16_at(14);
    if bits != 16 {
        return Err(WavError::Bits(bits));
    }
    Ok(())
}

/// 读满 `buf`；文件提前结束算不是 WAV。
fn read_or_not_wave(r: &mut impl Read, buf: &mut [u8]) -> Result<(), WavError> {
    r.read_exact(buf).map_err(|e| match e.kind() {
        io::ErrorKind::UnexpectedEof => WavError::NotWave,
        _ => WavError::Io(e),
    })
}

/// 读一个块头；正好在块边界上结束时回 `false`。
fn read_chunk_head(r: &mut impl Read, head: &mut [u8; 8]) -> Result<bool, WavError> {
    let mut got = 0;
    while got < head.len() {
        match r.read(&mut head[got..]) {
            Ok(0) if got == 0 => return Ok(false),
            Ok(0) => return Err(WavError::NotWave),
            Ok(n) => got += n,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(WavError::Io(e)),
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 拼一个 WAV 头：`fmt ` 块、可选的一个别的块、`data` 块。
    fn wav(
        tag: u16,
        channels: u16,
        rate: u32,
        bits: u16,
        extra: Option<&[u8]>,
        data: &[u8],
    ) -> Vec<u8> {
        let mut fmt = Vec::new();
        fmt.extend(tag.to_le_bytes());
        fmt.extend(channels.to_le_bytes());
        fmt.extend(rate.to_le_bytes());
        fmt.extend((rate * u32::from(channels) * u32::from(bits) / 8).to_le_bytes());
        fmt.extend((channels * bits / 8).to_le_bytes());
        fmt.extend(bits.to_le_bytes());
        let mut body = b"WAVE".to_vec();
        body.extend(b"fmt ");
        body.extend((fmt.len() as u32).to_le_bytes());
        body.extend(&fmt);
        if let Some(extra) = extra {
            body.extend(b"LIST");
            body.extend((extra.len() as u32).to_le_bytes());
            body.extend(extra);
            if extra.len() % 2 == 1 {
                body.push(0);
            }
        }
        body.extend(b"data");
        body.extend((data.len() as u32).to_le_bytes());
        body.extend(data);
        let mut out = b"RIFF".to_vec();
        out.extend((body.len() as u32).to_le_bytes());
        out.extend(body);
        out
    }

    fn parse(bytes: &[u8]) -> Result<DataChunk, WavError> {
        parse_header(&mut &bytes[..], bytes.len() as u64)
    }

    #[test]
    fn accepts_mono_16k_pcm_and_skips_other_chunks() {
        let plain = wav(FORMAT_PCM, 1, 16_000, 16, None, &[1, 0, 2, 0]);
        assert_eq!(parse(&plain).unwrap(), DataChunk { offset: 44, len: 4 });
        // 奇数长的 LIST 块后面有补齐字节。
        let listed = wav(FORMAT_PCM, 1, 16_000, 16, Some(b"INFOabc"), &[1, 0]);
        assert_eq!(parse(&listed).unwrap(), DataChunk { offset: 60, len: 2 });
    }

    #[test]
    fn refuses_everything_else() {
        let cases: [(Vec<u8>, &str); 6] = [
            (wav(FORMAT_PCM, 1, 8_000, 16, None, &[]), "8000 Hz"),
            (wav(FORMAT_PCM, 1, 44_100, 16, None, &[]), "44100 Hz"),
            (wav(FORMAT_PCM, 2, 16_000, 16, None, &[]), "2 channels"),
            (wav(FORMAT_PCM, 1, 16_000, 24, None, &[]), "24 bits"),
            (wav(3, 1, 16_000, 32, None, &[]), "not PCM"),
            (b"ID3\x04 not a wave at all".to_vec(), "not a RIFF"),
        ];
        for (bytes, expected) in cases {
            let e = parse(&bytes).unwrap_err();
            assert!(e.to_string().contains(expected), "{e} / {expected}");
            assert!(!e.is_io());
        }
    }

    #[test]
    fn data_before_fmt_or_missing_is_refused() {
        let mut no_data = wav(FORMAT_PCM, 1, 16_000, 16, None, &[]);
        no_data.truncate(36);
        assert!(matches!(parse(&no_data), Err(WavError::NoData)));
        let mut data_first = b"RIFF\0\0\0\0WAVEdata\x02\0\0\0".to_vec();
        data_first.extend([0, 0]);
        assert!(matches!(parse(&data_first), Err(WavError::NoFormat)));
    }

    #[test]
    fn streamed_length_is_cut_to_the_file() {
        let mut bytes = wav(FORMAT_PCM, 1, 16_000, 16, None, &[1, 0, 2, 0, 3, 0]);
        bytes[40..44].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(parse(&bytes).unwrap(), DataChunk { offset: 44, len: 6 });
    }

    #[test]
    fn extensible_pcm_is_pcm() {
        let mut bytes = wav(FORMAT_EXTENSIBLE, 1, 16_000, 16, None, &[]);
        // 把 fmt 块加长到 40 字节，子格式写 PCM。
        let mut fmt = bytes[20..36].to_vec();
        fmt.extend(22u16.to_le_bytes());
        fmt.extend(16u16.to_le_bytes());
        fmt.extend(0u32.to_le_bytes());
        fmt.extend(FORMAT_PCM.to_le_bytes());
        fmt.extend([0u8; 14]);
        let mut body = b"WAVEfmt ".to_vec();
        body.extend((fmt.len() as u32).to_le_bytes());
        body.extend(fmt);
        body.extend(&bytes[36..]);
        bytes = b"RIFF".to_vec();
        bytes.extend((body.len() as u32).to_le_bytes());
        bytes.extend(body);
        assert!(parse(&bytes).is_ok());
    }

    #[test]
    fn reads_samples_in_blocks() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.wav");
        let data: Vec<u8> = [0i16, 16384, -32768, 32767, 1]
            .iter()
            .flat_map(|s| s.to_le_bytes())
            .collect();
        std::fs::write(&path, wav(FORMAT_PCM, 1, 16_000, 16, Some(b"x"), &data)).unwrap();
        let mut pcm = Pcm16k::open(&path).unwrap();
        assert!((pcm.duration_sec() - 5.0 / 16_000.0).abs() < 1e-12);
        let mut block = Vec::new();
        pcm.read_block(&mut block, 3).unwrap();
        assert_eq!(block, vec![0.0, 0.5, -1.0]);
        pcm.read_block(&mut block, 3).unwrap();
        assert_eq!(block.len(), 2);
        pcm.read_block(&mut block, 3).unwrap();
        assert!(block.is_empty());
    }
}
