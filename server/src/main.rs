use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::{
    io::{prelude::*, BufReader, BufWriter, Cursor},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};

type Matrix = Vec<Vec<u32>>;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    let matrices: Arc<Mutex<Vec<(Matrix, Matrix)>>> = Arc::new(Mutex::new(Vec::new()));

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        println!("Connection established!");
        handle_connection(stream, &matrices.clone());
    }
}

fn handle_connection(stream: TcpStream, matrix_store: &Mutex<Vec<(Matrix, Matrix)>>) {
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
            7 => {
                println!(" > Preparing to receive a matrix");
                let size = read_number(&mut buf_reader);
                println!(" > Matrix size: {}x{}", size, size);

                let mut matrix1 = Matrix::new();
                let mut matrix2 = Matrix::new();
                matrix1.resize(size as usize, vec![0; size as usize]);
                matrix2.resize(size as usize, vec![0; size as usize]);

                for i in 0..size {
                    for j in 0..size {
                        matrix1[i as usize][j as usize] = read_number(&mut buf_reader);
                    }
                }

                for i in 0..size {
                    for j in 0..size {
                        matrix2[i as usize][j as usize] = read_number(&mut buf_reader);
                    }
                }

                println!(" > Matrices received:");
                print_matrix(&matrix1);
                println!("");
                print_matrix(&matrix2);
                let mut lock = matrix_store.lock().unwrap();

                let matrix_id = lock.len();

                lock.push((matrix1, matrix2));
                println!(" > Matrice pair stored with id: {}", matrix_id);
                buf_writer.write_u32::<BigEndian>(8).unwrap();
                buf_writer.write_u32::<BigEndian>(matrix_id as u32).unwrap();
                buf_writer.flush().unwrap();
            }
            _ => {
                println!("Unknown request")
            }
        }
    }
}

fn read_number(reader: &mut BufReader<TcpStream>) -> u32 {
    let mut buffer = [0; 4];
    if reader.read_exact(&mut buffer).is_err() {
        println!("Connection closed");
    }

    let mut rdr = Cursor::new(buffer);
    rdr.read_u32::<BigEndian>().unwrap()
}

fn print_matrix(arr: &Matrix) {
    arr.iter().for_each(|row| {
        print!("[ ");
        row.iter().for_each(|elem| {
            print!("{:5} ", elem);
        });
        println!("]");
    });
}
