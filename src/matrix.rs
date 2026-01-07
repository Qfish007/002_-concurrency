use crate::{dot_product, Vector};
use anyhow::{anyhow, Result};
use std::{
    fmt::{self, Display, Formatter},
    ops::{Add, AddAssign, Mul},
    sync::mpsc,
    thread,
};
const NUM_THREADS: usize = 4;
pub struct Matrix<T> {
    data: Vec<T>,
    row: usize,
    col: usize,
}

// 发送消息给线程进行矩阵乘法计算
pub struct MsgInput<T> {
    idx: usize,
    row: Vector<T>,
    col: Vector<T>,
}

// 线程计算完成后，发送结果消息
pub struct MsgOutput<T> {
    idx: usize,
    value: T,
}

// 消息结构体，包含输入和输出，以及用于发送结果的通道
pub struct Msg<T> {
    input: MsgInput<T>,
    // sender to send the result back
    sender: oneshot::Sender<MsgOutput<T>>, //
}

impl<T> MsgInput<T> {
    pub fn new(idx: usize, row: Vector<T>, col: Vector<T>) -> Self {
        Self { idx, row, col }
    }
}

impl<T> Msg<T> {
    pub fn new(input: MsgInput<T>, sender: oneshot::Sender<MsgOutput<T>>) -> Self {
        Self { input, sender }
    }
}

impl<T> Matrix<T> {
    pub fn new(data: impl Into<Vec<T>>, row: usize, col: usize) -> Self {
        Matrix {
            data: data.into(),
            row,
            col,
        }
    }
}

impl<T> Display for Matrix<T>
where
    T: Display,
{
    // display a 2x3 as {1 2 3, 4 5 6}, 3x2 as {1 2, 3 4, 5 6}
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{{")?;
        for i in 0..self.row {
            for j in 0..self.col {
                write!(f, "{}", self.data[i * self.col + j])?;
                if j != self.col - 1 {
                    write!(f, " ")?;
                }
            }

            if i != self.row - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "}}")?;
        Ok(())
    }
}

impl<T> fmt::Debug for Matrix<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Matrix(row={}, col={}, {})", self.row, self.col, self)
    }
}

pub fn multiply<T>(a: &Matrix<T>, b: &Matrix<T>) -> Result<Matrix<T>>
where
    T: Copy + Default + Add<Output = T> + AddAssign + Mul<Output = T> + Send + 'static,
{
    if a.col != b.row {
        return Err(anyhow!("Matrix multiply error: a.col != b.row"));
    }

    let senders = (0..NUM_THREADS)
        .map(|_| {
            // 1.创建线程和通道
            let (tx, rx) = mpsc::channel::<Msg<T>>();

            // 2.启动线程，【等待接收消息】并处理
            thread::spawn(move || {
                for msg in rx {
                    let value = dot_product(msg.input.row, msg.input.col)?;

                    // 3.计算完成后，通过oneshot通道发送结果回主线程
                    if let Err(e) = msg.sender.send(MsgOutput {
                        idx: msg.input.idx,
                        value,
                    }) {
                        eprintln!("Send error: {:?}", e);
                    }
                }
                Ok::<_, anyhow::Error>(())
            });
            tx
        })
        .collect::<Vec<_>>();

    // generate 4 threads which receive msg and do dot product
    let matrix_len = a.row * b.col;
    let mut data = vec![T::default(); matrix_len];
    let mut receivers = Vec::with_capacity(matrix_len);

    // map/reduce: map phase
    for i in 0..a.row {
        for j in 0..b.col {
            let row = Vector::new(&a.data[i * a.col..(i + 1) * a.col]);
            let col_data = b.data[j..]
                .iter()
                .step_by(b.col)
                .copied()
                .collect::<Vec<_>>();
            let col = Vector::new(col_data);
            let idx = i * b.col + j;
            // 构建消息并发送给线程
            let input = MsgInput::new(idx, row, col);
            // 创建单次性通道用于接收结果
            let (tx, rx) = oneshot::channel();
            // 构建消息
            let msg = Msg::new(input, tx);

            // step1. 通过tx发送消息给线程
            if let Err(e) = senders[idx % NUM_THREADS].send(msg) {
                eprintln!("Send error: {:?}", e);
            }

            // receivers 用于保存所有线程的接收通道
            receivers.push(rx);
        }
    }

    // 这里的是 oneshot::channel 的接收端，等待所有线程计算完成并接收结果
    for rx in receivers {
        let output = rx.recv()?;
        data[output.idx] = output.value;
    }

    Ok(Matrix {
        data,
        row: a.row,
        col: b.col,
    })
}

impl<T> Mul for Matrix<T>
where
    T: Copy + Default + Add<Output = T> + AddAssign + Mul<Output = T> + Send + 'static,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        multiply(&self, &rhs).expect("Matrix multiply error")
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_display1() -> Result<()> {
        let a = Matrix::new(vec![1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new(vec![1, 2, 3, 4, 5, 6], 3, 2);
        let c = multiply(&a, &b)?;
        assert_eq!(c.col, 2);
        assert_eq!(c.row, 2);
        assert_eq!(c.data, vec![22, 28, 49, 64]);
        assert_eq!(format!("{:?}", c), "Matrix(row=2, col=2, {22 28, 49 64})");
        Ok(())
    }

    #[test]
    fn test_matrix_display2() -> Result<()> {
        let a = Matrix::new(vec![1, 2, 3, 4], 2, 2);
        let b = Matrix::new(vec![1, 2, 3, 4], 2, 2);
        let c = multiply(&a, &b)?;
        assert_eq!(c.col, 2);
        assert_eq!(c.row, 2);
        assert_eq!(c.data, vec![7, 10, 15, 22]);
        assert_eq!(format!("{:?}", c), "Matrix(row=2, col=2, {7 10, 15 22})");
        println!("c: {}", c);
        Ok(())
    }
}
