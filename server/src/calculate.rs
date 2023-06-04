// use rand::rngs::SmallRng;
// use rand::Rng;
// use rand::SeedableRng;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::{thread, time::Instant};

use crate::Matrix;

// const RNG_FROM: i32 = 1_000;
// const RNG_TO: i32 = 9_999;
// const RNG_SEED: [u8; 32] = [1; 32];

const THREADS: usize = 4;
const PRINT_MATRICES: bool = true;


// fn main() {
//     let mut rng = SmallRng::from_seed(RNG_SEED);
//
//     let a = generate_matrix(&mut rng, 4);
//     let b = generate_matrix(&mut rng, 4);
//
//     let result: Arc<Mutex<HashMap<usize, Matrix>>> = Arc::new(Mutex::new(HashMap::new()));
//
//     calculate_detached(a, b, 1, Arc::clone(&result));
//
//     // sleep for 1 second to let detached thread finish
//     thread::sleep(std::time::Duration::from_secs(1));
// }

pub fn calculate_detached(
    a: Matrix,
    b: Matrix,
    id: usize,
    result_map: Arc<Mutex<HashMap<usize, Matrix>>>,
) {
    thread::spawn(move || {
        let result = calculate(a, b);
        let mut result_map = result_map.lock().unwrap();
        result_map.insert(id, result);
        println!("detached calculate finished: {}", id);
        print_matrix(&result_map[&id]);
    });
}

fn calculate(a: Matrix, b: Matrix) -> Matrix {
    let now = Instant::now();
    println!("> program init");

    let a = Arc::new(a);
    let b = Arc::new(b);

    if PRINT_MATRICES {
        print_matrix(&a);
        println!();
        print_matrix(&b);
    }

    let setup_elapsed = now.elapsed();
    println!(
        "> setup finished in {:.4} ms",
        (setup_elapsed.as_secs_f64() * 1000.0)
    );

    let size = a.len();
    let mut sum: Vec<Vec<u32>> = vec![vec![0; size]; size];

    if THREADS == 0 {
        simple_sum(&a, &b, &mut sum);
    } else {
        let mut handles = Vec::with_capacity(THREADS);

        for n in 0..THREADS {
            let a_ref = Arc::clone(&a);
            let b_ref = Arc::clone(&b);
            handles.push(thread::spawn(move || thread_sum(&a_ref, &b_ref, n)));
        }

        // println!("Theads: {}", handles.len());

        let joined = handles.into_iter().map(|h| h.join().unwrap());

        for s in joined {
            for (pos, e) in s.sum.iter().enumerate() {
                let index = s.indices[pos];
                sum[index] = e.clone();
            }
        }
    }

    if PRINT_MATRICES {
        print_matrix(&sum);
    }

    let total_elapsed = now.elapsed();
    println!(
        "> task finished in {:.4} ms",
        ((total_elapsed - setup_elapsed).as_secs_f64() * 1000.0)
    );

    return sum;
}

struct ThreadSumResult {
    sum: Vec<Vec<u32>>,
    indices: Vec<usize>,
}

fn thread_sum(a: &Matrix, b: &Matrix, thread_index: usize) -> ThreadSumResult {
    let mut sum: Vec<Vec<u32>> = Vec::new();
    let mut indices: Vec<usize> = Vec::new();

    let matrix_size = a.len();

    for x in (thread_index..matrix_size).step_by(THREADS) {
        // println!("Thread {} processing {} column", thread_index, x);

        let mut row: Vec<u32> = Vec::with_capacity(matrix_size);
        for j in 0..matrix_size {
            row.push(a[x][j] + b[x][j]);
        }

        sum.push(row);
        indices.push(x);
    }
    return ThreadSumResult { sum, indices };
}

fn simple_sum(a: &Matrix, b: &Matrix, sum: &mut Matrix) {
    let matrix_size = a.len();
    for i in 0..matrix_size {
        for j in 0..matrix_size {
            sum[i][j] = a[i][j] + b[i][j];
        }
    }
}

// fn generate_matrix(rng: &mut SmallRng, size: usize) -> Matrix {
//     let mut arr: Matrix = vec![vec![0; size]; size];
//
//     arr.iter_mut().for_each(|row| {
//         row.iter_mut().for_each(|elem| {
//             let num = rng.gen_range(RNG_FROM..RNG_TO) as u32;
//             *elem = num;
//         })
//     });
//
//     return arr;
// }

fn print_matrix(arr: &Matrix) {
    arr.iter().for_each(|row| {
        print!("[ ");
        row.iter().for_each(|elem| {
            print!("{:5} ", elem);
        });
        println!("]");
    });
}
