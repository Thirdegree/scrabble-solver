use prost::{length_delimiter_len, Message};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, ToSocketAddrs},
    task::JoinSet,
};

use crate::solver;
mod messages {
    include!(concat!(env!("OUT_DIR"), "/messages.rs"));
}

pub struct Server {
    reader: TcpListener,
}
async fn serve_client(mut socket: TcpStream) {
    let read_buf = &mut [0; 4096];
    let mut write_dst: Vec<u8> = vec![0; 4096];
    let solver = solver::NaiveSolver::new();
    loop {
        let n = match socket.read(read_buf).await {
            Ok(n) if n == 0 => {
                eprintln!("Client exited");
                return;
            }
            Ok(n) => n,
            Err(e) => {
                eprintln!("Ohno: {:?}", e);
                return;
            }
        };
        let mut read = 0;
        while read < n {
            match messages::WordRequest::decode_length_delimited(&read_buf[read..n]) {
                Ok(message) => {
                    eprintln!("Message: {message:?}");
                    read += message.encoded_len() + length_delimiter_len(message.encoded_len());
                    let words = match message.kind() {
                        messages::SolverKind::Ouija => {
                            solver.valid_words_ouija(message.letters.chars())
                        }
                        messages::SolverKind::Scrabble => {
                            solver.valid_words_scrabble(message.letters.chars())
                        }
                    };
                    let reply = messages::WordsReply { words };
                    if write_dst.len() < reply.encoded_len() + 10 {
                        write_dst.resize(reply.encoded_len() + 10, 0);
                    }
                    let mut write_buf = &mut write_dst[..];
                    reply.encode_length_delimited(&mut write_buf).unwrap();
                    let reply_byte_len =
                        reply.encoded_len() + length_delimiter_len(reply.encoded_len());
                    let mut written = 0;

                    while written < reply_byte_len {
                        println!("Writting next {} bytes", reply_byte_len - written);
                        match socket.write(&write_dst[written..reply_byte_len]).await {
                            Ok(n) => {
                                written += n;
                                eprintln!(
                                    "Replied with {n} ({written}/{reply_byte_len}) bytes of message of length {}",
                                    reply.encoded_len()
                                );
                            }
                            Err(e) => {
                                eprintln!("{e:?}");
                                return;
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: {e:?}");
                    break;
                }
            }
        }
    }
}

impl Server {
    pub async fn bind<A>(addr: A) -> std::io::Result<Self>
    where
        A: ToSocketAddrs,
    {
        let listener = TcpListener::bind(addr).await?;
        Ok(Self { reader: listener })
    }

    pub async fn run(&mut self) {
        let mut clients = JoinSet::new();
        loop {
            let res = self.reader.accept().await;
            if let Ok((socket, _)) = res {
                clients.spawn(serve_client(socket));
            }
        }
    }
}

#[cfg(test)]
mod test {

    use tokio::io::AsyncWriteExt;

    use super::*;

    #[test]
    fn test_message_encodes_to_buffer() {
        let message = messages::WordRequest {
            letters: "hello".to_string(),
            kind: messages::SolverKind::Ouija.into(),
        };
        let mut dst = [0; 8];
        let mut buf = &mut dst[..];
        message.encode_length_delimited(&mut buf).unwrap();
        assert_eq!(dst[..].to_vec(), vec![7, 10, 5, 104, 101, 108, 108, 111]);
    }

    #[tokio::test]
    async fn test_server_reading() {
        let server = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = server.local_addr().unwrap();
        let serve_task = tokio::spawn(async move {
            loop {
                let (socket, _) = server.accept().await.unwrap();
                serve_client(socket).await;
            }
        });
        let mut client = TcpStream::connect(addr.to_string()).await.unwrap();
        let mut dst = [0; 4098];
        let mut buf = &mut dst[..];
        let message = messages::WordRequest {
            letters: "hello".to_string(),
            kind: messages::SolverKind::Ouija.into(),
        };
        message.encode_length_delimited(&mut buf).unwrap();
        let encoded = &dst[..message.encoded_len() + length_delimiter_len(message.encoded_len())];
        let written = client.write(encoded).await.unwrap();
        assert!(written >= message.encoded_len() + length_delimiter_len(message.encoded_len()));
        let mut read_buf = [0; 4098];
        let reply_len = client.read(&mut read_buf).await.unwrap();
        let reply_msg =
            messages::WordsReply::decode_length_delimited(&read_buf[..reply_len]).unwrap();
        assert_eq!(
            reply_msg,
            messages::WordsReply {
                words: vec![
                    "ee", "eel", "eh", "el", "ell", "he", "heel", "heh", "hele", "hell",
                    "hellhole", "hello", "helo", "ho", "hoe", "hoh", "hole", "hollo", "holloo",
                    "holo", "hoo", "lee", "lo", "loll", "loo", "oe", "oh", "oho", "ole", "oleo",
                    "oo", "ooh"
                ]
                .iter()
                .map(|s| s.to_string())
                .collect()
            }
        );
        serve_task.abort();
        serve_task.await.unwrap_err();
    }

    #[tokio::test]
    async fn test_server_reading_very_large_response() {
        let server = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = server.local_addr().unwrap();
        let serve_task = tokio::spawn(async move {
            loop {
                let (socket, _) = server.accept().await.unwrap();
                serve_client(socket).await;
            }
        });
        let mut client = TcpStream::connect(addr.to_string()).await.unwrap();
        let mut dst = [0; 4098];
        let mut buf = &mut dst[..];
        let message = messages::WordRequest {
            letters: "abcdefghijklmnopqrstuvwxyz".to_string(),
            kind: messages::SolverKind::Ouija.into(),
        };
        message.encode_length_delimited(&mut buf).unwrap();
        let encoded = &dst[..message.encoded_len() + length_delimiter_len(message.encoded_len())];
        let written = client.write(encoded).await.unwrap();
        assert!(written >= message.encoded_len() + length_delimiter_len(message.encoded_len()));
        let mut tot_read = 0;
        let mut read_buf = vec![0; 3500000];
        let mut parsed_reply: Option<messages::WordsReply> = None;
        loop {
            let reply_len = client.read(&mut read_buf[tot_read..]).await.unwrap();
            tot_read += reply_len;
            println!("Got some data ({tot_read})");
            if let Ok(reply_msg) =
                messages::WordsReply::decode_length_delimited(&read_buf[..tot_read])
            {
                parsed_reply = Some(reply_msg);
                break;
            };
        }
        let mut words: Vec<String> = include_str!("wordlist.txt")
            .split('\n')
            .map(|s| s.to_string())
            .collect();
        words.sort();
        assert_eq!(parsed_reply.unwrap(), messages::WordsReply { words });
        serve_task.abort();
        serve_task.await.unwrap_err();
    }
}
