use std::time::Duration;

use tokio::{sync::mpsc::{self, UnboundedSender}, time::{sleep_until, Instant}};

#[derive(Debug, Clone)]
enum Compressed {
    Bytes(Vec<u8>),
    Binaries(Vec<u8>)
}

impl Compressed {
    fn to_binaries(&self) -> Vec<u8> {
        match self {
            Compressed::Binaries(binaries) => {
                binaries.to_vec()
            },
            Compressed::Bytes(bytes) => {
                panic!("bytes is not supported")
            }
        }
    }
}

#[tokio::main]
async fn main() {
    operation_head().await;
}

async fn operation_head() {
    //sample data
    let data1 = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let data2 = vec![11, 12, 13, 14, 15, 16, 17, 18, 19, 20];
    let (tx, rx) = mpsc::unbounded_channel::<Compressed>();

    //having a global temp variable is important to decrease the amount of time the programme need to allocate and deallocate memory
    let mut temp: Vec<u8> = vec![];


    //Concurrent task execute in parallel with heavy_duty(), rx.recv() will reactivate the task again if theres data to receive. 
    //if no data, the task will yield to make way for heavy_duty() to run.
    //if theres no other sender live, rx.recv() will return None and we will end the loop
    let res = tokio::task::spawn(async move {
        let mut rx = rx;
        let mut res = vec![];
        loop {
            match rx.recv().await {
                Some(msg) => {
                    println!("received from rx: {:?}", msg.to_binaries());
                    res.extend(msg.to_binaries());
                },
                None => {
                    break
                }
            }
        }
        res
    });

    //heavy duty cannot run in parallel bcs the data processed need to be in align.
    heavy_duty(tx.clone(), data1, &mut temp).await;
    heavy_duty(tx.clone(), data2, &mut temp).await;

    // we drop the tx manually instead of giving the heavy duty the ownership bcs easier to implement it in macro builder
    drop(tx);
    let res = res.await.unwrap();
    println!("res: {:?}", res);
}

async fn heavy_duty(sender: UnboundedSender<Compressed>, data: Vec<u8>, temp: &mut Vec<u8>) {
    for num in data {
        println!("{:?}", num);
        temp.clear();
        temp.push(num);
        sender.send(Compressed::Binaries(temp.clone())).unwrap();
        temp.clear();
        sleep_until(Instant::now() + Duration::from_secs(1)).await;
    }
}
