use std::{
    fs::File, 
    net::{TcpListener, TcpStream}, 
    io::{Write, Read, Result}, 
    thread,
    str, sync::Arc
};
use clap::Parser;

#[derive(Parser)]
struct Args {
   // Index file which will be returned on "/" route
   #[arg(short, long)]
   index: Option<String>,

   // Serving directory
   #[arg(short, long)]
   dir: Option<String>,

   // Port on which the socket will be binded
   #[arg(short, long)]
   port: u16
}

fn main() -> Result<()>{
    let args = Arc::new(Args::parse());
    let listener = TcpListener::bind(format!("127.0.0.1:{}", args.port))?;

    for con in listener.incoming() {
        handle_stream(con?, args.clone());
    }

    Ok(())
}

fn handle_stream(mut stream: TcpStream, args: Arc<Args>) {
    thread::spawn(move || {
        // READING STREAM
        let stream_request_string = read_stream(&mut stream);
        let http_request = stream_request_string.split("\r\n").collect::<Vec<&str>>();

        let request_url = http_request[0].split(" ").collect::<Vec<&str>>()[1];

        let index = args.index.clone().unwrap_or("index.html".to_string());
        let directory = args.dir.clone().unwrap_or(".".to_string());

        // WRITING ANSWER
        let mut http_body: Vec<u8> = Vec::new();
        let file_path = || {
            if request_url == "/" {
                return format!("{}/{}", directory, index)
            }
            format!("{}{}", directory, request_url)
        };
        
        file_to_http_body(file_path(), &mut http_body);

        stream.write(&http_body).expect("lol");
        stream.flush().unwrap();
    });
}

fn read_stream(stream: &mut TcpStream) -> String {
    let mut res_buf = vec![];
    loop {
        let mut buf = vec![0; 1000];
        stream.read(&mut buf).unwrap();
        res_buf.extend(buf.iter());

        let stringed = String::from_utf8_lossy(&res_buf);
        if stringed.contains("\r\n\r\n") {
            // END OF HEADERS
            break;
        }
    }

    let res_string = String::from_utf8_lossy(&res_buf).into_owned();

    res_string
        .split("\r\n\r\n")
        .map(|s| s.to_string())
        .collect::<Vec<String>>()[0]
        .clone()
}

fn file_to_http_body(file_url: String, http_body: &mut Vec<u8>) {
    let mut file_content = Vec::new();
    let file = File::open(file_url.clone()).ok();

    if file.is_none() {
        http_body.extend("HTTP/1.1 404 NOT FOUND\r\n\r\n".as_bytes());
        return;
    }

    let file_mime_type = {
        let name = file_url.split("/").last().unwrap().to_string();
        let mime = mime_guess::from_path(name).first_or_octet_stream();

        mime.to_string()
    };

    file.unwrap().read_to_end(&mut file_content).expect("file err");

    let headers = [
        "HTTP/1.1 200 OK".to_string(),
        format!("Content-type: {}", file_mime_type),
        format!("Content-length: {}", file_content.len()),
        "\r\n".to_string()
    ];

    http_body.extend(
        headers.join("\r\n")
            .to_string()
            .into_bytes()
    );
    http_body.extend(file_content);
}
