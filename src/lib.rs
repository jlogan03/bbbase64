//! Simple no-std base64 encoder/decoder,
//! inspired by
//! https://dev.to/tiemen/implementing-base64-from-scratch-in-rust-kb1 .
#![no_std]

const UPPERCASEOFFSET: u8 = 65;
const LOWERCASEOFFSET: u8 = 71;
const DIGITOFFSET: u8 = 4;

#[inline]
fn index_to_char(index: u8) -> Result<u8, &'static str> {
    let index = index as u8;

    let ascii_index = match index {
        0..=25 => index.saturating_add(UPPERCASEOFFSET), // A-Z
        26..=51 => index.saturating_add(LOWERCASEOFFSET), // a-z
        52..=61 => index.saturating_sub(DIGITOFFSET),    // 0-9
        62 => 43,                                        // +
        63 => 47,                                        // /

        _ => return Err("Invalid ascii character index encountered"),
    } as u8;

    Ok(ascii_index)
}

#[inline]
fn char_to_index(c: u8) -> Result<u8, &'static str> {
    let base64_index = match c {
        65..=90 => c.saturating_sub(UPPERCASEOFFSET),  // A-Z
        97..=122 => c.saturating_sub(LOWERCASEOFFSET), // a-z
        48..=57 => c.saturating_add(DIGITOFFSET),      // 0-9
        43 => 62,                                      // +
        47 => 63,                                      // /

        _ => return Err("Invalid base64 char encountered"),
    } as u8;

    Ok(base64_index)
}

/// This should only be used on a slice known to be of length 3
#[inline]
fn split_bytes(chunk: &[u8]) -> [u8; 4] {
    [
        &chunk[0] >> 2,
        (&chunk[0] & 0b00000011) << 4 | &chunk[1] >> 4,
        (&chunk[1] & 0b00001111) << 2 | &chunk[2] >> 6,
        &chunk[2] & 0b00111111,
    ]
}

/// This should only be used on a slice known to be of length 4
#[inline]
fn combine_bytes(chunk: &[u8]) -> [u8; 3] {
    [
        (chunk[0] & 0b00111111) << 2 | chunk[1] >> 4,
        (chunk[1] & 0b00001111) << 4 | chunk[2] >> 2,
        (chunk[2] & 0b00000011) << 6 | chunk[3] & 0b00111111,
    ]
}

/// Encode a base64 encoded slice without padding,
/// by lookup table.
///
/// Input length must be a multiple of 3 bytes.
/// Output length must be exactly 4/3 of input length.
///
/// # Errors
/// * If input length is not a multiple of 3 bytes
/// * If output length is not exactly 4/3 of input length
/// * If any invalid base64 characters are encountered
#[inline]
pub fn encode(data: &[u8], out: &mut [u8]) -> Result<(), &'static str> {
    let nin = data.len();
    let nout = out.len();
    let nchunks = nin / 3;

    if nin % 3 != 0 {
        return Err("Input data must be a multiple of 3 bytes");
    } else if nout != (nin * 4 / 3) {
        return Err("Output data length should be 4/3 input data length");
    } else {
        for j in 0..nchunks {
            let d = &data[3 * j..3 * j + 3];
            let o = &mut out[4 * j..4 * j + 4];
            let expanded = split_bytes(d);
            for i in 0..4 {
                o[i] = index_to_char(expanded[i])?;
            }
        }
    }

    Ok(())
}

/// Decode a base64 encoded slice without padding,
/// by lookup table.
///
/// Input length must be a multiple of 4 bytes.
/// Output length must be exactly 3/4 of input length.
///
/// # Errors
/// * If input length is not a multiple of 4 bytes
/// * If output length is not exactly 3/4 of input length
/// * If any invalid base64 characters are encountered
#[inline]
pub fn decode(data: &[u8], out: &mut [u8]) -> Result<(), &'static str> {
    let nin = data.len();
    let nout = out.len();
    let nchunks = nin / 4;

    if nin % 4 != 0 {
        return Err("Input data must be a multiple of 4 bytes");
    } else if nout != (nin * 3 / 4) {
        return Err("Output data length should be 3/4 input data length");
    } else {
        let mut converted = [0_u8; 4];
        for j in 0..nchunks {
            let d = &data[4 * j..4 * j + 4];
            let o = &mut out[3 * j..3 * j + 3];

            // Invert character mapping
            for i in 0..4 {
                converted[i] = char_to_index(d[i])?;
            }

            // Recombine 4 expanded bytes back to 3
            let combined: [u8; 3] = combine_bytes(&converted);
            o.copy_from_slice(&combined);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_mapping() {
        // Check that the char mapping produces is properly invertible
        for i in 0..63_u8 {
            assert_eq!(char_to_index(index_to_char(i).unwrap()).unwrap(), i);
        }
    }

    #[test]
    fn test_encode_decode() {
        // Test encoding and decoding every possible byte
        // (but not every combination of 3-byte sequences)
        let input: &mut [u8; 258] = &mut [0_u8; 258];
        for i in 0..258 {
            input[i] = (i % 256) as u8;
        }
        let output = b"AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8gISIjJCUmJygpKissLS4vMDEyMzQ1Njc4OTo7PD0+P0BBQkNERUZHSElKS0xNTk9QUVJTVFVWV1hZWltcXV5fYGFiY2RlZmdoaWprbG1ub3BxcnN0dXZ3eHl6e3x9fn+AgYKDhIWGh4iJiouMjY6PkJGSk5SVlpeYmZqbnJ2en6ChoqOkpaanqKmqq6ytrq+wsbKztLW2t7i5uru8vb6/wMHCw8TFxsfIycrLzM3Oz9DR0tPU1dbX2Nna29zd3t/g4eLj5OXm5+jp6uvs7e7v8PHy8/T19vf4+fr7/P3+/wAB";

        let input_de_buf = &mut [0_u8; 258];
        let output_ser_buf = &mut [0_u8; 344];

        encode(input, output_ser_buf).unwrap();
        decode(output, input_de_buf).unwrap();

        assert_eq!(input, input_de_buf);
        assert_eq!(output, output_ser_buf);
    }
}
