use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::{
    io::{prelude::*, BufReader, BufWriter, Cursor},
    net::{TcpListener, TcpStream},
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        println!("Connection established!");
        handle_connection(stream);
    }
}

fn handle_connection(stream: TcpStream) {
    let stream_clone = stream.try_clone().expect("Failed to clone TcpStream.");

    let mut buf_reader = BufReader::new(stream);
    let mut buf_writer = BufWriter::new(stream_clone);

    loop {
        let mut buffer = [0; 4];
        if buf_reader.read_exact(&mut buffer).is_err() {
            println!("Connection closed");
            break;
        }
        let mut rdr = Cursor::new(buffer);
        let decoded = rdr.read_u32::<BigEndian>().unwrap();

        println!("> Received request: {:#?}", decoded);

        match decoded {
            5 => {
                println!("> Received ping from the server, sendig pong");
                buf_writer.write_u32::<BigEndian>(6).unwrap();
                buf_writer.flush().unwrap();
            }
            3 => println!("Three"),
            4 => println!("Four"),
            _ => {
                println!("Unknown request")
            }
        }
    }
}
