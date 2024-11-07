use qrcode::{QrCode, EcLevel,  bits::Bits, types::{Version, QrResult, QrError}, types::Mode::{Alphanumeric}};
use qrcode::optimize::{Segment, Optimizer, total_encoded_len};

use qrcode::render::svg;
use image::ImageOutputFormat::{ Png, Jpeg };
use image::Luma;
use base64;
use std::io::Cursor;

use rustler::types::atom::ok;
use rustler::{Atom, Binary, Env, Error, NifUnitEnum, NifStruct, OwnedBinary};

#[derive(NifStruct)]
#[module = "Qrusty.Native.Options"]
struct Options {
    pub width: u32,
    pub height: u32,
    pub error_correction: ECL,
    pub format: Format,
}

#[rustler::nif]
fn svg_nif(data: &str, opts: Options) -> Result<(Atom, String), Error> {
    let code = QrCode::with_error_correction_level(data.as_bytes(), opts.error_correction.t()).unwrap();
    let svg = code.render::<svg::Color>()
                .min_dimensions(opts.width, opts.height)
                .build();
    Ok((ok(), svg))
}

#[rustler::nif]
fn image_binary_nif<'a>(env: Env<'a>, data: &str, opts: Options) -> Result<(Atom, Binary<'a>), Error> {
    let bytes = create_qr_image(data, opts);
    Ok((ok(), bytes_to_binary(env, bytes)))
}

#[rustler::nif]
fn image_base64_nif(data: &str, opts: Options) -> Result<(Atom, String), Error> {
    let bytes = create_qr_image(data, opts);
    Ok((ok(), base64::encode(&bytes)))
}

#[rustler::nif]
fn svg_alphanumeric(data: &str, opts: Options) -> Result<(Atom, String), Error> {
  let qr_bits = encode_bits(data, opts.error_correction.t()).unwrap();

  let code = QrCode::with_bits(qr_bits, opts.error_correction.t()).unwrap();

  let svg = code.render::<svg::Color>()
    .min_dimensions(opts.width, opts.height)
    .build();
  Ok((ok(), svg))
}

fn create_qr_image(data: &str, opts: Options) -> Vec<u8> {
    let code = QrCode::with_error_correction_level(data.as_bytes(), opts.error_correction.t()).unwrap();
    let img = code.render::<Luma<u8>>()
                .min_dimensions(opts.width, opts.height)
                .build();
    let mut bytes: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), opts.format.t().unwrap()).unwrap();
    bytes
}

fn bytes_to_binary<'a>(env: Env<'a>, bytes: Vec<u8>) -> Binary {
    let mut bin = OwnedBinary::new(bytes.len()).unwrap();
    bin.as_mut_slice().copy_from_slice(&bytes);
    bin.release(env)
}

#[derive(NifUnitEnum)]
pub enum ECL {
    L,
    M,
    Q,
    H
}

fn encode_bits(data: &str, ec_level: EcLevel) -> QrResult<Bits> {
  let segments = vec![Segment { mode: Alphanumeric, begin: 0, end: data.len() }];
  for version in &[Version::Normal(9), Version::Normal(26), Version::Normal(40)] {
      let opt_segments = Optimizer::new(segments.iter().copied(), *version).collect::<Vec<_>>();
      let total_len = total_encoded_len(&opt_segments, *version);
      let data_capacity = version.fetch(ec_level, &DATA_LENGTHS).expect("invalid DATA_LENGTHS");
      if total_len <= data_capacity {
          let min_version = find_min_version(total_len, ec_level);
          let mut bits = Bits::new(min_version);
          bits.push_segments(data.as_bytes(), opt_segments.into_iter())?;
          bits.push_terminator(ec_level)?;
          return Ok(bits);
      }
  }
  Err(QrError::DataTooLong)
}

fn find_min_version(length: usize, ec_level: EcLevel) -> Version {
  let mut base = 0_usize;
  let mut size = 39;
  while size > 1 {
      let half = size / 2;
      let mid = base + half;
      // mid is always in [0, size).
      // mid >= 0: by definition
      // mid < size: mid = size / 2 + size / 4 + size / 8 ...
      base = if DATA_LENGTHS[mid][ec_level as usize] > length { base } else { mid };
      size -= half;
  }
  // base is always in [0, mid) because base <= mid.
  base = if DATA_LENGTHS[base][ec_level as usize] >= length { base } else { base + 1 };
  Version::Normal(as_i16(base + 1))
}

fn as_i16(int: usize) -> i16 {
  i16::try_from(int).unwrap()
}

static DATA_LENGTHS: [[usize; 4]; 44] = [
    // Normal versions
    [152, 128, 104, 72],
    [272, 224, 176, 128],
    [440, 352, 272, 208],
    [640, 512, 384, 288],
    [864, 688, 496, 368],
    [1088, 864, 608, 480],
    [1248, 992, 704, 528],
    [1552, 1232, 880, 688],
    [1856, 1456, 1056, 800],
    [2192, 1728, 1232, 976],
    [2592, 2032, 1440, 1120],
    [2960, 2320, 1648, 1264],
    [3424, 2672, 1952, 1440],
    [3688, 2920, 2088, 1576],
    [4184, 3320, 2360, 1784],
    [4712, 3624, 2600, 2024],
    [5176, 4056, 2936, 2264],
    [5768, 4504, 3176, 2504],
    [6360, 5016, 3560, 2728],
    [6888, 5352, 3880, 3080],
    [7456, 5712, 4096, 3248],
    [8048, 6256, 4544, 3536],
    [8752, 6880, 4912, 3712],
    [9392, 7312, 5312, 4112],
    [10208, 8000, 5744, 4304],
    [10960, 8496, 6032, 4768],
    [11744, 9024, 6464, 5024],
    [12248, 9544, 6968, 5288],
    [13048, 10136, 7288, 5608],
    [13880, 10984, 7880, 5960],
    [14744, 11640, 8264, 6344],
    [15640, 12328, 8920, 6760],
    [16568, 13048, 9368, 7208],
    [17528, 13800, 9848, 7688],
    [18448, 14496, 10288, 7888],
    [19472, 15312, 10832, 8432],
    [20528, 15936, 11408, 8768],
    [21616, 16816, 12016, 9136],
    [22496, 17728, 12656, 9776],
    [23648, 18672, 13328, 10208],
    // Micro versions
    [20, 0, 0, 0],
    [40, 32, 0, 0],
    [84, 68, 0, 0],
    [128, 112, 80, 0],
];

// wrap qrcode's EcLevel enum
impl ECL {
    fn t(&self) -> EcLevel {
        match self {
            ECL::L => EcLevel::L,
            ECL::M => EcLevel::M,
            ECL::Q => EcLevel::Q,
            ECL::H => EcLevel::H,
        }
    }
}

#[derive(NifUnitEnum)]
pub enum Format {
    JPG,
    JPEG,
    PNG,
    JPG64,
    JPEG64,
    PNG64,
    SVG
}

// wrap image's ImageOutputFormat enum
impl Format {
    fn t(&self) -> Result<image::ImageOutputFormat, &str> {
        match self {
            Format::JPG    => Ok(Jpeg(100)),
            Format::JPEG   => Ok(Jpeg(100)),
            Format::PNG    => Ok(Png),
            Format::PNG64  => Ok(Png),
            Format::JPG64  => Ok(Jpeg(100)),
            Format::JPEG64 => Ok(Jpeg(100)),
            _ => Err("Not Implemented")
        }
    }
}

rustler::init!("Elixir.Qrusty.Native", [svg_nif, image_binary_nif, image_base64_nif, svg_alphanumeric]);