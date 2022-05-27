use rand::Rng;
use std::{
    env, fs::File, io::prelude::*, io::BufReader, io::BufWriter, io::Read, ops::Deref,
    process::exit,
};

use reed_solomon::Buffer;
use reed_solomon::Decoder;
use reed_solomon::Encoder;

// Settings
const ENCODED_BLOCK_SIZE: usize = 8;
const DATA_BLOCK_SIZE: usize = 4;

type AllResult = Result<(), Box<dyn std::error::Error>>;

fn encode_file(path_in: &str, path_out: &str) -> AllResult {
    let encoder = Encoder::new(ENCODED_BLOCK_SIZE - DATA_BLOCK_SIZE);

    let file_in = File::open(path_in)?;
    let file_out = File::create(path_out)?;

    let mut filebuf_in = BufReader::new(file_in);
    let mut filebuf_out = BufWriter::new(file_out);

    let mut buf: [u8; DATA_BLOCK_SIZE] = [0; DATA_BLOCK_SIZE];
    while let Ok(n) = filebuf_in.read(&mut buf) {
        // Pad EOF missing bytes
        if n == 0 {
            break;
        } else if n < DATA_BLOCK_SIZE {
            for x in buf[n..DATA_BLOCK_SIZE - 1].iter_mut() {
                *x = 0;
            }
        }
        // Encode
        let encoded = encoder.encode(&buf);
        filebuf_out.write(encoded.deref())?;
    }

    Ok(())
}

fn decode_file(path_in: &str, path_out: &str) -> AllResult {
    let decoder = Decoder::new(ENCODED_BLOCK_SIZE - DATA_BLOCK_SIZE);

    let file_in = File::open(path_in)?;
    let file_out = File::create(path_out)?;

    let mut filebuf_in = BufReader::new(file_in);
    let mut filebuf_out = BufWriter::new(file_out);

    let mut buf: [u8; ENCODED_BLOCK_SIZE] = [0; ENCODED_BLOCK_SIZE];
    while let Ok(n) = filebuf_in.read(&mut buf) {
        // Pad EOF missing bytes
        if n == 0 {
            break;
        }
        // Encode
        let blank_buf = Buffer::from_slice(&[0; ENCODED_BLOCK_SIZE], ENCODED_BLOCK_SIZE);
        let decoded = decoder.correct(&buf, None).unwrap_or(blank_buf);
        filebuf_out.write(decoded.data())?;
    }

    Ok(())
}

fn encode_file_with_err(path_in: &str, path_out: &str) -> AllResult {
    let encoder = Encoder::new(ENCODED_BLOCK_SIZE - DATA_BLOCK_SIZE);

    let file_in = File::open(path_in)?;
    let file_out = File::create(path_out)?;

    let mut filebuf_in = BufReader::new(file_in);
    let mut filebuf_out = BufWriter::new(file_out);

    let mut buf: [u8; DATA_BLOCK_SIZE] = [0; DATA_BLOCK_SIZE];
    while let Ok(n) = filebuf_in.read(&mut buf) {
        // Pad EOF missing bytes
        if n == 0 {
            break;
        } else if n < DATA_BLOCK_SIZE {
            for x in buf[n..DATA_BLOCK_SIZE - 1].iter_mut() {
                *x = 0;
            }
        }

        // Encode
        let mut encoded = encoder.encode(&buf);

        // Add errors
        let mut rng = rand::thread_rng();
        for x in encoded.iter_mut().take(2) {
            *x = rng.gen::<u8>();
        }

        filebuf_out.write(encoded.deref())?;
    }

    Ok(())
}

fn main() -> AllResult {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        println!("Invalid number of args! Try: clrfec <file in> <file out>");
        exit(0);
    }

    // Get paths
    let op = &args[1];
    let path_in = &args[2];
    let path_out = &args[3];

    if op == "encode" {
        encode_file(path_in, path_out)?;
    } else if op == "decode" {
        decode_file(path_in, path_out)?;
    } else if op == "errcode" {
        encode_file_with_err(path_in, path_out)?;
    } else {
        println!("{op} is not a valid op, try 'encode' or 'decode' instead");
    }

    Ok(())
}
