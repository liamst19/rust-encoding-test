use std::error::Error;
use std::fs::File;
use std::io::{
  BufReader,
  BufRead,
  // Read,
};
use std::io::prelude::*;
use encoding_rs::{
  EUC_JP,
  CoderResult,
};
use encoding_rs_io::DecodeReaderBytesBuilder;

fn main() -> Result<(), Box<dyn Error>> {

  encoding_read_test_one()?;
  encoding_read_test_two()?;
  encoding_write_test_one()?;

  Ok(())
}

// modifying encoding_read_test_two for encoding UTF-8 text to EUC-JP
pub fn encoding_write_test_one() -> Result<(), Box<dyn Error>> {

  let input_str = "<html><head><meta charset=\"euc-jp\"></head>
<body>
<p>眼は壁のごつごつ水曲へ晩をしふくましまし。</p>
<p>それでずいぶん生意気ないなという用ですまし。まじめでしだことだもたすると曲の生手のうちをはすっかりだめただながら、おまえだけ晩をやめがっんたず。</p>
<p>し過ぎそれは窓に面白ましていまの次のゴーシュ曲をねむり第六ゴーシュ目の息のちがわてきたた。猫もたくさん込みてしまいまし。</p>
<p>二つは三きれつぶのようをつかれてしまいた。ゴーシュは風眼と何をあるてしまいた。</p>
<p>両手はかっこうをそうに泣きてかっこうに包みのようから叩きてバケツを見えてどんと矢をあいて出しう。もっとこつこつゴーシュをゴムを考えましまし。何どうにガラスへ教えてトォテテテテテイに置いましまし。</p>
<p>猫をこすりんまし。「おっかさんへありた。小麦、みんなをけち。やっ。」</p>
</body></html>";

  let outfile = "./euc_jp_output_text.html";

  let mut buffer = [0u8; 2048];
  let mut bytes_in_buffer = 0usize;
  let mut total_had_errors = false;
  let mut encoder = EUC_JP.new_encoder();

  let mut file = File::create(outfile)?;

  for c in input_str.chars() {
    loop {
      let (result, _, written, had_errors) =
        encoder.encode_from_utf8(
          &c.to_string(),
          &mut buffer[bytes_in_buffer..],
          true // needed for ISO-2022-JP and is ignored for others
        );
      bytes_in_buffer += written;
      total_had_errors |= had_errors;

      match result {
        CoderResult::InputEmpty => {
          break;
        },
        CoderResult::OutputFull => {
          file.write_all(&buffer[..bytes_in_buffer])?;
          bytes_in_buffer = 0usize;
          continue;
        }
      }
    }
  }

  // write the contents remaining in buffer
  if bytes_in_buffer > 0usize {
    file.write_all(&buffer[..bytes_in_buffer])?;
  }

  println!("euc-jp output written to {}", outfile);

  if total_had_errors {
    println!("there were errors while encoding");
  }

  Ok(())
}

// From Stack Overflow Answer, with some modification
// https://stackoverflow.com/questions/64040851/how-can-i-read-a-non-utf8-file-line-by-line-in-rust
pub fn encoding_read_test_one() -> Result<(), Box<dyn Error>> {

  // text file encoded in euc-jp
  let filepath = "./sample_euc_jp_text_1.txt";

  // below will give error because it assumes UTF-8
  // let content = fs::read_to_string("./sample_euc_jp_text.txt")?;
  // println!("{}", content);

  let file = File::open(filepath)?;
  let reader = BufReader::new(
    DecodeReaderBytesBuilder::new()
      .encoding(Some(EUC_JP))
      .build(file));

  // let mut buffer = String::new();
  // reader.read_to_string(&mut buffer)?;
  // println!("{}", buffer);
  for line in reader.lines().map(|l| l.unwrap()) {
    println!("{}", line);
  }

  println!("\nfinished reading {}\n", filepath);

  Ok(())
}

// Based on rust docs for encoding_rs
// https://docs.rs/encoding_rs/0.8.28/encoding_rs/
pub fn encoding_read_test_two() -> Result<(), Box<dyn Error>> {

  // text file encoded in euc-jp
  let filepath = "./sample_euc_jp_text_2.txt";
  let file = File::open(filepath)?;
  // let mut reader = BufReader::new(file);

  // buffer
  let mut buffer_bytes = [0u8; 2048];
  let buffer: &mut str = std::str::from_utf8_mut(&mut buffer_bytes[..]).unwrap();

  // How many bytes in the buffer currently hold significant data.
  let mut bytes_in_buffer = 0usize;

  // Collect the output to a string for demonstration purposes.
  let mut output = String::new();

  // The `Decoder`
  let mut decoder = EUC_JP.new_decoder();

  // Track whether we see errors.
  let mut total_had_errors = false;

  // Decode using a fixed-size intermediate buffer (for demonstrating the
  // use of a fixed-size buffer; normally when the output of an incremental
  // decode goes to a `String` one would use `Decoder.decode_to_string()` to
  // avoid the intermediate buffer).
  for b in file.bytes() {
    let input = vec![b?];

    // The number of bytes already read from current `input` in total.
    let mut total_read_from_current_input = 0usize;

    loop {
      let (result, read, written, had_errors) =
        decoder.decode_to_str(
          &input[total_read_from_current_input..],
          &mut buffer[bytes_in_buffer..],
          false // needed for ISO-2022-JP and is ignored for others
        );
      total_read_from_current_input += read;
      bytes_in_buffer += written;
      total_had_errors |= had_errors;
      match result {
        CoderResult::InputEmpty => {
          // We have consumed the current input buffer. Break out of
          // the inner loop to get the next input buffer from the
          // outer loop.
          break;
        },
        CoderResult::OutputFull => {
          // Write the current buffer out and consider the buffer
          // empty.
          output.push_str(&buffer[..bytes_in_buffer]);
          bytes_in_buffer = 0usize;
          continue;
        }
      }
    }
  }

  // Process EOF
  loop {
    let (result, _, written, had_errors) =
      decoder.decode_to_str(b"",
                            &mut buffer[bytes_in_buffer..],
                            true);
    bytes_in_buffer += written;
    total_had_errors |= had_errors;
    // Write the current buffer out and consider the buffer empty.
    // Need to do this here for both `match` arms, because we exit the
    // loop on `CoderResult::InputEmpty`.
    output.push_str(&buffer[..bytes_in_buffer]);
    bytes_in_buffer = 0usize;
    match result {
      CoderResult::InputEmpty => {
        // Done!
        break;
      },
      CoderResult::OutputFull => {
        continue;
      }
    }
  }

  println!("{}", output);
  println!("finished reading {}\n", filepath);

  if total_had_errors {
    println!("there were errors while decoding");
  }

  Ok(())
}
