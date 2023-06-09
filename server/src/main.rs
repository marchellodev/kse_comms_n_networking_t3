use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::{
    collections::HashMap,
    io::{prelude::*, BufReader, BufWriter, Cursor},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

use crate::calculate::calculate_detached;

mod calculate;

type Matrix = Vec<Vec<u32>>;

const PING: u32 = 5;
const PONG: u32 = 6;
const MATRIX_RECEIVING: u32 = 7;
const MATRIX_RECEIVED: u32 = 8;
const MATRIX_CALCULATE_SUM: u32 = 9;
const MATRIX_SUM_RESULT: u32 = 10;
const MATRIX_SUM_RESULT_NO: u32 = 11;
const MATRIX_SUM_RESULT_SENDING: u32 = 12;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    let matrices: Arc<Mutex<Vec<(Matrix, Matrix)>>> = Arc::new(Mutex::new(Vec::new()));
    let results: Arc<Mutex<HashMap<usize, Matrix>>> = Arc::new(Mutex::new(HashMap::new()));

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        println!("Connection established!");

        let matrices = matrices.clone();
        let results = results.clone();

        thread::spawn(move || {
            handle_connection(stream, &matrices, &results);
        });
    }
}

fn handle_connection(
    stream: TcpStream,
    matrix_store: &Mutex<Vec<(Matrix, Matrix)>>,
    results_store: &Arc<Mutex<HashMap<usize, Matrix>>>,
) {
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
            PING => {
                println!("> Received ping from the server, sendig pong");
                buf_writer.write_u32::<BigEndian>(PONG).unwrap();
                buf_writer.flush().unwrap();
            }
            MATRIX_RECEIVING => {
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
                if size < 10 {
                    print_matrix(&matrix1);
                    println!("");
                    print_matrix(&matrix2);
                } else {
                    println!(" > Matrices are too big to print");
                }

                let mut lock = matrix_store.lock().unwrap();

                let matrix_id = lock.len();

                lock.push((matrix1, matrix2));
                println!(" > Matrice pair stored with id: {}", matrix_id);
                buf_writer.write_u32::<BigEndian>(MATRIX_RECEIVED).unwrap();
                buf_writer.write_u32::<BigEndian>(matrix_id as u32).unwrap();
                buf_writer.flush().unwrap();
            }
            MATRIX_CALCULATE_SUM => {
                println!("> Calculating the sum of the matrices");
                let matrix_id = read_number(&mut buf_reader);
                println!(" > Matrix id: {}", matrix_id);

                let matrices = matrix_store.lock().unwrap()[matrix_id as usize].clone();

                calculate_detached(
                    matrices.0,
                    matrices.1,
                    matrix_id as usize,
                    results_store.clone(),
                );
            }
            MATRIX_SUM_RESULT => {
                println!("> Getting calculation status");
                let matrix_id = read_number(&mut buf_reader);
                println!(" > Matrix id: {}", matrix_id);

                let result = results_store.lock().unwrap();
                let result = result.get(&(matrix_id as usize));

                if result.is_none() {
                    buf_writer
                        .write_u32::<BigEndian>(MATRIX_SUM_RESULT_NO)
                        .unwrap();
                    buf_writer.write_u32::<BigEndian>(matrix_id).unwrap();
                    buf_writer.flush().unwrap();
                    println!(" > No result sent!");

                    continue;
                }

                let result = result.unwrap();

                buf_writer
                    .write_u32::<BigEndian>(MATRIX_SUM_RESULT_SENDING)
                    .unwrap();
                buf_writer.write_u32::<BigEndian>(matrix_id).unwrap();

                buf_writer
                    .write_u32::<BigEndian>(result.len() as u32)
                    .unwrap();

                for row in result {
                    for elem in row {
                        buf_writer.write_u32::<BigEndian>(*elem).unwrap();
                    }
                }
                buf_writer.flush().unwrap();
                println!(" > Result sent!");
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
